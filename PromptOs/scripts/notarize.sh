#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR/.."

DMG_PATH="$PROJECT_DIR/build/PromptOS.dmg"
APPLE_ID="${APPLE_ID:?Apple ID is required}"
APPLE_PASSWORD="${APPLE_PASSWORD:?App password is required}"
TEAM_ID="${TEAM_ID:?Team ID is required}"

echo "=== Notarizing PromptOS ==="

echo "Submitting to Apple notary..."
xcrun notarytool submit "$DMG_PATH" \
    --apple-id "$APPLE_ID" \
    --password "$APPLE_PASSWORD" \
    --team-id "$TEAM_ID" \
    --wait

echo "Stapling notarization ticket..."
xcrun stapler staple "$DMG_PATH"

echo "Verifying notarization..."
spctl --assess --verbose --type exec "$DMG_PATH"

echo "=== Notarization complete ==="
