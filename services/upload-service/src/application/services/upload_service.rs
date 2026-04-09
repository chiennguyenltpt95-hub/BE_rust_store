use std::time::Duration;

use aws_config::Region;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client;
use domain_core::error::DomainError;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateUploadPresignRequest {
    #[serde(alias = "fileName")]
    pub file_name: String,
    #[serde(alias = "contentType")]
    pub content_type: Option<String>,
    pub folder: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateUploadPresignResponse {
    pub method: String,
    pub upload_url: String,
    pub object_key: String,
    pub required_headers: Vec<(String, String)>,
    pub expires_in_seconds: u64,
}

pub struct UploadAppService {
    bucket: String,
    key_prefix: String,
    expires_in_seconds: u64,
    client: Client,
}

impl UploadAppService {
    pub async fn new(
        aws_region: String,
        s3_bucket: String,
        s3_key_prefix: String,
        s3_presign_expires_seconds: u64,
        s3_endpoint_url: Option<String>,
    ) -> Result<Self, DomainError> {
        let shared_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(Region::new(aws_region))
            .load()
            .await;

        let mut config_builder = aws_sdk_s3::config::Builder::from(&shared_config);

        if let Some(endpoint) = s3_endpoint_url {
            if !endpoint.trim().is_empty() {
                config_builder = config_builder.endpoint_url(endpoint).force_path_style(true);
            }
        }

        Ok(Self {
            bucket: s3_bucket,
            key_prefix: s3_key_prefix,
            expires_in_seconds: s3_presign_expires_seconds,
            client: Client::from_conf(config_builder.build()),
        })
    }

    #[instrument(skip(self, req))]
    pub async fn create_presigned_upload_url(
        &self,
        req: CreateUploadPresignRequest,
    ) -> Result<CreateUploadPresignResponse, DomainError> {
        let file_name = req.file_name.trim();
        if file_name.is_empty() {
            return Err(DomainError::ValidationError(
                "file_name cannot be empty".into(),
            ));
        }

        let folder = req
            .folder
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(sanitize_folder);

        let key = build_object_key(
            &self.key_prefix,
            folder.as_deref(),
            &Uuid::new_v4().to_string(),
            &sanitize_file_name(file_name),
        );

        let mut op = self.client.put_object().bucket(&self.bucket).key(&key);

        if let Some(content_type) = req.content_type {
            let ct = content_type.trim().to_string();
            if !ct.is_empty() {
                op = op.content_type(ct);
            }
        }

        let presign_config =
            PresigningConfig::expires_in(Duration::from_secs(self.expires_in_seconds))
                .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        let presigned_req = op
            .presigned(presign_config)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        let required_headers: Vec<(String, String)> = presigned_req
            .headers()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Ok(CreateUploadPresignResponse {
            method: "PUT".into(),
            upload_url: presigned_req.uri().to_string(),
            object_key: key,
            required_headers,
            expires_in_seconds: self.expires_in_seconds,
        })
    }
}

fn sanitize_file_name(input: &str) -> String {
    let mut out = String::with_capacity(input.len());

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }

    let trimmed = out.trim_matches('_').trim_matches('.');
    if trimmed.is_empty() {
        "file.bin".into()
    } else {
        trimmed.to_string()
    }
}

fn sanitize_folder(input: &str) -> String {
    let mut out = String::with_capacity(input.len());

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '/' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }

    out.trim_matches('/').to_string()
}

fn build_object_key(prefix: &str, folder: Option<&str>, id: &str, file_name: &str) -> String {
    let mut parts = Vec::new();

    let p = prefix.trim_matches('/');
    if !p.is_empty() {
        parts.push(p.to_string());
    }

    if let Some(folder) = folder {
        let f = folder.trim_matches('/');
        if !f.is_empty() {
            parts.push(f.to_string());
        }
    }

    parts.push(format!("{}-{}", id, file_name));
    parts.join("/")
}
