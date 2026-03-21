#!/bin/bash
# Watch & auto-restart all services
# Usage: bash watch-all.sh

export PATH="$HOME/protoc/bin:$PATH"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PIDS=()

cleanup() {
    echo -e "\n${YELLOW}Stopping all services...${NC}"
    for pid in "${PIDS[@]}"; do
        kill "$pid" 2>/dev/null
    done
    wait 2>/dev/null
    echo -e "${GREEN}All services stopped.${NC}"
    exit 0
}

trap cleanup SIGINT SIGTERM

echo -e "${GREEN}Starting all services in watch mode...${NC}"

# User service
echo -e "${YELLOW}[user-service]${NC} watching on port 3001"
cargo watch -w services/user-service/src -w shared -w proto \
    -x "run -p user-service" \
    --why 2>&1 | sed "s/^/[user-service] /" &
PIDS+=($!)

# Mail service
echo -e "${YELLOW}[mail-service]${NC} watching"
cargo watch -w services/mail-service/src -w shared -w proto \
    -x "run -p mail-service" \
    --why 2>&1 | sed "s/^/[mail-service] /" &
PIDS+=($!)

echo ""
echo -e "${GREEN}All services started. Press Ctrl+C to stop all.${NC}"
echo ""

wait
