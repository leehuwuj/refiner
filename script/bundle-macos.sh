#!/usr/bin/env bash
# Builds a menu-bar Refiner.app (LSUIElement). An optional Rust target puts
# the bundle under target/<target>/release; no argument keeps target/release.
set -euo pipefail
cd "$(dirname "$0")/.."

if (( $# > 1 )); then
  echo "Usage: $0 [aarch64-apple-darwin|x86_64-apple-darwin]" >&2
  exit 2
fi
TARGET="${1:-}"
case "$TARGET" in
  ""|aarch64-apple-darwin|x86_64-apple-darwin) ;;
  *) echo "Unsupported macOS target: $TARGET" >&2; exit 2 ;;
esac
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-11.0}"
BUILD_ARGS=(build --release --locked)
if [[ -n "$TARGET" ]]; then BUILD_ARGS+=(--target "$TARGET"); fi
RELEASE_DIR="${CARGO_TARGET_DIR:-target}/${TARGET:+$TARGET/}release"

VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
cargo "${BUILD_ARGS[@]}"

APP="$RELEASE_DIR/Refiner.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$RELEASE_DIR/refiner" "$APP/Contents/MacOS/refiner"
cp assets/icons/icon.icns "$APP/Contents/Resources/icon.icns"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Refiner</string>
  <key>CFBundleDisplayName</key><string>Refiner</string>
  <key>CFBundleIdentifier</key><string>dev.leehuwuj.refiner</string>
  <key>CFBundleVersion</key><string>${VERSION}</string>
  <key>CFBundleShortVersionString</key><string>${VERSION}</string>
  <key>CFBundleExecutable</key><string>refiner</string>
  <key>CFBundleIconFile</key><string>icon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSMinimumSystemVersion</key><string>${MACOSX_DEPLOYMENT_TARGET}</string>
  <key>LSUIElement</key><true/>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSAppleEventsUsageDescription</key><string>Refiner sends ⌘C to the frontmost app to read the selected text.</string>
</dict>
</plist>
PLIST

# Ad-hoc signature so macOS remembers the Accessibility grant between launches.
plutil -lint "$APP/Contents/Info.plist"
codesign --force --deep --sign - "$APP"
codesign --verify --deep --strict "$APP"
echo "Built $APP"
