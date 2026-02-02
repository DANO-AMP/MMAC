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

# Copy app icon
echo "[3/5] Adding app icon..."
ICNS_SRC="$PROJECT_DIR/SysMac/Resources/AppIcon.icns"
if [ -f "$ICNS_SRC" ]; then
    cp "$ICNS_SRC" "$APP_BUNDLE/Contents/Resources/AppIcon.icns"
    echo "  Copied AppIcon.icns"
else
    echo "  Warning: AppIcon.icns not found, run scripts/generate-icon.py first"
fi

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

rm -rf "$DMG_TEMP"

echo ""
echo "=== Build complete ==="
echo "  App:  $APP_BUNDLE"
echo "  DMG:  $BUILD_DIR/$DMG_NAME"
echo "  Size: $(du -h "$BUILD_DIR/$DMG_NAME" | cut -f1)"
echo ""
echo "To install: open $BUILD_DIR/$DMG_NAME"
