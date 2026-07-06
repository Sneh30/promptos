#!/bin/bash
set -euo pipefail

export PATH="$HOME/.cargo/bin:$PATH"

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RUST_TARGET="aarch64-apple-darwin"
DYLIB_NAME="libpromptos_llama.dylib"

echo "Building Rust dylib..."
cd "$PROJECT_ROOT"
cargo build --release -p promptos-llama

DYLIB_SRC="$PROJECT_ROOT/target/release/$DYLIB_NAME"
SWIFT_DYLIB_DIR="$PROJECT_ROOT/swift/.build/debug"
mkdir -p "$SWIFT_DYLIB_DIR"

if [ -f "$DYLIB_SRC" ]; then
    cp "$DYLIB_SRC" "$SWIFT_DYLIB_DIR/$DYLIB_NAME"
    echo "Copied $DYLIB_NAME to $SWIFT_DYLIB_DIR"
else
    echo "Warning: $DYLIB_NAME not found at $DYLIB_SRC"
fi
