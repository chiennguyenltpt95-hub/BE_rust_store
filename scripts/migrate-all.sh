#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

APP_ENV="${APP_ENV:-dev}"
LOCAL_DATABASE_URL="${LOCAL_DATABASE_URL:-postgresql://db-jayden.c1gymyg48fuo.ap-southeast-1.rds.amazonaws.com:5432/postgres?user=jayden&password=db-jayden}"
USE_LOCAL_DB="${USE_LOCAL_DB:-0}"
CHECK_MIGRATION_CONFLICTS="${CHECK_MIGRATION_CONFLICTS:-1}"
RUN_DRIFT_AUDIT="${RUN_DRIFT_AUDIT:-0}"

if [[ -z "${STRICT_MIGRATIONS:-}" ]]; then
  case "${APP_ENV,,}" in
    prod|production|staging)
      STRICT_MIGRATIONS="1"
      ;;
    *)
      STRICT_MIGRATIONS="0"
      ;;
  esac
fi

# Keep this list explicit for predictable migration ordering.
SERVICES=(
  "user-service"
  "product-service"
  "comment-service"
  "cart-service"
  "checkout-service"
  "order-service"
  "inventory-service"
  "notification-service"
  "shipping-service"
)

if ! command -v sqlx >/dev/null 2>&1; then
  echo "sqlx CLI is required for migrate-all. Install with: cargo install sqlx-cli --no-default-features --features rustls,postgres"
  exit 1
fi

check_migration_conflicts() {
  mapfile -t migration_files < <(find "$ROOT/services" -type f -path '*/migrations/*.sql' | sort)

  if [[ "${#migration_files[@]}" -eq 0 ]]; then
    echo "No migration files found."
    return 0
  fi

  declare -A seen_versions
  local has_error=0

  for file in "${migration_files[@]}"; do
    local service
    local base
    local version
    local service_versions_key
    local prev_version

    service="$(sed -nE 's#.*services/([^/]+)/migrations/.*#\1#p' <<< "$file")"
    if [[ -z "$service" ]]; then
      echo "[ERROR] Could not determine service from path: $file"
      has_error=1
      continue
    fi
    base="$(basename "$file")"

    if [[ ! "$base" =~ ^([0-9]+)_(.+)\.sql$ ]]; then
      echo "[ERROR] Invalid migration name format: $file"
      echo "        Expected: <numeric_version>_<description>.sql"
      has_error=1
      continue
    fi

    version="${BASH_REMATCH[1]}"

    if [[ -n "${seen_versions[$version]:-}" ]]; then
      echo "[ERROR] Duplicate migration version '$version'"
      echo "        - ${seen_versions[$version]}"
      echo "        - $file"
      has_error=1
    else
      seen_versions[$version]="$file"
    fi

    service_versions_key="svc:${service}"
    if [[ -z "${seen_versions[$service_versions_key]:-}" ]]; then
      seen_versions[$service_versions_key]="$version"
    else
      prev_version="${seen_versions[$service_versions_key]}"
      if (( 10#$version <= 10#$prev_version )); then
        echo "[ERROR] Non-increasing migration version order in $service"
        echo "        Previous: $prev_version"
        echo "        Current : $version ($file)"
        has_error=1
      fi
      seen_versions[$service_versions_key]="$version"
    fi
  done

  if [[ "$has_error" -ne 0 ]]; then
    echo "Migration conflict check failed."
    return 1
  fi

  echo "Migration conflict check passed."
  return 0
}

resolve_database_url() {
  local service="$1"

  if [[ "$USE_LOCAL_DB" == "1" ]]; then
    echo "$LOCAL_DATABASE_URL"
    return 0
  fi

  local env_file="$ROOT/services/$service/.env"
  if [[ -f "$env_file" ]]; then
    local from_env
    from_env="$(awk -F= '/^DATABASE_URL=/{print substr($0, index($0,$2)); exit}' "$env_file")"
    if [[ -n "$from_env" ]]; then
      echo "$from_env"
      return 0
    fi
  fi

  if [[ -n "${DATABASE_URL:-}" ]]; then
    echo "$DATABASE_URL"
    return 0
  fi

  echo ""
}

run_service_migrations() {
  local service="$1"
  local source_dir="$ROOT/services/$service/migrations"

  if [[ ! -d "$source_dir" ]]; then
    echo "- $service: no migrations directory, skipping"
    return 0
  fi

  if [[ -z "$(find "$source_dir" -maxdepth 1 -type f -name '*.sql' 2>/dev/null)" ]]; then
    echo "- $service: no migration files, skipping"
    return 0
  fi

  local database_url
  database_url="$(resolve_database_url "$service")"
  if [[ -z "$database_url" ]]; then
    echo "- $service: missing DATABASE_URL, cannot run migrations"
    return 1
  fi

  echo "- $service: running migrations"

  local output
  if ! output="$(sqlx migrate run --source "$source_dir" --database-url "$database_url" 2>&1)"; then
    if [[ "$STRICT_MIGRATIONS" == "1" ]]; then
      echo "$output"
      return 1
    fi

    if grep -qi "previously applied but has been modified" <<< "$output"; then
      echo "  warn: checksum mismatch detected, skipping in local/dev"
      return 0
    fi

    if grep -qi "previously applied but is missing in the resolved migrations" <<< "$output"; then
      echo "  warn: missing historical migration detected, skipping in local/dev"
      return 0
    fi

    echo "$output"
    return 1
  fi

  if [[ -n "$output" ]]; then
    echo "$output"
  fi

  return 0
}

if [[ "$CHECK_MIGRATION_CONFLICTS" == "1" ]]; then
  echo "Running migration conflict preflight..."
  check_migration_conflicts
fi

if [[ "$RUN_DRIFT_AUDIT" == "1" ]]; then
  echo "Running migration drift audit..."
  FAIL_ON_DRIFT="${FAIL_ON_DRIFT:-0}" "$ROOT/scripts/audit-migration-drift.sh"
fi

echo "Migration policy: APP_ENV=$APP_ENV STRICT_MIGRATIONS=$STRICT_MIGRATIONS CHECK_MIGRATION_CONFLICTS=$CHECK_MIGRATION_CONFLICTS RUN_DRIFT_AUDIT=$RUN_DRIFT_AUDIT"

echo "Running DB migrations sequentially..."
for service in "${SERVICES[@]}"; do
  run_service_migrations "$service"
done

echo "Migration phase completed."
