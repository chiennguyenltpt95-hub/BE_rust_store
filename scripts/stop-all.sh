#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PID_DIR="$ROOT/.run"

is_windows() {
  case "$(uname -s 2>/dev/null || echo)" in
    MINGW*|MSYS*|CYGWIN*|Windows_NT) return 0 ;;
    *) return 1 ;;
  esac
}

is_pid_running_windows() {
  local pid="$1"
  tasklist //FI "PID eq ${pid}" 2>/dev/null | grep -qE "[[:space:]]${pid}[[:space:]]"
}
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

    if is_windows; then
      if is_pid_running_windows "$proc_id"; then
        taskkill //PID "$proc_id" //T //F >/dev/null 2>&1 || true
        echo "- stopped $name (PID $proc_id)"
      else
        echo "- $name: process $proc_id not found"
      fi
    else
      if kill -0 "$proc_id" >/dev/null 2>&1; then
        kill "$proc_id" >/dev/null 2>&1 || true
        sleep 0.3
        kill -9 "$proc_id" >/dev/null 2>&1 || true
        echo "- stopped $name (PID $proc_id)"
      else
        echo "- $name: process $proc_id not found"
      fi
    fi

    rm -f "$pid_path"
  done
fi

if is_windows; then
  echo "Cleaning orphan Rust service processes..."
  for name in "${SERVICES[@]}"; do
    taskkill //IM "${name}.exe" //T //F >/dev/null 2>&1 || true
  done
fi

echo "Stopping infrastructure containers..."
docker compose stop postgres kafka kafka-ui redis kafka-exporter prometheus grafana >/dev/null 2>&1 || true
docker stop be_store_redis >/dev/null 2>&1 || true

echo "Done."
