#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PID_DIR="$ROOT/.run"
LOG_DIR="$ROOT/logs"
mkdir -p "$PID_DIR" "$LOG_DIR"

LOCAL_DATABASE_URL="${LOCAL_DATABASE_URL:-postgresql://db-jayden.c1gymyg48fuo.ap-southeast-1.rds.amazonaws.com:5432/postgres?user=jayden&password=db-jayden}"
USE_LOCAL_DB="${USE_LOCAL_DB:-0}"
HEALTH_TIMEOUT_SECONDS="${HEALTH_TIMEOUT_SECONDS:-180}"
HEALTH_CHECK_INTERVAL_SECONDS="${HEALTH_CHECK_INTERVAL_SECONDS:-1}"
HEALTH_REQUEST_TIMEOUT_SECONDS="${HEALTH_REQUEST_TIMEOUT_SECONDS:-2}"
SERVICE_STARTUP_TIMEOUT_SECONDS="${SERVICE_STARTUP_TIMEOUT_SECONDS:-120}"
APP_ENV="${APP_ENV:-dev}"
USE_CARGO_WATCH="${USE_CARGO_WATCH:-0}"
WATCH_SERVICES="${WATCH_SERVICES:-}"
FORCE_RESTART_SERVICES="${FORCE_RESTART_SERVICES:-0}"
RUN_MIGRATIONS_ON_STARTUP="${RUN_MIGRATIONS_ON_STARTUP:-0}"
RUN_DATABASE_MIGRATIONS="${RUN_DATABASE_MIGRATIONS:-1}"

SERVICES=(
  "user-service:3001"
  "mail-service:3002"
  "product-service:3006"
  "upload-service:3011"
  "comment-service:3010"
  "cart-service:3003"
  "checkout-service:3004"
  "order-service:3005"
  "inventory-service:3007"
  "notification-service:3008"
  "shipping-service:3009"
)

if [[ "$USE_LOCAL_DB" == "1" ]]; then
  DB_MODE="local-postgres"
else
  DB_MODE="rds-from-service-env"
fi

echo "[1/6] Starting infrastructure containers (DB mode: $DB_MODE)..."

INFRA_SERVICES=(kafka kafka-ui kafka-exporter prometheus grafana)
if [[ "$USE_LOCAL_DB" == "1" ]]; then
  INFRA_SERVICES=(postgres "${INFRA_SERVICES[@]}")
fi

docker compose up -d "${INFRA_SERVICES[@]}" >/dev/null

if docker ps -a --format '{{.Names}}' | grep -qx 'be_store_redis'; then
  docker start be_store_redis >/dev/null 2>&1 || true
else
  docker run -d --name be_store_redis -p 6379:6379 redis:7-alpine >/dev/null
fi

echo "[2/6] Ensuring Kafka topic exists (domain-events)..."
kafka_ready=0
for _ in $(seq 1 60); do
  if MSYS_NO_PATHCONV=1 docker exec be_store_kafka /opt/kafka/bin/kafka-topics.sh --list --bootstrap-server localhost:9092 >/dev/null 2>&1; then
    kafka_ready=1
    break
  fi
  sleep 1
done

if [[ "$kafka_ready" -ne 1 ]]; then
  echo "Kafka is not ready after 60 seconds"
  exit 1
fi

MSYS_NO_PATHCONV=1 docker exec be_store_kafka /opt/kafka/bin/kafka-topics.sh \
  --create --if-not-exists \
  --topic domain-events \
  --bootstrap-server localhost:9092 \
  --partitions 1 \
  --replication-factor 1 >/dev/null 2>&1 || true

is_healthy() {
  local port="$1"
  curl -fsS --connect-timeout "$HEALTH_REQUEST_TIMEOUT_SECONDS" --max-time "$HEALTH_REQUEST_TIMEOUT_SECONDS" "http://localhost:${port}/health" >/dev/null 2>&1
}

is_pid_running_windows() {
  local pid="$1"
  if ! [[ "$pid" =~ ^[0-9]+$ ]]; then
    return 1
  fi

  powershell.exe -NoProfile -Command "if (Get-Process -Id ${pid} -ErrorAction SilentlyContinue) { exit 0 } else { exit 1 }" >/dev/null 2>&1
}

has_cargo_watch() {
  cargo watch --version >/dev/null 2>&1
}

is_service_in_watch_list() {
  local name="$1"
  [[ -n "$WATCH_SERVICES" ]] || return 1

  case ",${WATCH_SERVICES}," in
    *",${name},"*) return 0 ;;
    *) return 1 ;;
  esac
}

should_use_watch_for_service() {
  local name="$1"

  if [[ "$USE_CARGO_WATCH" != "1" ]]; then
    return 1
  fi

  if [[ "$CARGO_WATCH_AVAILABLE" != "1" ]]; then
    return 1
  fi

  is_service_in_watch_list "$name"
}

CARGO_WATCH_IGNORE_ARGS=(
  --ignore logs
  --ignore .run
  --ignore target
)

service_binary_path() {
  local name="$1"

  if [[ -f "$ROOT/target/x86_64-pc-windows-msvc/debug/${name}.exe" ]]; then
    echo "$ROOT/target/x86_64-pc-windows-msvc/debug/${name}.exe"
    return
  fi

  if [[ -f "$ROOT/target/debug/${name}" ]]; then
    echo "$ROOT/target/debug/${name}"
    return
  fi

  echo "$ROOT/target/debug/${name}.exe"
}

kill_pid_tree_windows() {
  local pid="$1"
  taskkill //PID "$pid" //T //F >/dev/null 2>&1 || true
}

kill_service_executables_windows() {
  local name="$1"

  # Kill orphan binaries that may still hold DB advisory locks even when ports are free.
  taskkill //IM "${name}.exe" //T //F >/dev/null 2>&1 || true
}

free_port_if_occupied() {
  local port="$1"
  local pids
  pids="$(netstat -ano 2>/dev/null | awk -v needle=":${port}" '$2 ~ needle && $4 == "LISTENING" { print $5 }' | sort -u)"

  if [[ -z "$pids" ]]; then
    return
  fi

  while IFS= read -r pid; do
    if [[ -n "$pid" ]] && [[ "$pid" =~ ^[0-9]+$ ]] && [[ "$pid" != "0" ]]; then
      echo "  freeing port :$port (PID $pid)"
      kill_pid_tree_windows "$pid"
    fi
  done <<< "$pids"
}

start_service() {
  local name="$1"
  local port="$2"
  local pid_path="$PID_DIR/$name.pid"

  if [[ "$FORCE_RESTART_SERVICES" == "1" ]]; then
    if is_healthy "$port"; then
      echo "- $name healthy on :$port but force restart is enabled"
    fi
    kill_service_executables_windows "$name"
    free_port_if_occupied "$port"
    rm -f "$pid_path"
  fi

  if is_healthy "$port"; then
    echo "- $name already healthy on :$port, skipping"
    return
  fi

  if [[ -f "$pid_path" ]]; then
    local existing_pid
    existing_pid="$(head -n 1 "$pid_path" || true)"
    if [[ -n "$existing_pid" ]] && [[ "$existing_pid" =~ ^[0-9]+$ ]] && is_pid_running_windows "$existing_pid"; then
      echo "- $name has stale/unhealthy PID $existing_pid, restarting"
      kill_pid_tree_windows "$existing_pid"
    fi
    kill_service_executables_windows "$name"
    rm -f "$pid_path"
  fi

  free_port_if_occupied "$port"

  if should_use_watch_for_service "$name"; then
    if [[ "$USE_LOCAL_DB" == "1" ]]; then
      DATABASE_URL="$LOCAL_DATABASE_URL" RUN_MIGRATIONS_ON_STARTUP="$RUN_MIGRATIONS_ON_STARTUP" nohup cargo watch "${CARGO_WATCH_IGNORE_ARGS[@]}" -q -x "run -p $name" >"$LOG_DIR/$name.out.log" 2>"$LOG_DIR/$name.err.log" &
    else
      RUN_MIGRATIONS_ON_STARTUP="$RUN_MIGRATIONS_ON_STARTUP" nohup cargo watch "${CARGO_WATCH_IGNORE_ARGS[@]}" -q -x "run -p $name" >"$LOG_DIR/$name.out.log" 2>"$LOG_DIR/$name.err.log" &
    fi

    local watch_pid=$!
    echo "$watch_pid" >"$pid_path"
    echo "- started $name in watch mode (PID $watch_pid)"
    return
  fi

  local bin_path
  bin_path="$(service_binary_path "$name")"

  if [[ ! -f "$bin_path" ]]; then
    echo "- $name binary not found at $bin_path"
    return 1
  fi

  if [[ "$USE_LOCAL_DB" == "1" ]]; then
    DATABASE_URL="$LOCAL_DATABASE_URL" RUN_MIGRATIONS_ON_STARTUP="$RUN_MIGRATIONS_ON_STARTUP" nohup "$bin_path" >"$LOG_DIR/$name.out.log" 2>"$LOG_DIR/$name.err.log" &
  else
    RUN_MIGRATIONS_ON_STARTUP="$RUN_MIGRATIONS_ON_STARTUP" nohup "$bin_path" >"$LOG_DIR/$name.out.log" 2>"$LOG_DIR/$name.err.log" &
  fi
  local new_pid=$!
  echo "$new_pid" >"$pid_path"
  echo "- started $name (PID $new_pid)"
  return 0
}

if [[ "$RUN_DATABASE_MIGRATIONS" == "1" ]]; then
  echo "[3/6] Running database migrations..."
  APP_ENV="$APP_ENV" \
  USE_LOCAL_DB="$USE_LOCAL_DB" \
  LOCAL_DATABASE_URL="$LOCAL_DATABASE_URL" \
  "$ROOT/scripts/migrate-all.sh"
else
  echo "[3/6] Skipping centralized database migrations (RUN_DATABASE_MIGRATIONS=0)"
fi

echo "[4/6] Preparing Rust services..."
CARGO_WATCH_AVAILABLE="0"
if has_cargo_watch; then
  CARGO_WATCH_AVAILABLE="1"
fi

if [[ "$USE_CARGO_WATCH" == "1" ]]; then
  if [[ "$CARGO_WATCH_AVAILABLE" != "1" ]]; then
    echo "- cargo-watch not found, watch mode disabled"
    USE_CARGO_WATCH="0"
  elif [[ -z "$WATCH_SERVICES" ]]; then
    echo "- USE_CARGO_WATCH=1 but WATCH_SERVICES is empty, watch mode disabled"
    USE_CARGO_WATCH="0"
  else
    echo "- watch mode enabled for: $WATCH_SERVICES"
  fi
fi

for svc in "${SERVICES[@]}"; do
  name="${svc%%:*}"

  if should_use_watch_for_service "$name"; then
    echo "- $name will run in watch mode, skipping pre-build"
    continue
  fi

  if [[ "$USE_LOCAL_DB" == "1" ]]; then
    DATABASE_URL="$LOCAL_DATABASE_URL" cargo build -p "$name" >/dev/null
  else
    cargo build -p "$name" >/dev/null
  fi
  echo "- built $name"
done

echo "[5/6] Starting Rust services..."
if [[ "$FORCE_RESTART_SERVICES" == "1" ]]; then
  echo "- force restart enabled; existing healthy services will be restarted"
fi

wait_healthy() {
  local port="$1"
  local name="$2"
  local timeout_seconds="${3:-$HEALTH_TIMEOUT_SECONDS}"
  local started_at="$SECONDS"
  local deadline=$((started_at + timeout_seconds))
  local next_report=$((started_at + 15))

  while (( SECONDS < deadline )); do
    if is_healthy "$port"; then
      echo "  OK  $name on :$port"
      return 0
    fi

    if (( SECONDS >= next_report )); then
      echo "  ...waiting $name on :$port ($((SECONDS - started_at))s elapsed)"
      next_report=$((SECONDS + 15))
    fi

    sleep "$HEALTH_CHECK_INTERVAL_SECONDS"
  done

  echo "  FAIL $name on :$port"
  if [[ -f "$LOG_DIR/$name.err.log" ]]; then
    echo "  last logs ($name.err.log):"
    tail -n 12 "$LOG_DIR/$name.err.log" || true
  fi
  if [[ -f "$LOG_DIR/$name.out.log" ]]; then
    echo "  last logs ($name.out.log):"
    tail -n 12 "$LOG_DIR/$name.out.log" || true
  fi
  return 1
}

echo "[6/6] Starting services sequentially and waiting for health..."
startup_failed=0
for svc in "${SERVICES[@]}"; do
  name="${svc%%:*}"
  port="${svc##*:}"

  if ! start_service "$name" "$port"; then
    startup_failed=1
    continue
  fi

  if ! wait_healthy "$port" "$name" "$SERVICE_STARTUP_TIMEOUT_SECONDS"; then
    startup_failed=1
  fi
done

echo "Final health sweep..."
failed=0
for svc in "${SERVICES[@]}"; do
  name="${svc%%:*}"
  port="${svc##*:}"
  if is_healthy "$port"; then
    echo "  OK  $name on :$port"
  else
    echo "  FAIL $name on :$port"
    failed=1
  fi
done

if [[ "$failed" -ne 0 ]]; then
  echo "One or more services did not become healthy. Check logs in ./logs/*.log"
  if [[ "$startup_failed" -ne 0 ]]; then
    echo "Note: Some services were slow during startup but recovered later."
  fi
  exit 1
fi

echo "All services are healthy."
echo "Logs: ./logs"
echo "PIDs: ./.run"
