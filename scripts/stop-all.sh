#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PID_DIR="$ROOT/.run"
SERVICES=(
  "user-service"
  "mail-service"
  "product-service"
  "upload-service"
  "comment-service"
  "cart-service"
  "checkout-service"
  "order-service"
  "inventory-service"
  "notification-service"
  "shipping-service"
)

if [[ ! -d "$PID_DIR" ]]; then
  echo "No PID directory found. Nothing to stop."
else
  echo "Stopping Rust services..."
  for name in "${SERVICES[@]}"; do
    pid_path="$PID_DIR/$name.pid"
    if [[ ! -f "$pid_path" ]]; then
      echo "- $name: no PID file"
      continue
    fi

    proc_id="$(head -n 1 "$pid_path" || true)"
    if [[ -z "$proc_id" ]]; then
      echo "- $name: invalid PID file"
      rm -f "$pid_path"
      continue
    fi

    if kill -0 "$proc_id" >/dev/null 2>&1; then
      kill "$proc_id" >/dev/null 2>&1 || true
      sleep 0.3
      kill -9 "$proc_id" >/dev/null 2>&1 || true
      echo "- stopped $name (PID $proc_id)"
    else
      echo "- $name: process $proc_id not found"
    fi

    rm -f "$pid_path"
  done
fi

echo "Stopping infrastructure containers..."
docker compose stop postgres kafka kafka-ui redis kafka-exporter prometheus grafana >/dev/null 2>&1 || true
docker stop be_store_redis >/dev/null 2>&1 || true

echo "Done."
