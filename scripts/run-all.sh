#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PID_DIR="$ROOT/.run"
LOG_DIR="$ROOT/logs"
mkdir -p "$PID_DIR" "$LOG_DIR"

SERVICES=(
  "user-service:3001"
  "mail-service:3002"
  "product-service:3006"
  "comment-service:3010"
  "cart-service:3003"
  "checkout-service:3004"
  "order-service:3005"
  "inventory-service:3007"
  "notification-service:3008"
  "shipping-service:3009"
)

echo "[1/4] Starting infrastructure containers..."
docker compose up -d postgres kafka kafka-ui kafka-exporter prometheus grafana >/dev/null

if docker ps -a --format '{{.Names}}' | grep -qx 'be_store_redis'; then
  docker start be_store_redis >/dev/null 2>&1 || true
else
  docker run -d --name be_store_redis -p 6379:6379 redis:7-alpine >/dev/null
fi

echo "[2/4] Ensuring Kafka topic exists (domain-events)..."
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

start_service() {
  local name="$1"
  local pid_path="$PID_DIR/$name.pid"

  if [[ -f "$pid_path" ]]; then
    local existing_pid
    existing_pid="$(head -n 1 "$pid_path" || true)"
    if [[ -n "$existing_pid" ]] && kill -0 "$existing_pid" >/dev/null 2>&1; then
      echo "- $name already running (PID $existing_pid), skipping"
      return
    fi
    rm -f "$pid_path"
  fi

  nohup cargo run -p "$name" >"$LOG_DIR/$name.out.log" 2>"$LOG_DIR/$name.err.log" &
  local new_pid=$!
  echo "$new_pid" >"$pid_path"
  echo "- started $name (PID $new_pid)"
}

echo "[3/4] Starting Rust services..."
for svc in "${SERVICES[@]}"; do
  name="${svc%%:*}"
  start_service "$name"
done

wait_healthy() {
  local port="$1"
  local name="$2"
  local deadline=$((SECONDS + 90))

  while (( SECONDS < deadline )); do
    if curl -fsS "http://localhost:${port}/health" >/dev/null 2>&1; then
      echo "  OK  $name on :$port"
      return 0
    fi
    sleep 1
  done

  echo "  FAIL $name on :$port"
  return 1
}

echo "[4/4] Waiting for health endpoints..."
failed=0
for svc in "${SERVICES[@]}"; do
  name="${svc%%:*}"
  port="${svc##*:}"
  if ! wait_healthy "$port" "$name"; then
    failed=1
  fi
done

if [[ "$failed" -ne 0 ]]; then
  echo "One or more services did not become healthy. Check logs in ./logs/*.log"
  exit 1
fi

echo "All services are healthy."
echo "Logs: ./logs"
echo "PIDs: ./.run"
