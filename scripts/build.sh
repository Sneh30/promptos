#!/bin/bash
set -euo pipefail

echo "=== PromptOS Build Script ==="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR/.."

# Build Rust crates
echo ""
echo "--- Building Rust crates ---"
cd "$PROJECT_DIR"
cargo build --workspace --release 2>&1 | tail -5

# Create universal binary
echo ""
echo "--- Creating universal binary ---"
mkdir -p target/universal
lipo -create \
    target/aarch64-apple-darwin/release/libpromptos_core.a \
    target/x86_64-apple-darwin/release/libpromptos_core.a \
    -output target/universal/libpromptos_core.a 2>/dev/null || \
    cp target/release/libpromptos_core.a target/universal/libpromptos_core.a 2>/dev/null || true

# Build Swift app
echo ""
echo "--- Building Swift app ---"
cd "$PROJECT_DIR/swift"
xcodebuild -scheme PromptOSApp -derivedDataPath build -configuration Release 2>&1 | tail -10

echo ""
echo "=== Build complete ==="
