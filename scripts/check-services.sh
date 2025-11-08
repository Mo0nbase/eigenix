#!/usr/bin/env bash
# Service health check script for Eigenix deployment

set -e

echo "=== Eigenix Service Health Check ==="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

check_service() {
    local service=$1
    local description=$2

    if systemctl is-active --quiet "$service"; then
        echo -e "${GREEN}✓${NC} $description is running"
        return 0
    else
        echo -e "${RED}✗${NC} $description is not running"
        return 1
    fi
}

check_container() {
    local container=$1
    local description=$2

    if podman ps --format "{{.Names}}" | grep -q "^${container}$"; then
        echo -e "${GREEN}✓${NC} $description container is running"
        return 0
    else
        echo -e "${RED}✗${NC} $description container is not running"
        return 1
    fi
}

check_port() {
    local host=$1
    local port=$2
    local description=$3

    if timeout 2 bash -c "echo > /dev/tcp/$host/$port" 2>/dev/null; then
        echo -e "${GREEN}✓${NC} $description is reachable on $host:$port"
        return 0
    else
        echo -e "${RED}✗${NC} $description is not reachable on $host:$port"
        return 1
    fi
}

echo "--- Podman Network ---"
if podman network inspect eigenix-network &>/dev/null; then
    echo -e "${GREEN}✓${NC} eigenix-network exists"
else
    echo -e "${RED}✗${NC} eigenix-network does not exist"
fi
echo

echo "--- Core Services ---"
check_container "monerod" "Monero daemon"
check_container "bitcoind" "Bitcoin daemon"
check_container "electrs" "Electrs"
echo

echo "--- ASB Services ---"
check_container "asb" "ASB (Atomic Swap Backend)"
check_container "asb-controller" "ASB Controller"
check_container "asb-tracing-logger" "ASB Tracing Logger"
echo

echo "--- Backend Services ---"
if systemctl list-units --full --all | grep -q "podman-surrealdb.service"; then
    check_container "surrealdb" "SurrealDB"
fi

if systemctl list-units --full --all | grep -q "eigenix-backend.service"; then
    check_service "eigenix-backend.service" "Eigenix Backend API"
fi
echo

echo "--- Web Services ---"
if systemctl list-units --full --all | grep -q "eigenix-web.service"; then
    check_service "eigenix-web.service" "Eigenix Web Frontend"
fi
echo

echo "--- Port Connectivity (Internal) ---"
check_port "localhost" "8332" "Bitcoin RPC"
check_port "localhost" "18081" "Monero RPC"
check_port "localhost" "9944" "ASB RPC"

if systemctl list-units --full --all | grep -q "podman-surrealdb.service"; then
    check_port "localhost" "8001" "SurrealDB"
fi
echo

echo "--- Port Connectivity (External - should be accessible) ---"
check_port "localhost" "9939" "ASB P2P"

if systemctl list-units --full --all | grep -q "eigenix-web.service"; then
    check_port "localhost" "8080" "Eigenix Web"
fi
echo

echo "--- Container Logs (last 5 lines each) ---"
for container in monerod bitcoind electrs asb surrealdb; do
    if podman ps --format "{{.Names}}" | grep -q "^${container}$"; then
        echo -e "${YELLOW}[$container]${NC}"
        podman logs --tail 5 "$container" 2>&1 | sed 's/^/  /'
        echo
    fi
done

echo "=== Health Check Complete ==="
