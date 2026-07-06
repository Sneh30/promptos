.PHONY: build test lint clean release package sign notarize swift-build swift-test

# Build all Rust crates
build:
	cargo build --workspace

# Build release binaries
build-release:
	cargo build --workspace --release

# Run all tests
test:
	cargo test --workspace

# Run linting
lint:
	cargo clippy --workspace -- -D warnings

# Clean build artifacts
clean:
	cargo clean
	rm -rf swift/PromptOSApp/build

# Universal binary build for macOS
universal:
	cargo build --release --target aarch64-apple-darwin
	cargo build --release --target x86_64-apple-darwin
	mkdir -p target/universal
	lipo -create \
		target/aarch64-apple-darwin/release/libpromptos_core.a \
		target/x86_64-apple-darwin/release/libpromptos_core.a \
		-output target/universal/libpromptos_core.a

# Swift build
swift-build:
	cd swift && xcodebuild -scheme PromptOSApp -derivedDataPath build -configuration Debug

swift-release:
	cd swift && xcodebuild -scheme PromptOSApp -derivedDataPath build -configuration Release

# Full release build
release: build-release universal swift-release
	@echo "Release build complete"

# Package into .app
package: release
	mkdir -p build/PromptOS.app/Contents/MacOS
	mkdir -p build/PromptOS.app/Contents/Resources
	mkdir -p build/PromptOS.app/Contents/Resources/profiles
	cp -r swift/build/Build/Products/Release/PromptOSApp.app/Contents/MacOS/* build/PromptOS.app/Contents/MacOS/
	cp crates/prompts-profiles/profiles/*.toml build/PromptOS.app/Contents/Resources/profiles/ 2>/dev/null || true
	@echo "App bundle created at build/PromptOS.app"

# Sign the app bundle
sign:
	codesign --deep --force --verify-verbose --options runtime --timestamp \
		--entitlements promptos.entitlements \
		-s "Developer ID Application" \
		build/PromptOS.app

# Notarize the app
notarize:
	xcrun notarytool submit build/PromptOS.dmg \
		--apple-id "$(APPLE_ID)" \
		--password "$(APPLE_PASSWORD)" \
		--team-id "$(TEAM_ID)" \
		--wait
	xcrun stapler staple build/PromptOS.dmg

# Create DMG
dmg:
	mkdir -p build/dmg
	cp -r build/PromptOS.app build/dmg/
	ln -s /Applications build/dmg/Applications
	hdiutil create -volname "PromptOS" -srcfolder build/dmg -ov -format UDZO build/PromptOS.dmg
	rm -rf build/dmg

# Run evaluation benchmarks
evaluate:
	cargo run --bin evaluate

# Print help
help:
	@echo "PromptOS Build Targets:"
	@echo "  build       - Build all Rust crates"
	@echo "  test        - Run all tests"
	@echo "  lint        - Run clippy linter"
	@echo "  clean       - Clean build artifacts"
	@echo "  release     - Full release build"
	@echo "  package     - Create .app bundle"
	@echo "  sign        - Code sign the app"
	@echo "  notarize    - Notarize with Apple"
	@echo "  dmg         - Create DMG installer"
	@echo "  evaluate    - Run evaluation benchmarks"

package: ## Build self-contained .app bundle with model + llama.cpp
	@bash scripts/package.sh

dist: package ## Build distributable ZIP
	@cd swift/.build && zip -r "PromptOSApp-$(shell date +%Y%m%d).zip" PromptOSApp.app && echo "Created swift/.build/PromptOSApp-$(shell date +%Y%m%d).zip"
