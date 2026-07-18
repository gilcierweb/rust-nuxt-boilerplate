#!/usr/bin/env bash
# Security Headers Audit Script
# Checks that required security headers are present in HTTP responses

set -euo pipefail

BASE_URL="${1:-http://localhost:3000}"
FAIL_ON_MISSING="${2:-true}"

echo "🔍 Auditing security headers for: $BASE_URL"

# Required security headers and their expected patterns
declare -A REQUIRED_HEADERS=(
    ["strict-transport-security"]="max-age="
    ["x-frame-options"]="DENY|SAMEORIGIN"
    ["x-content-type-options"]="nosniff"
    ["referrer-policy"]="strict-origin-when-cross-origin|no-referrer|same-origin"
    ["permissions-policy"]=".*"
    ["content-security-policy"]="default-src"
)

# Optional but recommended headers
declare -A RECOMMENDED_HEADERS=(
    ["cross-origin-opener-policy"]="same-origin|same-origin-allow-popups"
    ["cross-origin-resource-policy"]="same-origin|cross-origin"
    ["cross-origin-embedder-policy"]="require-corp"
)

check_header() {
    local header_name="$1"
    local expected_pattern="$2"
    local header_value="$3"
    local is_required="$4"

    if [[ -z "$header_value" ]]; then
        if [[ "$is_required" == "true" ]]; then
            echo "❌ MISSING REQUIRED: $header_name"
            return 1
        else
            echo "⚠️  MISSING RECOMMENDED: $header_name"
            return 0
        fi
    fi

    if [[ "$header_value" =~ $expected_pattern ]]; then
        echo "✅ $header_name: $header_value"
        return 0
    else
        if [[ "$is_required" == "true" ]]; then
            echo "❌ INVALID REQUIRED: $header_name = '$header_value' (expected: $expected_pattern)"
            return 1
        else
            echo "⚠️  INVALID RECOMMENDED: $header_name = '$header_value' (expected: $expected_pattern)"
            return 0
        fi
    fi
}

# Fetch headers from the base URL
echo "📡 Fetching headers from $BASE_URL ..."
HEADERS=$(curl -sSI "$BASE_URL" --max-time 30 2>/dev/null || echo "CURL_FAILED")

if [[ "$HEADERS" == "CURL_FAILED" ]]; then
    echo "❌ Failed to fetch headers from $BASE_URL"
    exit 1
fi

echo "📋 Checking required security headers..."
FAILED=0

for header in "${!REQUIRED_HEADERS[@]}"; do
    # Extract header value (case-insensitive)
    value=$(echo "$HEADERS" | grep -i "^${header}:" | sed 's/^[^:]*: *//' | tr -d '\r\n')
    check_header "$header" "${REQUIRED_HEADERS[$header]}" "$value" "true" || FAILED=1
done

echo ""
echo "📋 Checking recommended security headers..."
for header in "${!RECOMMENDED_HEADERS[@]}"; do
    value=$(echo "$HEADERS" | grep -i "^${header}:" | sed 's/^[^:]*: *//' | tr -d '\r\n')
    check_header "$header" "${RECOMMENDED_HEADERS[$header]}" "$value" "false" || true
done

echo ""
if [[ $FAILED -eq 0 ]]; then
    echo "✅ All required security headers are present and valid!"
    exit 0
else
    echo "❌ Some required security headers are missing or invalid."
    if [[ "$FAIL_ON_MISSING" == "true" ]]; then
        exit 1
    else
        exit 0
    fi
fi