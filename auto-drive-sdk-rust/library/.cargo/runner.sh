#!/usr/bin/env bash
set -euo pipefail

BASE_URL="https://cloud-autodrive.hugostechlab.com/autoscript"

# --- Authentication Requirement ---
# Look for .env in current directory, then script's directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE=""
if [[ -f ".env" ]]; then
    ENV_FILE=".env"
elif [[ -f "$SCRIPT_DIR/.env" ]]; then
    ENV_FILE="$SCRIPT_DIR/.env"
fi

AUTH_TOKEN=""
if [[ -n "$ENV_FILE" ]]; then
    # Safely extract AUTH_TOKEN (ignoring comments, stripping carriage returns and quotes)
    AUTH_TOKEN=$(grep -E '^AUTH_TOKEN=' "$ENV_FILE" | cut -d '=' -f 2- | tr -d '\r' | sed -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//")
fi

# Halt execution and show instructions if token is missing
if [[ -z "$AUTH_TOKEN" ]]; then
    echo -e "\033[31m[ERROR]\033[0m Authentication token could not be loaded." >&2
    echo "" >&2
    echo "Please provide a valid .env file to authenticate." >&2
    echo "----------------------------------------------------" >&2
    echo "1. Create a file named exactly '.env' in one of these locations:" >&2
    echo "   - Current directory : $(pwd)" >&2
    echo "   - Script directory  : $SCRIPT_DIR" >&2
    echo "" >&2
    echo "2. Add your token to the file like this:" >&2
    echo "   AUTH_TOKEN=your_secure_token_here" >&2
    echo "----------------------------------------------------" >&2
    echo "Exiting." >&2
    exit 1
fi

# Token found; set up the header array
AUTH_ARGS=("--header" "Authorization: Bearer $AUTH_TOKEN")
echo "Authentication token loaded successfully."
# ---------------------------------

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
        
        curl --fail \
            --show-error \
            --request POST \
            ${AUTH_ARGS[@]+"${AUTH_ARGS[@]}"} \
            --header "Content-Type: application/json" \
            --data "{\"run_uuid\":\"$RUN_UUID\"}" \
            "$BASE_URL/cancel" \
            || echo "Warning: cancel request failed"
    else
        echo "No run_uuid received yet, sending general cancel."
        curl --fail \
            --show-error \
            --request POST \
            ${AUTH_ARGS[@]+"${AUTH_ARGS[@]}"} \
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
    ${AUTH_ARGS[@]+"${AUTH_ARGS[@]}"} \
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
        !/^:/ && NF > 0 {
            print "\033[31m[SERVER ERROR]\033[0m " $0
            fflush()
        }
    '