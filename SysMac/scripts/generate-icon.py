#!/usr/bin/env python3
"""Generate SysMac app icon as .icns using only Python stdlib."""
import struct, zlib, os, subprocess, sys, math

ICON_DIR = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "SysMac", "Resources")
ICONSET_DIR = os.path.join(ICON_DIR, "AppIcon.iconset")

def create_png(width, height, filepath):
    """Create a macOS utility-style app icon PNG."""
    raw_data = b""
    cx, cy = width / 2, height / 2
    radius = width * 0.43
    border = width * 0.02

    for y in range(height):
        raw_data += b"\x00"  # PNG filter byte
        for x in range(width):
            # Normalized coords
            nx = (x - cx) / radius
            ny = (y - cy) / radius
            dist = math.sqrt(nx * nx + ny * ny)

            # Outside rounded rect -> transparent
            corner_r = 0.38
            ax = abs(nx)
            ay = abs(ny)
            in_rect = True
            if ax > 1.0 - corner_r and ay > 1.0 - corner_r:
                cdist = math.sqrt((ax - (1.0 - corner_r)) ** 2 + (ay - (1.0 - corner_r)) ** 2)
                if cdist > corner_r:
                    in_rect = False
            elif ax > 1.0 or ay > 1.0:
                in_rect = False

            if not in_rect:
                raw_data += struct.pack("BBBB", 0, 0, 0, 0)
                continue

            # Background gradient: deep blue to purple
            t = (ny + 1) / 2  # 0 at top, 1 at bottom
            s = (nx + 1) / 2  # 0 at left, 1 at right
            r = int(25 + 45 * s + 20 * t)
            g = int(20 + 15 * t)
            b = int(90 + 100 * (1 - t) + 40 * s)

            # Subtle inner glow at top
            if ny < -0.3:
                glow = (1 - (ny + 1) / 0.7) * 0.3
                r = min(255, int(r + glow * 80))
                g = min(255, int(g + glow * 60))
                b = min(255, int(b + glow * 100))

            # Draw a stylized monitor/waveform icon in the center
            a = 255

            # Monitor frame
            mon_w, mon_h = 0.55, 0.4
            mon_cx, mon_cy = 0.0, -0.08
            mon_x = (nx - mon_cx) / mon_w
            mon_y = (ny - mon_cy) / mon_h
            mon_border = 0.08 / min(mon_w, mon_h)

            in_monitor = abs(mon_x) < 1.0 and abs(mon_y) < 1.0
            in_screen = abs(mon_x) < 1.0 - mon_border and abs(mon_y) < 1.0 - mon_border

            # Monitor stand
            stand_in = abs(nx - mon_cx) < 0.08 and ny > mon_cy + mon_h and ny < mon_cy + mon_h + 0.15
            stand_base = abs(nx - mon_cx) < 0.18 and ny > mon_cy + mon_h + 0.12 and ny < mon_cy + mon_h + 0.18

            if in_monitor and not in_screen:
                # Silver bezel
                r, g, b = 180, 185, 195
            elif in_screen:
                # Dark screen background
                r, g, b = 15, 18, 30

                # Draw waveform/pulse line on screen
                screen_nx = mon_x / (1.0 - mon_border)
                screen_ny = mon_y / (1.0 - mon_border)
                sx = (screen_nx + 1) / 2  # 0 to 1 across screen

                # ECG-style waveform
                wave_y = 0.0
                if 0.1 < sx < 0.25:
                    wave_y = math.sin((sx - 0.1) / 0.15 * math.pi) * 0.15
                elif 0.3 < sx < 0.38:
                    wave_y = -math.sin((sx - 0.3) / 0.08 * math.pi) * 0.3
                elif 0.38 < sx < 0.42:
                    wave_y = math.sin((sx - 0.38) / 0.04 * math.pi) * 0.7
                elif 0.42 < sx < 0.50:
                    wave_y = -math.sin((sx - 0.42) / 0.08 * math.pi) * 0.25
                elif 0.55 < sx < 0.7:
                    wave_y = math.sin((sx - 0.55) / 0.15 * math.pi) * 0.12
                elif 0.75 < sx < 0.9:
                    wave_y = math.sin((sx - 0.75) / 0.15 * math.pi) * 0.08

                wave_dist = abs(screen_ny - wave_y)
                line_w = 0.06
                if wave_dist < line_w:
                    intensity = 1.0 - wave_dist / line_w
                    # Cyan/green glow
                    r = min(255, int(r + intensity * 50))
                    g = min(255, int(g + intensity * 240))
                    b = min(255, int(b + intensity * 200))

                    # Extra glow
                    if wave_dist < line_w * 0.4:
                        extra = (1 - wave_dist / (line_w * 0.4)) * 0.5
                        r = min(255, int(r + extra * 30))
                        g = min(255, int(g + extra * 50))
                        b = min(255, int(b + extra * 50))

                # Subtle grid lines on screen
                grid_x = (sx * 8) % 1.0
                grid_y = ((screen_ny + 1) / 2 * 6) % 1.0
                if grid_x < 0.03 or grid_y < 0.03:
                    r = min(255, r + 8)
                    g = min(255, g + 10)
                    b = min(255, b + 15)

            elif stand_in or stand_base:
                r, g, b = 150, 155, 165

            raw_data += struct.pack("BBBB", min(255, max(0, r)), min(255, max(0, g)), min(255, max(0, b)), a)

    # Encode PNG
    def make_chunk(chunk_type, data):
        chunk = chunk_type + data
        return struct.pack(">I", len(data)) + chunk + struct.pack(">I", zlib.crc32(chunk) & 0xFFFFFFFF)

    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    png = b"\x89PNG\r\n\x1a\n"
    png += make_chunk(b"IHDR", ihdr)
    png += make_chunk(b"IDAT", zlib.compress(raw_data, 9))
    png += make_chunk(b"IEND", b"")

    with open(filepath, "wb") as f:
        f.write(png)


def main():
    os.makedirs(ICONSET_DIR, exist_ok=True)

    # Required sizes for macOS .iconset
    sizes = {
        "icon_16x16.png": 16,
        "icon_16x16@2x.png": 32,
        "icon_32x32.png": 32,
        "icon_32x32@2x.png": 64,
        "icon_128x128.png": 128,
        "icon_128x128@2x.png": 256,
        "icon_256x256.png": 256,
        "icon_256x256@2x.png": 512,
        "icon_512x512.png": 512,
        "icon_512x512@2x.png": 1024,
    }

    print("Generating icon sizes...")
    for name, size in sizes.items():
        filepath = os.path.join(ICONSET_DIR, name)
        print(f"  {name} ({size}x{size})")
        create_png(size, size, filepath)

    # Convert to .icns
    icns_path = os.path.join(ICON_DIR, "AppIcon.icns")
    print(f"Converting to {icns_path}...")
    result = subprocess.run(
        ["iconutil", "-c", "icns", ICONSET_DIR, "-o", icns_path],
        capture_output=True, text=True
    )
    if result.returncode != 0:
        print(f"  Error: {result.stderr}")
        sys.exit(1)

    # Cleanup iconset
    import shutil
    shutil.rmtree(ICONSET_DIR)

    print(f"Done! Icon saved to {icns_path}")
    print(f"Size: {os.path.getsize(icns_path) / 1024:.0f} KB")


if __name__ == "__main__":
    main()
