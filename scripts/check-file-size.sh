#!/bin/bash
set -euo pipefail

# Enforce a maximum of 100 lines for source files not in the allowlist.
# Existing files that exceeded the limit are grandfathered via .file-size-allowlist.

MAX_LINES=100
ALLOWLIST=".file-size-allowlist"
FAILED=0

if [ ! -f "$ALLOWLIST" ]; then
    echo "Error: $ALLOWLIST not found"
    exit 1
fi

# Load allowlist into an associative-style check (portable sh-compatible)
allowlisted() {
    grep -qxF "$1" "$ALLOWLIST"
}

# Find all source files matching the extensions
while IFS= read -r file; do
    lines=$(wc -l < "$file" | tr -d ' ')
    if [ "$lines" -gt "$MAX_LINES" ]; then
        if ! allowlisted "$file"; then
            echo "Error: $file has $lines lines (max $MAX_LINES)"
            FAILED=1
        fi
    fi
done < <(find src/ proto/ -type f \( -name "*.rs" -o -name "*.proto" \) | sort)

if [ "$FAILED" -eq 1 ]; then
    echo ""
    echo "New files must not exceed $MAX_LINES lines."
    echo "Refactor into smaller modules instead."
    exit 1
fi

echo "File size check passed."
