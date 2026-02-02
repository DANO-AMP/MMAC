#!/bin/bash
set -euo pipefail

# Configuration
APP_NAME="SysMac"
VERSION="1.0.0"
BUNDLE_ID="com.sysmac.app"
MIN_MACOS="13.0"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/dist"
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"
DMG_NAME="$APP_NAME-$VERSION.dmg"

echo "=== Building $APP_NAME v$VERSION ==="

# Clean previous build
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

# Build release binary
echo "[1/5] Compiling release binary..."
cd "$PROJECT_DIR"
swift build -c release --arch arm64 --arch x86_64 2>&1 | tail -5

BINARY="$PROJECT_DIR/.build/apple/Products/Release/$APP_NAME"
if [ ! -f "$BINARY" ]; then
    # Fallback: single-arch build
    echo "  Universal build failed, trying single arch..."
    swift build -c release 2>&1 | tail -3
    BINARY="$PROJECT_DIR/.build/release/$APP_NAME"
fi

echo "  Binary: $BINARY ($(du -h "$BINARY" | cut -f1))"

# Create .app bundle structure
echo "[2/5] Creating .app bundle..."
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

cp "$BINARY" "$APP_BUNDLE/Contents/MacOS/$APP_NAME"

# Info.plist
cat > "$APP_BUNDLE/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>es</string>
    <key>CFBundleExecutable</key>
    <string>$APP_NAME</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.utilities</string>
    <key>LSMinimumSystemVersion</key>
    <string>$MIN_MACOS</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright 2026. All rights reserved.</string>
</dict>
</plist>
PLIST

# PkgInfo
echo -n "APPL????" > "$APP_BUNDLE/Contents/PkgInfo"

# Generate app icon using sips (macOS built-in)
echo "[3/5] Generating app icon..."
ICON_DIR="$BUILD_DIR/AppIcon.iconset"
mkdir -p "$ICON_DIR"

# Create a simple icon with a colored background using Python
ICON_DIR="$ICON_DIR" python3 << 'PYEOF'
import struct, zlib, os, sys

def create_png(width, height, filepath):
    """Create a simple gradient PNG icon."""
    raw_data = b""
    for y in range(height):
        raw_data += b"\x00"  # filter byte
        for x in range(width):
            # Dark background with blue-purple gradient
            r = int(30 + 40 * (x / width))
            g = int(30 + 20 * (y / height))
            b = int(80 + 120 * (x / width) * (1 - y / height))
            a = 255
            # Circle mask for rounded look
            cx, cy = width / 2, height / 2
            radius = width * 0.45
            dist = ((x - cx) ** 2 + (y - cy) ** 2) ** 0.5
            if dist > radius:
                a = max(0, int(255 * (1 - (dist - radius) / (width * 0.05))))
            raw_data += struct.pack("BBBB", r, g, b, a)

    def make_chunk(chunk_type, data):
        chunk = chunk_type + data
        return struct.pack(">I", len(data)) + chunk + struct.pack(">I", zlib.crc32(chunk) & 0xFFFFFFFF)

    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    png = b"\x89PNG\r\n\x1a\n"
    png += make_chunk(b"IHDR", ihdr)
    png += make_chunk(b"IDAT", zlib.compress(raw_data))
    png += make_chunk(b"IEND", b"")

    with open(filepath, "wb") as f:
        f.write(png)

icon_dir = os.environ.get("ICON_DIR", ".")
sizes = [16, 32, 64, 128, 256, 512, 1024]
for s in sizes:
    create_png(s, s, os.path.join(icon_dir, f"icon_{s}x{s}.png"))
    if s <= 512:
        create_png(s * 2, s * 2, os.path.join(icon_dir, f"icon_{s}x{s}@2x.png"))
PYEOF

# Rename to iconset format
cd "$ICON_DIR"
mv icon_16x16.png icon_16x16.png 2>/dev/null || true
mv icon_32x32.png icon_32x32.png 2>/dev/null || true
mv icon_16x16@2x.png icon_16x16@2x.png 2>/dev/null || true
mv icon_32x32@2x.png icon_32x32@2x.png 2>/dev/null || true
mv icon_128x128.png icon_128x128.png 2>/dev/null || true
mv icon_128x128@2x.png icon_128x128@2x.png 2>/dev/null || true
mv icon_256x256.png icon_256x256.png 2>/dev/null || true
mv icon_256x256@2x.png icon_256x256@2x.png 2>/dev/null || true
mv icon_512x512.png icon_512x512.png 2>/dev/null || true
mv icon_512x512@2x.png icon_512x512@2x.png 2>/dev/null || true

# Convert to icns
iconutil -c icns "$ICON_DIR" -o "$APP_BUNDLE/Contents/Resources/AppIcon.icns" 2>/dev/null || {
    echo "  Warning: iconutil failed, app will use default icon"
}

# Ad-hoc code signing
echo "[4/5] Code signing (ad-hoc)..."
codesign --force --deep --sign - "$APP_BUNDLE" 2>&1 || {
    echo "  Warning: code signing failed, DMG will still work locally"
}

# Create DMG
echo "[5/5] Creating DMG..."
DMG_TEMP="$BUILD_DIR/dmg_temp"
mkdir -p "$DMG_TEMP"
cp -R "$APP_BUNDLE" "$DMG_TEMP/"

# Create Applications symlink for drag-to-install
ln -s /Applications "$DMG_TEMP/Applications"

hdiutil create \
    -volname "$APP_NAME" \
    -srcfolder "$DMG_TEMP" \
    -ov \
    -format UDZO \
    "$BUILD_DIR/$DMG_NAME" 2>&1 | tail -2

rm -rf "$DMG_TEMP" "$ICON_DIR"

echo ""
echo "=== Build complete ==="
echo "  App:  $APP_BUNDLE"
echo "  DMG:  $BUILD_DIR/$DMG_NAME"
echo "  Size: $(du -h "$BUILD_DIR/$DMG_NAME" | cut -f1)"
echo ""
echo "To install: open $BUILD_DIR/$DMG_NAME"
