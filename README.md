# BE_rust_store

## Run All Services (Bash)

From repo root:

```bash
./scripts/run-all.sh
```

This will:

- Start infra containers (`postgres`, `kafka`, `kafka-ui`, `redis`, `prometheus`, `grafana`)
- Ensure Kafka topic `domain-events` exists
- Run DB migrations sequentially once (`scripts/migrate-all.sh`)
- Start all Rust services in background
- Wait for all `/health` endpoints to return `200`

### Recommended Flags

- `RUN_DATABASE_MIGRATIONS=1` (default): run centralized migration phase before startup
- `RUN_MIGRATIONS_ON_STARTUP=0` (default): do not let each service re-run migrations at boot
- `CHECK_MIGRATION_CONFLICTS=1` (default): fail fast if migration versions conflict
- `RUN_DRIFT_AUDIT=1` (optional): run read-only drift audit before migration
- `STRICT_MIGRATIONS` (auto): defaults to `1` in `APP_ENV=prod|production|staging`, otherwise `0`

Example:

```bash
FORCE_RESTART_SERVICES=1 USE_CARGO_WATCH=0 RUN_DATABASE_MIGRATIONS=1 RUN_MIGRATIONS_ON_STARTUP=0 ./scripts/run-all.sh
```

CI/production-style strict migration run:

```bash
APP_ENV=production CHECK_MIGRATION_CONFLICTS=1 RUN_DRIFT_AUDIT=1 STRICT_MIGRATIONS=1 FAIL_ON_DRIFT=1 ./scripts/migrate-all.sh
```

Production startup flow:

```bash
APP_ENV=production FORCE_RESTART_SERVICES=1 USE_CARGO_WATCH=0 RUN_DATABASE_MIGRATIONS=1 RUN_MIGRATIONS_ON_STARTUP=0 ./scripts/run-all.sh
```

## Run Migrations Only

```bash
./scripts/migrate-all.sh
```

This runs migration files service-by-service in a fixed order to avoid advisory lock contention.

## Audit Migration Drift (Read-Only)

```bash
./scripts/audit-migration-drift.sh
```

This script does not modify the database. It checks each service migration source against DB migration history and writes a report to:

- `./logs/migration-drift-report.txt`

Strict CI-style audit (fail on any detected drift/config error):

```bash
FAIL_ON_DRIFT=1 ./scripts/audit-migration-drift.sh
```

Logs and PID files:

- Logs: `./logs/*.log`
- PIDs: `./.run/*.pid`

## Stop All Services (Bash)

```bash
./scripts/stop-all.sh
```

This will stop all Rust services by PID and stop the infra containers.
