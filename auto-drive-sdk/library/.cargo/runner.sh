#!/usr/bin/env bash
set -euo pipefail

# TODO: maybe make this script cross platform? Rust executable?
# .local names don't seem to be working in wsl
# the-beginner.local
# 192.168.0.116

BASE_URL="http://192.168.0.116/autoscript"

wasm="$1"

echo "Uploading: $wasm"

curl --fail \
    --show-error \
    --data-binary "@$wasm" \
    "$BASE_URL/upload"

echo
echo "Starting autoscript..."

cleanup() {
    echo
    echo "Cancelling."
    curl --fail \
        --show-error \
        --request POST \
        "$BASE_URL/cancel" \
        || echo "Warning: cancel request failed"
    echo

    # The curl process is terminated automatically when the script exits.
    exit 130
}

trap cleanup INT TERM

curl --fail \
    --show-error \
    --no-buffer \
    --request POST \
    "$BASE_URL/run"

