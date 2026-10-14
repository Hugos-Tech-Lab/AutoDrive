#!/usr/bin/env bash
set -euo pipefail

BASE_URL="http://127.0.0.1:3030/autoscript"

# On windows we cannot rely on the $1 variable from Cargo because the slashes (i.e. / or \) are completely disappeared."
# Use this hack to get the most recently changed .wasm file to pass to the AutoDrive servers.
wasm=$(ls -t target/wasm32-unknown-unknown/{release,debug}/*.wasm 2>/dev/null | head -n 1)

if [[ ! -f "$wasm" ]]; then
    echo "WASM file not found: $wasm" >&2
    exit 1
fi

echo "Running: $wasm"
echo "Starting autoscript..."

cleanup() {
    trap - INT TERM
    echo
    echo "Cancelling."
    curl --fail \
        --show-error \
        --request POST \
        "$BASE_URL/cancel" \
        || echo "Warning: cancel request failed"
    echo
    exit 130
}

trap cleanup INT TERM

curl --fail \
    --silent \
    --show-error \
    --no-buffer \
    --request POST \
    --data-binary "@$wasm" \
    "$BASE_URL/run" |
    sed '/^[[:space:]]*$/d'
