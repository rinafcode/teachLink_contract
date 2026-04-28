#!/bin/bash

# Code Duplication Monitoring Script
# Uses jscpd to detect code duplication

set -e

REPORT_DIR="reports/duplication"
mkdir -p "$REPORT_DIR"

echo "🔍 Checking for code duplication..."

# Check if jscpd is installed, if not, try to run it via npx
if command -v jscpd &> /dev/null; then
    jscpd . --config .jscpd.json
else
    echo "📦 jscpd not found, using npx..."
    npx -y jscpd . --config .jscpd.json
fi

echo "📊 Duplication report generated in $REPORT_DIR"
