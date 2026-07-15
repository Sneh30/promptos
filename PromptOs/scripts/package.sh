#!/bin/bash
set -euo pipefail

export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/swift/.build"
APP_NAME="PromptOSApp"
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"
MODEL_FILENAME="qwen2.5-0.5b-instruct-q4_k_m.gguf"
MODELS_DIR="$HOME/Library/Application Support/com.promptos.app/models"
FW="$APP_BUNDLE/Contents/Frameworks"
MACOS="$APP_BUNDLE/Contents/MacOS"

echo "=== Building self-contained $APP_NAME.app ==="

echo "[1/5] Building Rust dylib..."
cargo build --release -p promptos-llama 2>&1 | tail -1

echo "[2/5] Building Swift app..."
cd "$PROJECT_ROOT/swift"
swift build -c release 2>&1 | tail -1

echo "[3/5] Creating app bundle structure..."
rm -rf "$APP_BUNDLE"
mkdir -p "$MACOS" "$APP_BUNDLE/Contents/Resources" "$FW"

echo "[4/5] Copying binaries, libraries, and model..."
cp "$BUILD_DIR/release/$APP_NAME" "$MACOS/"
cp "$PROJECT_ROOT/target/release/libpromptos_llama.dylib" "$FW/"
cp "$PROJECT_ROOT/swift/Sources/PromptOSApp/Info.plist" "$APP_BUNDLE/Contents/"
cp "$PROJECT_ROOT/swift/Sources/PromptOSApp/Resources/AppIcon.icns" "$APP_BUNDLE/Contents/Resources/"

# ---- Bundle llama-completion with all its dylib dependencies ----
LLAMA_BIN="/opt/homebrew/bin/llama-completion"
cp "$LLAMA_BIN" "$MACOS/llama-completion"

# Resolve and copy all @rpath and brew-path dylibs
# These are the direct dependencies of llama-completion
for lib in \
    /opt/homebrew/opt/llama.cpp/lib/libllama-completion-impl.dylib \
    /opt/homebrew/opt/llama.cpp/lib/libllama-common.0.dylib \
    /opt/homebrew/opt/llama.cpp/lib/libllama.0.dylib \
    /opt/homebrew/opt/ggml/lib/libggml.0.dylib \
    /opt/homebrew/opt/ggml/lib/libggml-base.0.dylib \
    /opt/homebrew/opt/openssl@3/lib/libssl.3.dylib \
    /opt/homebrew/opt/openssl@3/lib/libcrypto.3.dylib; do
    if [ -f "$lib" ]; then
        cp -L "$lib" "$FW/"
    fi
done

# Fix rpath in llama-completion binary: add @loader_path/../Frameworks
install_name_tool -add_rpath "@executable_path/../Frameworks" "$MACOS/llama-completion" 2>/dev/null || true

# Change direct library references from abs paths to @rpath
for lib in \
    libllama-completion-impl \
    libllama-common.0 \
    libllama.0 \
    libggml.0 \
    libggml-base.0 \
    libssl.3 \
    libcrypto.3; do
    install_name_tool -change "/opt/homebrew/opt/llama.cpp/lib/$lib.dylib" "@rpath/$lib.dylib" "$MACOS/llama-completion" 2>/dev/null || true
    install_name_tool -change "/opt/homebrew/opt/ggml/lib/$lib.dylib" "@rpath/$lib.dylib" "$MACOS/llama-completion" 2>/dev/null || true
    install_name_tool -change "/opt/homebrew/opt/openssl@3/lib/$lib.dylib" "@rpath/$lib.dylib" "$MACOS/llama-completion" 2>/dev/null || true
done

# Fix cross-references between the dylibs themselves
for dylib in "$FW"/*.dylib; do
    base=$(basename "$dylib")
    for lib in \
        libllama-completion-impl \
        libllama-common.0 \
        libllama.0 \
        libggml.0 \
        libggml-base.0 \
        libssl.3 \
        libcrypto.3; do
        install_name_tool -change "/opt/homebrew/opt/llama.cpp/lib/$lib.dylib" "@rpath/$lib.dylib" "$dylib" 2>/dev/null || true
        install_name_tool -change "/opt/homebrew/opt/ggml/lib/$lib.dylib" "@rpath/$lib.dylib" "$dylib" 2>/dev/null || true
        install_name_tool -change "/opt/homebrew/opt/openssl@3/lib/$lib.dylib" "@rpath/$lib.dylib" "$dylib" 2>/dev/null || true
    done
    # Fix dylib's own identity to use @rpath
    install_name_tool -id "@rpath/$base" "$dylib" 2>/dev/null || true
done

echo "  Bundled llama-completion + $(ls "$FW"/*.dylib 2>/dev/null | wc -l | tr -d ' ') dylibs"

# Copy model
if [ -f "$MODELS_DIR/$MODEL_FILENAME" ]; then
    cp "$MODELS_DIR/$MODEL_FILENAME" "$APP_BUNDLE/Contents/Resources/$MODEL_FILENAME"
    MODEL_SIZE=$(stat -f%z "$APP_BUNDLE/Contents/Resources/$MODEL_FILENAME" 2>/dev/null || echo 0)
    echo "  Bundled model ($(($MODEL_SIZE / 1048576)) MB)"
else
    echo "  WARNING: Model not found at $MODELS_DIR/$MODEL_FILENAME"
fi

echo "[5/5] Signing app bundle (ad-hoc)..."
for f in "$FW"/*.dylib "$MACOS/llama-completion"; do
    codesign --force --sign - "$f" 2>/dev/null || true
done
codesign --force --deep --sign - "$APP_BUNDLE" 2>/dev/null || true

echo ""
echo "=== Done: $APP_BUNDLE ==="
echo "Size: $(du -sh "$APP_BUNDLE" | cut -f1)"
echo "MacOS: $(ls -lh "$MACOS" | grep -v total)"
echo "Frameworks:"
ls -lh "$FW"/
echo "Resources:"
ls -lh "$APP_BUNDLE/Contents/Resources/"
