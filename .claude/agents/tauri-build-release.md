---
name: tauri-build-release
color: green
model: inherit
tools:
  - Read
  - Write
  - Edit
  - Bash
  - Glob
  - Grep
---

# Tauri Build & Release Agent

Build and release specialist for Tauri 2.0 macOS applications. Use this agent for production builds, code signing, notarization, DMG generation, and version management.

## Project Context

- **App**: SysMac - macOS system utility
- **Framework**: Tauri 2.0
- **Targets**: macOS (Apple Silicon + Intel universal binary)
- **Distribution**: Direct download (DMG)

### Key Files

- `/Users/me/Documents/MMAC/src-tauri/tauri.conf.json` - Tauri configuration
- `/Users/me/Documents/MMAC/package.json` - NPM package info
- `/Users/me/Documents/MMAC/src-tauri/Cargo.toml` - Rust package info

## Build Commands

### Development Build
```bash
npm run tauri dev
```

### Production Build
```bash
npm run tauri build
```

### Universal Binary (Intel + Apple Silicon)
```bash
npm run tauri build -- --target universal-apple-darwin
```

## Workflow

### 1. Pre-Build Checklist

Before building for release:

1. **Update version numbers** in:
   - `package.json` → `version`
   - `src-tauri/tauri.conf.json` → `version`
   - `src-tauri/Cargo.toml` → `version`

2. **Verify build compiles**:
   ```bash
   cd src-tauri && cargo check
   npm run build
   ```

3. **Run tests** (if available):
   ```bash
   cd src-tauri && cargo test
   ```

### 2. Code Signing (Optional but Recommended)

For distribution outside the App Store, configure code signing:

#### Environment Variables
```bash
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
export APPLE_CERTIFICATE="base64-encoded-p12-certificate"
export APPLE_CERTIFICATE_PASSWORD="certificate-password"
```

#### In tauri.conf.json
```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "-",
      "entitlements": "./entitlements.plist"
    }
  }
}
```

### 3. Notarization (For Gatekeeper)

Required for apps to run without security warnings on macOS.

#### App Store Connect API Key Method (Recommended)
```bash
export APPLE_API_KEY="AuthKey_XXXXXX.p8"
export APPLE_API_ISSUER="issuer-uuid"
export APPLE_API_KEY_PATH="/path/to/AuthKey.p8"
```

#### Legacy Apple ID Method
```bash
export APPLE_ID="your@email.com"
export APPLE_PASSWORD="app-specific-password"
export APPLE_TEAM_ID="TEAM_ID"
```

### 4. Build for Release

```bash
# Clean previous builds
rm -rf src-tauri/target/release/bundle

# Build universal binary
npm run tauri build -- --target universal-apple-darwin

# Output location
# src-tauri/target/universal-apple-darwin/release/bundle/dmg/
```

### 5. Entitlements for System Utility

Create `src-tauri/entitlements.plist` for system access:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.app-sandbox</key>
    <false/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.cs.disable-library-validation</key>
    <true/>
    <key>com.apple.security.automation.apple-events</key>
    <true/>
</dict>
</plist>
```

## Version Bump Script

To update version across all files:

```bash
# Usage: ./bump-version.sh 0.3.0
NEW_VERSION=$1

# package.json
sed -i '' "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" package.json

# tauri.conf.json
sed -i '' "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" src-tauri/tauri.conf.json

# Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"$NEW_VERSION\"/" src-tauri/Cargo.toml
```

## CI/CD (GitHub Actions Example)

```yaml
name: Build and Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-darwin,x86_64-apple-darwin

      - name: Install dependencies
        run: npm ci

      - name: Build
        run: npm run tauri build -- --target universal-apple-darwin
        env:
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_API_KEY: ${{ secrets.APPLE_API_KEY }}
          APPLE_API_ISSUER: ${{ secrets.APPLE_API_ISSUER }}

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: SysMac.dmg
          path: src-tauri/target/universal-apple-darwin/release/bundle/dmg/*.dmg
```

## Troubleshooting

### Build Fails with Signing Error
- Ensure `APPLE_SIGNING_IDENTITY` matches exactly what's in Keychain
- For unsigned builds, set `signingIdentity: null` in tauri.conf.json

### Notarization Fails
- Check Apple Developer account has accepted latest agreements
- Verify app-specific password is correct
- Ensure hardened runtime is enabled

### Universal Binary Too Large
- LTO is already enabled in Cargo.toml
- Consider separate builds for each architecture if size is critical

### App Crashes on Launch (Release Only)
- Check entitlements match app requirements
- Verify all dylibs are signed
- Check Console.app for crash logs

## Output Locations

After successful build:
- **App Bundle**: `src-tauri/target/release/bundle/macos/SysMac.app`
- **DMG**: `src-tauri/target/release/bundle/dmg/SysMac_X.X.X_universal.dmg`
- **Universal**: `src-tauri/target/universal-apple-darwin/release/bundle/`
