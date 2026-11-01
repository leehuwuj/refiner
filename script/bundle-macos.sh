#!/usr/bin/env bash
# Builds target/release/Refiner.app: a menu-bar app (LSUIElement) with the
# Refiner icon. Grant it Accessibility access to use the ⌘E shortcut.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
cargo build --release --locked

APP=target/release/Refiner.app
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/refiner "$APP/Contents/MacOS/refiner"
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
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>LSUIElement</key><true/>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSAppleEventsUsageDescription</key><string>Refiner sends ⌘C to the frontmost app to read the selected text.</string>
</dict>
</plist>
PLIST

# Ad-hoc signature so macOS remembers the Accessibility grant between launches.
codesign --force --deep --sign - "$APP" >/dev/null 2>&1 || true
echo "Built $APP"
