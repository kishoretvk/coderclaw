#!/bin/bash
# TitanClaw Health Check Script
#
# This script is used by Docker HEALTHCHECK to verify
# that the TitanClaw application is running correctly.

set -e

# Check if the process is running
if ! pgrep -x "titanclaw" > /dev/null; then
    echo "TitanClaw process not found"
    exit 1
fi

# Check if the HTTP endpoint is responding
if command -v curl > /dev/null 2>&1; then
    if curl -sf http://localhost:3000/health > /dev/null 2>&1; then
        echo "TitanClaw is healthy"
        exit 0
    else
        echo "TitanClaw HTTP endpoint not responding"
        exit 1
    fi
else
    # Fallback: just check if process is running
    echo "curl not available, checking process only"
    pgrep -x "titanclaw" > /dev/null && exit 0 || exit 1
fi
