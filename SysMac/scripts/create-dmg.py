#!/usr/bin/env python3
"""Create DMG for SysMac macOS application."""
import os
import subprocess
import shutil
from pathlib import Path

def main():
    # Paths
    project_root = Path("/Users/me/Documents/MMAC/SysMac")
    dist_dir = project_root / "dist"
    app_dir = dist_dir / "SysMac.app"
    contents_dir = app_dir / "Contents"
    macos_dir = contents_dir / "MacOS"
    resources_dir = contents_dir / "Resources"
    
    # Executable path
    executable = project_root / ".build" / "arm64-apple-macosx" / "release" / "SysMac"
    
    print("Creating DMG for SysMac...")
    
    # Verify executable exists
    if not executable.exists():
        print(f"❌ Executable not found: {executable}")
        print("Building in release mode first...")
        subprocess.run(["swift", "build", "--configuration", "release"], 
                      cwd=project_root, check=True)
    
    # Clean dist directory
    if dist_dir.exists():
        shutil.rmtree(dist_dir)
    
    # Create .app structure
    print(f"Creating .app bundle at {app_dir}...")
    macos_dir.mkdir(parents=True, exist_ok=True)
    resources_dir.mkdir(parents=True, exist_ok=True)
    
    # Copy executable
    print(f"Copying executable from {executable}...")
    shutil.copy2(executable, macos_dir / "SysMac")
    os.chmod(macos_dir / "SysMac", 0o755)
    
    # Copy resources
    resources_src = project_root / "SysMac" / "Resources"
    if resources_src.exists():
        print("Copying resources...")
        for item in resources_src.iterdir():
            if item.name == "AppIcon.icns":
                shutil.copy2(item, resources_dir)
    
    # Create Info.plist
    print("Creating Info.plist...")
    info_plist = contents_dir / "Info.plist"
    info_plist.write_text("""<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>SysMac</string>
    <key>CFBundleIdentifier</key>
    <string>com.sysmac.app</string>
    <key>CFBundleName</key>
    <string>SysMac</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>LSMinimumSystemVersion</key>
    <string>13.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticTermination</key>
    <true/>
    <key>NSSupportsSuddenTermination</key>
    <false/>
    <key>NSRequiresAquaSystemAppearance</key>
    <false/>
</dict>
</plist>
""")
    
    # Create simple DMG
    dmg_path = dist_dir / "SysMac-1.0.dmg"
    print(f"Creating DMG at {dmg_path}...")
    
    # Use hdiutil create directly
    subprocess.run([
        "hdiutil", "create",
        "-srcfolder", str(dist_dir),
        "-volname", "SysMac",
        "-format", "UDZO",
        "-imagekey", "zlib-level=9",
        str(dmg_path)
    ], check=True)
    
    print(f"\n✅ DMG created successfully!")
    print(f"Location: {dmg_path}")
    
    # Get file size
    size_mb = dmg_path.stat().st_size / (1024 * 1024)
    print(f"Size: {size_mb:.1f} MB")
    
    return 0

if __name__ == "__main__":
    exit(main())