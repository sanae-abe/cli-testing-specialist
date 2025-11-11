#!/bin/bash
# Memory profiling script for cli-testing-specialist

set -euo pipefail

echo "========================================="
echo "Memory Profiling - CLI Testing Specialist"
echo "========================================="
echo ""

# Build in release mode
echo "[1/4] Building release binary..."
cargo build --release --quiet

BINARY="./target/release/cli-testing-specialist"

# Function to run and measure memory
measure_memory() {
    local test_name="$1"
    local cmd="$2"

    echo "Testing: $test_name"

    if command -v /usr/bin/time &> /dev/null; then
        # macOS/BSD time command
        /usr/bin/time -l sh -c "$cmd" 2>&1 | grep -E "maximum resident set size|real" || true
    else
        # GNU time command
        /usr/bin/time -v sh -c "$cmd" 2>&1 | grep -E "Maximum resident set size|Elapsed" || true
    fi

    echo ""
}

echo "[2/4] Testing small CLI (curl)..."
measure_memory "curl analysis" "$BINARY analyze /usr/bin/curl --output /tmp/curl-analysis.json"

echo "[3/4] Testing medium CLI (npm)..."
if [ -f "$HOME/.nvm/versions/node/v25.0.0/bin/npm" ]; then
    measure_memory "npm analysis" "$BINARY analyze $HOME/.nvm/versions/node/v25.0.0/bin/npm --output /tmp/npm-analysis.json"
else
    echo "npm not found at expected path, skipping..."
fi

echo "[4/4] Testing binary size..."
ls -lh "$BINARY" | awk '{print "Binary size: " $5}'

echo ""
echo "========================================="
echo "Memory Profile Complete"
echo "========================================="
echo ""
echo "Target: <50MB for typical workloads"
echo "Check results above for actual usage"
