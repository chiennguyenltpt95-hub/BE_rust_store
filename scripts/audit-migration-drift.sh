#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

LOCAL_DATABASE_URL="${LOCAL_DATABASE_URL:-postgresql://db-jayden.c1gymyg48fuo.ap-southeast-1.rds.amazonaws.com:5432/postgres?user=jayden&password=db-jayden}"
USE_LOCAL_DB="${USE_LOCAL_DB:-0}"
OUTPUT_PATH="${OUTPUT_PATH:-$ROOT/logs/migration-drift-report.txt}"
FAIL_ON_DRIFT="${FAIL_ON_DRIFT:-0}"

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
  echo "sqlx CLI is required for migration drift audit. Install with: cargo install sqlx-cli --no-default-features --features rustls,postgres"
  exit 1
fi

mkdir -p "$(dirname "$OUTPUT_PATH")"

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

append_recommendation() {
  local message="$1"
  {
    echo "    Recommendation: $message"
  } >> "$OUTPUT_PATH"
}

report_header() {
  {
    echo "Migration Drift Audit Report"
    echo "Generated at: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    echo "Workspace: $ROOT"
    echo ""
  } > "$OUTPUT_PATH"
}

audit_service() {
  local service="$1"
  local source_dir="$ROOT/services/$service/migrations"

  {
    echo "[$service]"
  } >> "$OUTPUT_PATH"

  if [[ ! -d "$source_dir" ]]; then
    echo "  status: skipped (no migrations directory)" >> "$OUTPUT_PATH"
    echo "" >> "$OUTPUT_PATH"
    return 0
  fi

  if [[ -z "$(find "$source_dir" -maxdepth 1 -type f -name '*.sql' 2>/dev/null)" ]]; then
    echo "  status: skipped (no migration files)" >> "$OUTPUT_PATH"
    echo "" >> "$OUTPUT_PATH"
    return 0
  fi

  local database_url
  database_url="$(resolve_database_url "$service")"
  if [[ -z "$database_url" ]]; then
    echo "  status: error (missing DATABASE_URL)" >> "$OUTPUT_PATH"
    append_recommendation "Set DATABASE_URL in services/$service/.env or export DATABASE_URL before running audit."
    echo "" >> "$OUTPUT_PATH"
    return 1
  fi

  local output
  local has_drift=0

  if output="$(sqlx migrate info --source "$source_dir" --database-url "$database_url" 2>&1)"; then
    echo "  status: clean" >> "$OUTPUT_PATH"
  else
    echo "  status: drift-detected" >> "$OUTPUT_PATH"
    has_drift=1

    if grep -qi "previously applied but is missing in the resolved migrations" <<< "$output"; then
      echo "  drift: missing migration file that was already applied in DB" >> "$OUTPUT_PATH"
      append_recommendation "Restore the missing migration file from VCS history, or create a baseline/reconciliation plan before enabling strict mode."
    fi

    if grep -qi "previously applied but has been modified" <<< "$output"; then
      echo "  drift: applied migration file checksum changed" >> "$OUTPUT_PATH"
      append_recommendation "Revert edited applied migration file to original checksum and create a new migration for further changes."
    fi

    if [[ "$has_drift" -eq 1 ]] && ! grep -qi "previously applied but is missing in the resolved migrations\|previously applied but has been modified" <<< "$output"; then
      echo "  drift: unclassified sqlx migration error" >> "$OUTPUT_PATH"
      append_recommendation "Review sqlx output and DB migration table _sqlx_migrations for manual reconciliation."
    fi

    {
      echo "  details:"
      while IFS= read -r line; do
        echo "    $line"
      done <<< "$output"
    } >> "$OUTPUT_PATH"
  fi

  echo "" >> "$OUTPUT_PATH"

  return "$has_drift"
}

report_header

drift_count=0
error_count=0

for service in "${SERVICES[@]}"; do
  if ! audit_service "$service"; then
    drift_count=$((drift_count + 1))
  fi
done

if grep -q "status: error" "$OUTPUT_PATH"; then
  error_count=1
fi

{
  echo "Summary"
  echo "  drift_services: $drift_count"
  echo "  config_errors: $error_count"
} >> "$OUTPUT_PATH"

echo "Migration drift audit report written to: $OUTPUT_PATH"

if [[ "$FAIL_ON_DRIFT" == "1" ]] && (( drift_count > 0 || error_count > 0 )); then
  exit 1
fi
