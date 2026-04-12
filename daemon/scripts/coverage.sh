#!/bin/bash
set -euo pipefail

echo "Running code coverage with cargo-tarpaulin..."

if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

mkdir -p coverage
cargo tarpaulin --config .tarpaulin.toml

if [ -f "coverage/tarpaulin-report.html" ]; then
    echo "Coverage report: coverage/tarpaulin-report.html"
    [[ "$OSTYPE" == "darwin"* ]] && open coverage/tarpaulin-report.html
fi
