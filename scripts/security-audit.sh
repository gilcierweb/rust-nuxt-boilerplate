#!/bin/bash
# Security Audit Script
# Runs cargo audit and generates a report

set -e

echo "🔒 Running security audit..."
echo ""

cd "$(dirname "$0")/../backend"

# Check if cargo-audit is installed
if ! command -v cargo-audit &> /dev/null; then
    echo "Installing cargo-audit..."
    cargo install cargo-audit
fi

# Run cargo audit
echo "Scanning dependencies for security vulnerabilities..."
echo ""

if cargo audit; then
    echo ""
    echo "✅ No security vulnerabilities found!"
    exit 0
else
    echo ""
    echo "❌ Security vulnerabilities detected!"
    echo ""
    echo "Review the audit report above and update vulnerable dependencies."
    echo "See: https://rustsec.org/advisories/"
    exit 1
fi