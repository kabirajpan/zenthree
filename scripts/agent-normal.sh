#!/usr/bin/env bash
set -e

# Capture directory where user invoked the command
CALL_DIR="$(pwd)"

# Resolve repository root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Load .env if present
if [ -f "$ROOT_DIR/.env" ]; then
    export $(grep -v '^#' "$ROOT_DIR/.env" | xargs -d '\n')
fi

# Check if user explicitly provided a workspace argument
has_ws=false
for arg in "$@"; do
    if [ "$arg" = "-w" ] || [ "$arg" = "--workspace" ]; then
        has_ws=true
        break
    fi
done

cd "$ROOT_DIR"
if [ "$has_ws" = true ]; then
    cargo run -p agent --bin zene-agent -- "$@"
else
    cargo run -p agent --bin zene-agent -- --workspace "$CALL_DIR" "$@"
fi
