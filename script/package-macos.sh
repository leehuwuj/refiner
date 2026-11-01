#!/usr/bin/env bash
# Build the native GPUI app, then preserve its bundle/signature in ZIP and DMG.
set -euo pipefail
cd "$(dirname "$0")/.."

TARGET="${1:-$(rustc -vV | sed -n 's/^host: //p')}"
case "$TARGET" in
  aarch64-apple-darwin) ARCH=arm64 ;;
  x86_64-apple-darwin) ARCH=x64 ;;
  *) echo "Unsupported macOS target: $TARGET" >&2; exit 2 ;;
esac
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
./script/bundle-macos.sh "$TARGET"
APP="${CARGO_TARGET_DIR:-target}/$TARGET/release/Refiner.app"
mkdir -p dist
NAME="Refiner_${VERSION}_macos_${ARCH}"
ditto -c -k --sequesterRsrc --keepParent "$APP" "dist/$NAME.zip"

STAGE=$(mktemp -d "${TMPDIR:-/tmp}/refiner-dmg.XXXXXX")
trap 'rm -rf "$STAGE"' EXIT
ditto "$APP" "$STAGE/Refiner.app"
ln -s /Applications "$STAGE/Applications"
hdiutil create -volname Refiner -srcfolder "$STAGE" -fs HFS+ -format UDZO -ov "dist/$NAME.dmg"
hdiutil verify "dist/$NAME.dmg"
echo "Packaged dist/$NAME.zip and dist/$NAME.dmg"
