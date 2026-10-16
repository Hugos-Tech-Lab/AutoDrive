#!/usr/bin/env bash
set -euo pipefail

BASE_URL="http://127.0.0.1:3030/autoscript"

# Cargo passes the executable path as $1. Shift it out so $@ contains only user arguments.
if [[ $# -gt 0 ]]; then
    wasm_arg="$1"
    shift
fi

# Capture all remaining arguments into a single string (preserving spaces)
ARGS_STRING="$*"

# Temporary file to share the run_uuid between AWK and cleanup()
UUID_FILE=$(mktemp)
trap 'rm -f "$UUID_FILE"' EXIT

# Normalize backslashes to forward slashes if passed from Cargo ($1), otherwise find latest .wasm file
if [[ -n "${wasm_arg:-}" ]] && [[ -f "${wasm_arg//\\//}" ]]; then
    wasm="${wasm_arg//\\//}"
else
    # Find most recently modified .wasm file without relying on bash brace expansion
    wasm=$(find target/wasm32-unknown-unknown/release target/wasm32-unknown-unknown/debug -name "*.wasm" -type f -printf "%T@ %p\n" 2>/dev/null | sort -nr | head -n 1 | cut -d' ' -f2-)
fi

if [[ -z "$wasm" || ! -f "$wasm" ]]; then
    echo "WASM file not found: ${wasm:-none}" >&2
    exit 1
fi

echo "Running: $wasm"
if [[ -n "$ARGS_STRING" ]]; then
    echo "Arguments passed: $ARGS_STRING"
fi
echo "Sending run wasm request to the cloud..."

cleanup() {
    trap - INT TERM
    echo
    echo "Cancelling."

    if [[ -s "$UUID_FILE" ]]; then
        RUN_UUID=$(cat "$UUID_FILE")
        echo "Sending cancel for run: $RUN_UUID"
        
        # Adjust payload format below based on how your /cancel endpoint expects run_uuid
        curl --fail \
            --show-error \
            --request POST \
            --header "Content-Type: application/json" \
            --data "{\"run_uuid\":\"$RUN_UUID\"}" \
            "$BASE_URL/cancel" \
            || echo "Warning: cancel request failed"
    else
        echo "No run_uuid received yet, sending general cancel."
        curl --fail \
            --show-error \
            --request POST \
            "$BASE_URL/cancel" \
            || echo "Warning: cancel request failed"
    fi

    echo
    exit 130
}

trap cleanup INT TERM

# Using multipart form-data to send both binary wasm file and string args
curl --fail-with-body \
    --silent \
    --show-error \
    --no-buffer \
    --request POST \
    -F "file=@$wasm;type=application/wasm" \
    -F "args=$ARGS_STRING" \
    "$BASE_URL/run" |
    awk -v uuid_file="$UUID_FILE" '
        # Ignore the SSE "event:" framing lines
        /^event:/ {
            next
        }

        /^data:/ {
            sub(/^data:[[:space:]]?/, "")
            line = $0

            # Parse and save run_uuid if present in stream line
            if (line ~ /run_uuid/) {
                # Case 1: JSON formatted key (e.g., "run_uuid": "abc-123")
                if (match(line, /"run_uuid"[[:space:]]*:[[:space:]]*"[^"]+"/)) {
                    str = substr(line, RSTART, RLENGTH)
                    sub(/^"run_uuid"[[:space:]]*:[[:space:]]*"/, "", str)
                    sub(/"$/, "", str)
                    print str > uuid_file
                    close(uuid_file)
                }
            }

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