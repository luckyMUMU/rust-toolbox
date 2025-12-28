#!/bin/bash

# Simple Docker tool that processes workflow parameters
# This script demonstrates how to create a Docker-based tool for the workflow toolkit

set -e

# Function to log messages
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >&2
}

# Function to output JSON result
output_json() {
    echo "$1"
}

# Function to output error and exit
error_exit() {
    output_json "{\"success\": false, \"error\": \"$1\"}"
    exit 1
}

log "Starting simple processor tool"

# Read parameters from environment variable
PARAMS="${WORKFLOW_PARAMS:-{}}"
log "Received parameters: $PARAMS"

# Parse parameters using jq
if ! echo "$PARAMS" | jq . > /dev/null 2>&1; then
    error_exit "Invalid JSON parameters"
fi

# Extract specific parameters
OPERATION=$(echo "$PARAMS" | jq -r '.operation // "echo"')
INPUT_DATA=$(echo "$PARAMS" | jq -r '.input // "Hello, World!"')
DELAY=$(echo "$PARAMS" | jq -r '.delay // 0' | sed 's/[^0-9]//g')

log "Operation: $OPERATION"
log "Input data: $INPUT_DATA"
log "Delay: ${DELAY}s"

# Add delay if specified
if [ "$DELAY" -gt 0 ]; then
    log "Waiting for ${DELAY} seconds..."
    sleep "$DELAY"
fi

# Process based on operation
case "$OPERATION" in
    "echo")
        RESULT="$INPUT_DATA"
        ;;
    "uppercase")
        RESULT=$(echo "$INPUT_DATA" | tr '[:lower:]' '[:upper:]')
        ;;
    "lowercase")
        RESULT=$(echo "$INPUT_DATA" | tr '[:upper:]' '[:lower:]')
        ;;
    "reverse")
        RESULT=$(echo "$INPUT_DATA" | rev)
        ;;
    "length")
        RESULT=$(echo -n "$INPUT_DATA" | wc -c)
        ;;
    "base64_encode")
        RESULT=$(echo -n "$INPUT_DATA" | base64)
        ;;
    "base64_decode")
        if ! RESULT=$(echo -n "$INPUT_DATA" | base64 -d 2>/dev/null); then
            error_exit "Invalid base64 input"
        fi
        ;;
    "json_pretty")
        if ! RESULT=$(echo "$INPUT_DATA" | jq . 2>/dev/null); then
            error_exit "Invalid JSON input"
        fi
        ;;
    "word_count")
        RESULT=$(echo "$INPUT_DATA" | wc -w)
        ;;
    "line_count")
        RESULT=$(echo "$INPUT_DATA" | wc -l)
        ;;
    *)
        error_exit "Unknown operation: $OPERATION"
        ;;
esac

log "Processing completed successfully"

# Output the result as JSON
output_json "{
    \"success\": true,
    \"result\": {
        \"operation\": \"$OPERATION\",
        \"input\": \"$INPUT_DATA\",
        \"output\": \"$RESULT\",
        \"processed_at\": \"$(date -Iseconds)\",
        \"container_hostname\": \"$(hostname)\",
        \"container_user\": \"$(whoami)\"
    },
    \"metadata\": {
        \"execution_time_seconds\": $DELAY,
        \"tool_version\": \"1.0.0\",
        \"container_os\": \"$(uname -s)\",
        \"container_arch\": \"$(uname -m)\"
    }
}"

log "Tool execution completed"