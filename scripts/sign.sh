#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR/.."

APP_PATH="$PROJECT_DIR/build/PromptOS.app"
IDENTITY="${DEVELOPER_ID_CERT:-Developer ID Application}"

echo "=== Signing PromptOS ==="

codesign --deep --force --verify-verbose --options runtime --timestamp \
    --entitlements "$PROJECT_DIR/promptos.entitlements" \
    -s "$IDENTITY" \
    "$APP_PATH"

echo "Verifying signature..."
codesign --verify --deep --strict "$APP_PATH"
spctl --assess --verbose --type exec "$APP_PATH"

echo "=== Signing complete ==="
