#!/usr/bin/env bash
set -euo pipefail

BASE_URL="http://127.0.0.1:3030/autoscript"

# Cargo passes Windows paths with backslashes, which Bash may consume.
wasm="${1//\\/\/}"

# If the path was mangled before reaching this script, reconstruct it.
if [[ ! -f "$wasm" ]]; then
    wasm="target/wasm32-unknown-unknown/debug/auto-drive.wasm"
fi

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
    --show-error \
    --no-buffer \
    --request POST \
    --data-binary "@$wasm" \
    "$BASE_URL/run"

