#!/usr/bin/env bash
# API endpoint testing script for Eigenix Backend
# Tests all endpoints to ensure they are responding correctly

set -e

# Configuration
API_HOST="${API_HOST:-127.0.0.1}"
API_PORT="${API_PORT:-3000}"
BASE_URL="http://${API_HOST}:${API_PORT}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TOTAL=0
PASSED=0
FAILED=0

echo "=== Eigenix Backend API Endpoint Tests ==="
echo "Testing API at: ${BASE_URL}"
echo

test_endpoint() {
    local method=$1
    local path=$2
    local description=$3
    local expected_status=${4:-200}
    
    TOTAL=$((TOTAL + 1))
    
    echo -n "Testing ${description}... "
    
    # Make request and capture status code and response
    response=$(curl -s -w "\n%{http_code}" -X "${method}" "${BASE_URL}${path}" 2>/dev/null || echo "000")
    http_code=$(echo "$response" | tail -n 1)
    body=$(echo "$response" | head -n -1)
    
    if [ "$http_code" = "$expected_status" ]; then
        echo -e "${GREEN}✓${NC} (HTTP ${http_code})"
        PASSED=$((PASSED + 1))
        
        # Show response preview if available
        if [ -n "$body" ] && [ "$body" != "null" ]; then
            echo "$body" | jq -C '.' 2>/dev/null | head -n 5 | sed 's/^/  /' || echo "  $body" | head -n 5
        fi
    else
        echo -e "${RED}✗${NC} (HTTP ${http_code}, expected ${expected_status})"
        FAILED=$((FAILED + 1))
        
        # Show error details
        if [ -n "$body" ]; then
            echo -e "  ${YELLOW}Response:${NC} $body" | head -n 3
        fi
    fi
    echo
}

# Check if API is reachable
echo -e "${BLUE}→${NC} Checking API availability..."
if ! curl -s --connect-timeout 5 "${BASE_URL}/health" > /dev/null 2>&1; then
    echo -e "${RED}✗${NC} API is not reachable at ${BASE_URL}"
    echo "Make sure the eigenix-backend service is running:"
    echo "  sudo systemctl status eigenix-backend"
    exit 1
fi
echo -e "${GREEN}✓${NC} API is reachable"
echo

# Health Endpoint
echo -e "${BLUE}=== Health Check ===${NC}"
test_endpoint "GET" "/health" "Health endpoint"
echo

# Wallet Endpoints
echo -e "${BLUE}=== Wallet Endpoints ===${NC}"
test_endpoint "GET" "/wallets/balances" "Combined wallet balances"
test_endpoint "GET" "/wallets/health" "Combined wallet health"
test_endpoint "GET" "/wallets/bitcoin/balance" "Bitcoin balance"
test_endpoint "GET" "/wallets/bitcoin/health" "Bitcoin health"
test_endpoint "GET" "/wallets/monero/balance" "Monero balance"
test_endpoint "GET" "/wallets/monero/health" "Monero health"
echo

# Note: POST endpoint test (won't actually execute)
echo -e "${YELLOW}Note:${NC} Skipping POST /wallets/monero/refresh (would modify state)"
echo

# Metrics Summary
echo -e "${BLUE}=== Metrics Endpoints ===${NC}"
test_endpoint "GET" "/metrics/summary" "Metrics summary"
echo

# Bitcoin Metrics
echo -e "${BLUE}=== Bitcoin Metrics ===${NC}"
test_endpoint "GET" "/metrics/bitcoin" "Latest Bitcoin metrics"
test_endpoint "GET" "/metrics/bitcoin/history" "Bitcoin metrics history (24h default)"
test_endpoint "GET" "/metrics/bitcoin/history?from=2024-01-01T00:00:00Z&to=2024-12-31T23:59:59Z" "Bitcoin metrics history (custom range)"
test_endpoint "GET" "/metrics/bitcoin/interval?minutes=5" "Bitcoin metrics interval (5 min)"
test_endpoint "GET" "/metrics/bitcoin/interval?minutes=60" "Bitcoin metrics interval (60 min)"
echo

# Monero Metrics
echo -e "${BLUE}=== Monero Metrics ===${NC}"
test_endpoint "GET" "/metrics/monero" "Latest Monero metrics"
test_endpoint "GET" "/metrics/monero/history" "Monero metrics history (24h default)"
test_endpoint "GET" "/metrics/monero/history?from=2024-01-01T00:00:00Z&to=2024-12-31T23:59:59Z" "Monero metrics history (custom range)"
test_endpoint "GET" "/metrics/monero/interval?minutes=5" "Monero metrics interval (5 min)"
test_endpoint "GET" "/metrics/monero/interval?minutes=60" "Monero metrics interval (60 min)"
echo

# ASB Metrics
echo -e "${BLUE}=== ASB Metrics ===${NC}"
test_endpoint "GET" "/metrics/asb" "Latest ASB metrics"
test_endpoint "GET" "/metrics/asb/history" "ASB metrics history (24h default)"
test_endpoint "GET" "/metrics/asb/history?from=2024-01-01T00:00:00Z&to=2024-12-31T23:59:59Z" "ASB metrics history (custom range)"
test_endpoint "GET" "/metrics/asb/interval?minutes=5" "ASB metrics interval (5 min)"
test_endpoint "GET" "/metrics/asb/interval?minutes=60" "ASB metrics interval (60 min)"
echo

# Electrs Metrics
echo -e "${BLUE}=== Electrs Metrics ===${NC}"
test_endpoint "GET" "/metrics/electrs" "Latest Electrs metrics"
test_endpoint "GET" "/metrics/electrs/history" "Electrs metrics history (24h default)"
test_endpoint "GET" "/metrics/electrs/history?from=2024-01-01T00:00:00Z&to=2024-12-31T23:59:59Z" "Electrs metrics history (custom range)"
test_endpoint "GET" "/metrics/electrs/interval?minutes=5" "Electrs metrics interval (5 min)"
test_endpoint "GET" "/metrics/electrs/interval?minutes=60" "Electrs metrics interval (60 min)"
echo

# Container Metrics
echo -e "${BLUE}=== Container Metrics ===${NC}"
test_endpoint "GET" "/metrics/containers" "Latest container metrics"
test_endpoint "GET" "/metrics/containers/history?name=bitcoind" "Container history (bitcoind)"
test_endpoint "GET" "/metrics/containers/history?name=monerod" "Container history (monerod)"
test_endpoint "GET" "/metrics/containers/history?name=asb" "Container history (asb)"
echo

# Summary
echo "=== Test Summary ==="
echo -e "Total tests: ${TOTAL}"
echo -e "${GREEN}Passed: ${PASSED}${NC}"
echo -e "${RED}Failed: ${FAILED}${NC}"
echo

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi

