#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
target="${1:?Rust target required}"
case "$target" in
  aarch64-apple-darwin) arch=arm64 ;;
  x86_64-apple-darwin) arch=x64 ;;
  *) echo "Unsupported target: $target" >&2; exit 1 ;;
esac
out="artifacts/macos-$arch"
app="$out/EnterGuard.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "rust/target/$target/release/enter-guard" "$app/Contents/MacOS/enter-guard"
chmod 755 "$app/Contents/MacOS/enter-guard"
cp packaging/macos/Info.plist "$app/Contents/Info.plist"
plutil -lint "$app/Contents/Info.plist"
iconset="$(mktemp -d)/EnterGuard.iconset"
mkdir -p "$iconset"
for size in 16 32 128 256; do
  sips -z "$size" "$size" assets/EnterGuard.png --out "$iconset/icon_${size}x${size}.png" >/dev/null
done
for size in 16 32 128; do
  doubled=$((size * 2))
  sips -z "$doubled" "$doubled" assets/EnterGuard.png --out "$iconset/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$iconset" -o "$app/Contents/Resources/EnterGuard.icns"
rm -rf "$(dirname "$iconset")"
# Local ad-hoc signature: no Developer ID, credentials or notarization service.
codesign --force --sign - --identifier works.spa77.enterguard "$app"
codesign --verify --deep --strict --verbose=2 "$app"
cp docs/MACOS.md "$out/はじめに.txt"
mkdir -p artifacts/distribution
ditto -c -k --sequesterRsrc --keepParent "$app" "artifacts/distribution/EnterGuard-mac-$arch.zip"
cp "$out/はじめに.txt" "artifacts/distribution/EnterGuard-mac-使い方.txt"
(cd artifacts/distribution && shasum -a 256 "EnterGuard-mac-$arch.zip" > "SHA256SUMS-mac-$arch.txt")
