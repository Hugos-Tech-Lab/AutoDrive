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
echo "Sending run wasm request to the..."

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

curl --fail-with-body \
    --silent \
    --show-error \
    --no-buffer \
    --request POST \
    --data-binary "@$wasm" \
    "$BASE_URL/run" |
    awk '
        # Ignore the SSE "event:" framing lines
        /^event:/ {
            next
        }

        /^data:/ {
            sub(/^data:[[:space:]]?/, "")
            line = $0

            reset = "\033[0m"
            red = "\033[31m"
            yellow = "\033[33m"
            green = "\033[32m"
            blue = "\033[34m"
            gray = "\033[90m"

            if (line ~ /^\[ERROR\]/) {
                color = red
            } else if (line ~ /^\[WARN\]/) {
                color = yellow
            } else if (line ~ /^\[INFO\]/) {
                color = green
            } else if (line ~ /^\[DEBUG\]/) {
                color = gray
            } else {
                color = reset
            }

            if (match(line, /^\[[A-Z]+\]/)) {
                type = substr(line, 1, RLENGTH)
                rest = substr(line, RLENGTH + 1)
                print color type reset rest
            } else {
                print line
            }

            fflush()
            next
        }
        
        # Catch non-SSE plain text responses (like your 413 error message)
        # We already skipped "event:" and "data:". Skip comments (":") and empty lines.
        !/^:/ && NF > 0 {
            print "\033[31m[SERVER ERROR]\033[0m " $0
            fflush()
        }
    '