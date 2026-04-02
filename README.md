# BE_rust_store

## Run All Services (Bash)

From repo root:

```bash
./scripts/run-all.sh
```

This will:

- Start infra containers (`postgres`, `kafka`, `kafka-ui`, `redis`, `prometheus`, `grafana`)
- Ensure Kafka topic `domain-events` exists
- Start all Rust services in background
- Wait for all `/health` endpoints to return `200`

Logs and PID files:

- Logs: `./logs/*.log`
- PIDs: `./.run/*.pid`

## Stop All Services (Bash)

```bash
./scripts/stop-all.sh
```

This will stop all Rust services by PID and stop the infra containers.
