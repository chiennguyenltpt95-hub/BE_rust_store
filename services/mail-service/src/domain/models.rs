use serde::{Deserialize, Serialize};

/// Một email address đã được validate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailAddress {
    pub email: String,
    pub name: Option<String>,
}

impl MailAddress {
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: None,
        }
    }

    pub fn with_name(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: Some(name.into()),
        }
    }
}

/// Nội dung email — hỗ trợ cả plain text và HTML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailBody {
    pub text: Option<String>,
    pub html: Option<String>,
}

/// Model chính đại diện cho một email cần gửi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailMessage {
    pub from: MailAddress,
    pub to: Vec<MailAddress>,
    pub subject: String,
    pub body: MailBody,
}

/// Builder pattern — giúp tạo MailMessage linh hoạt, dễ đọc
pub struct MailMessageBuilder {
    from: Option<MailAddress>,
    to: Vec<MailAddress>,
    subject: Option<String>,
    text: Option<String>,
    html: Option<String>,
}

impl MailMessageBuilder {
    pub fn new() -> Self {
        Self {
            from: None,
            to: Vec::new(),
            subject: None,
            text: None,
            html: None,
        }
    }

    pub fn from(mut self, addr: MailAddress) -> Self {
        self.from = Some(addr);
        self
    }

    pub fn to(mut self, addr: MailAddress) -> Self {
        self.to.push(addr);
        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn html(mut self, html: impl Into<String>) -> Self {
        self.html = Some(html.into());
        self
    }

    pub fn build(self) -> Result<MailMessage, String> {
        let from = self.from.ok_or("'from' address is required")?;
        if self.to.is_empty() {
            return Err("At least one 'to' address is required".into());
        }
        let subject = self.subject.ok_or("'subject' is required")?;
        if self.text.is_none() && self.html.is_none() {
            return Err("Either 'text' or 'html' body is required".into());
        }

        Ok(MailMessage {
            from,
            to: self.to,
            subject,
            body: MailBody {
                text: self.text,
                html: self.html,
            },
        })
    }
}

impl Default for MailMessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}
