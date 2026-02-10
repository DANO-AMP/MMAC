#!/usr/bin/env python3
"""Highly optimized SysMac app icon generator using only Python stdlib."""
import struct, zlib, os, subprocess, sys, math, shutil
import logging
from pathlib import Path
from typing import Tuple, Dict, List, Optional
from dataclasses import dataclass, field
from functools import lru_cache
import array
import time

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

@dataclass
class IconColors:
    """Icon color palette configuration."""
    background_top: Tuple[int, int, int] = (25, 20, 90)
    background_bottom: Tuple[int, int, int] = (70, 35, 130)
    monitor_silver: Tuple[int, int, int] = (180, 185, 195)
    screen_background: Tuple[int, int, int] = (15, 18, 30)
    glow_cyan: Tuple[int, int, int] = (0, 255, 255)
    glow_green: Tuple[int, int, int] = (50, 255, 50)
    stand_base: Tuple[int, int, int] = (150, 155, 165)
    grid_line: Tuple[int, int, int] = (8, 10, 15)

ICON_DIR = Path(os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "SysMac", "Resources"))
ICONSET_DIR = ICON_DIR / "AppIcon.iconset"
COLORS = IconColors()

def generate_background(width: int, height: int) -> bytearray:
    """Generate background gradient efficiently."""
    data = bytearray(width * height * 4)
    
    for y in range(height):
        t = (y / height - 0.5) * 2
        
        # Calculate gradient
        r = int(COLORS.background_top[0] + (COLORS.background_bottom[0] - COLORS.background_top[0]) * (t + 1) / 2)
        g = int(COLORS.background_top[1] + (COLORS.background_bottom[1] - COLORS.background_top[1]) * (t + 1) / 2)
        b = int(COLORS.background_top[2] + (COLORS.background_bottom[2] - COLORS.background_top[2]) * (t + 1) / 2)
        
        # Subtle inner glow at top
        if t < -0.3:
            glow = (1 - (t + 1) / 0.7) * 0.3
            r = min(255, r + glow * 80)
            g = min(255, g + glow * 60)
            b = min(255, b + glow * 100)
        
        for x in range(width):
            data[y * width * 4 + x * 4] = int(min(255, max(0, r)))
            data[y * width * 4 + x * 4 + 1] = int(min(255, max(0, g)))
            data[y * width * 4 + x * 4 + 2] = int(min(255, max(0, b)))
            data[y * width * 4 + x * 4 + 3] = 255
    
    return data

def check_pixel_in_rect(nx: float, ny: float) -> bool:
    """Check if pixel is within rounded rectangle."""
    corner_r = 0.38
    ax, ay = abs(nx), abs(ny)
    
    if ax > 1.0 - corner_r and ay > 1.0 - corner_r:
        cdist = math.sqrt((ax - (1.0 - corner_r)) ** 2 + (ay - (1.0 - corner_r)) ** 2)
        if cdist > corner_r:
            return False
    elif ax > 1.0 or ay > 1.0:
        return False
    
    return True

def check_in_monitor(nx: float, ny: float, mon_w: float, mon_h: float, 
                     mon_cx: float, mon_cy: float, mon_border: float) -> Tuple[bool, bool, bool, bool, float, float, float]:
    """Check if pixel is within monitor components."""
    mon_x = (nx - mon_cx) / mon_w
    mon_y = (ny - mon_cy) / mon_h
    
    in_monitor = abs(mon_x) < 1.0 and abs(mon_y) < 1.0
    in_screen = abs(mon_x) < 1.0 - mon_border and abs(mon_y) < 1.0 - mon_border
    
    stand_width = 0.08
    stand_height = 0.15
    stand_base_width = 0.18
    stand_base_height = 0.06
    
    stand_in = abs(nx - mon_cx) < stand_width and ny > mon_cy + mon_h and ny < mon_cy + mon_h + stand_height
    stand_base = abs(nx - mon_cx) < stand_base_width and ny > mon_cy + mon_h + stand_height and ny < mon_cy + mon_h + stand_height + stand_base_height
    
    return in_monitor, in_screen, stand_in, stand_base, mon_x, mon_y, 0.0

def get_wave_intensity(sx_norm: float, mon_x: float, mon_y: float, mon_border: float) -> Tuple[float, bool, bool]:
    """Calculate waveform intensity at given position."""
    screen_nx = mon_x / (1.0 - mon_border)
    screen_ny = mon_y / (1.0 - mon_border)
    
    wave_y = 0.0
    in_wave = False
    
    wave_segments = [
        (0.1, 0.25, 0.15, 1.0),
        (0.3, 0.38, 0.08, -1.0),
        (0.38, 0.42, 0.04, 1.0),
        (0.42, 0.50, 0.08, -1.0),
        (0.55, 0.70, 0.15, 1.0),
        (0.75, 0.90, 0.15, 1.0)
    ]
    
    for seg in wave_segments:
        start, end, width, _ = seg
        if sx_norm >= start and sx_norm < end:
            wave_y = math.sin((sx_norm - start) / width * math.pi) * 0.3 * 1.0
            in_wave = True
            break
    
    if in_wave:
        wave_dist = abs(screen_ny - wave_y)
        line_w = 0.06
        
        if wave_dist < line_w:
            intensity = 1.0 - wave_dist / line_w
            glow_intensity = 0.5 if wave_dist < line_w * 0.4 else 0.0
            
            return intensity, True, bool(glow_intensity)
    
    # Grid lines
    grid_x = (sx_norm * 8) % 1.0
    grid_y = ((screen_ny + 1) / 2 * 6) % 1.0
    
    if grid_x < 0.03 or grid_y < 0.03:
        return 0.0, False, False
    
    return 0.0, False, False

def create_png(width: int, height: int, filepath: str) -> None:
    """Create a highly optimized macOS utility-style app icon PNG."""
    cx, cy = width / 2, height / 2
    radius = width * 0.43
    mon_w, mon_h = 0.55, 0.4
    mon_cx, mon_cy = 0.0, -0.08
    mon_border = 0.08 / min(mon_w, mon_h)
    
    # Generate background once
    bg_data = generate_background(width, height)
    
    # Main pixel generation
    raw_data = b""
    for y in range(height):
        # PNG filter byte
        line = bytearray([0])
        
        for x in range(width):
            nx = (x - cx) / radius
            ny = (y - cy) / radius
            
            if not check_pixel_in_rect(nx, ny):
                line.extend([0, 0, 0, 0])
                continue
            
            # Get background pixel
            px = bg_data[y * width * 4 + x * 4 : y * width * 4 + x * 4 + 4]
            
            # Check if pixel is in monitor or stand
            in_monitor, in_screen, stand_in, stand_base, mon_x, mon_y, _ = check_in_monitor(
                nx, ny, mon_w, mon_h, mon_cx, mon_cy, mon_border
            )
            
            if in_monitor and not in_screen:
                # Monitor bezel
                line.extend(px)
            elif in_screen:
                # Screen with waveform
                sx_norm = (mon_x + 1) / 2
                intensity, in_wave, glow = get_wave_intensity(sx_norm, mon_x, mon_y, mon_border)
                
                if in_wave or glow:
                    px[0] = int(min(255, px[0] + intensity * 50 + (1.0 if glow else 0.0) * 30))
                    px[1] = int(min(255, px[1] + intensity * 240 + (1.0 if glow else 0.0) * 50))
                    px[2] = int(min(255, px[2] + intensity * 200 + (1.0 if glow else 0.0) * 50))
                else:
                    # Check grid lines
                    grid_x = (sx_norm * 8) % 1.0
                    screen_nx = mon_x / (1.0 - mon_border)
                    screen_ny = mon_y / (1.0 - mon_border)
                    grid_y = ((screen_ny + 1) / 2 * 6) % 1.0
                    
                    if grid_x < 0.03 or grid_y < 0.03:
                        px[0] = min(255, px[0] + 8)
                        px[1] = min(255, px[1] + 10)
                        px[2] = min(255, px[2] + 15)
                
                line.extend(px)
            elif stand_in or stand_base:
                # Stand
                line.extend(px)
            else:
                # Background pixel already in px
                line.extend(px)
        
        raw_data += bytes(line)
    
    # Encode PNG efficiently
    def make_chunk(chunk_type: bytes, data: bytes) -> bytes:
        chunk = chunk_type + data
        return struct.pack(">I", len(data)) + chunk + struct.pack(">I", zlib.crc32(chunk) & 0xFFFFFFFF)
    
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    png = b"\x89PNG\r\n\x1a\n"
    png += make_chunk(b"IHDR", ihdr)
    png += make_chunk(b"IDAT", zlib.compress(raw_data, 9))
    png += make_chunk(b"IEND", b"")
    
    with open(filepath, "wb") as f:
        f.write(png)

def validate_size(size: int) -> None:
    """Validate icon size."""
    if size < 16 or size > 1024:
        raise ValueError(f"Invalid icon size: {size}. Must be between 16 and 1024.")
    if size & (size - 1):
        raise ValueError(f"Invalid icon size: {size}. Must be a power of 2.")

def validate_iconutil() -> None:
    """Validate iconutil is installed."""
    if not shutil.which("iconutil"):
        raise RuntimeError(
            "iconutil no está instalado. "
            "Instálalo con: xcode-select --install"
        )

@lru_cache(maxsize=1)
def get_required_sizes() -> Dict[str, int]:
    """Get required icon sizes from macOS specifications."""
    return {
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

def should_regenerate_icon(filepath: Path) -> bool:
    """Check if icon needs regeneration based on file info."""
    return not filepath.exists() or \
           filepath.stat().st_mtime < os.path.getmtime(os.path.dirname(__file__))

def cleanup_iconset(iconset_dir: Path) -> None:
    """Remove iconset directory after conversion."""
    import shutil
    if iconset_dir.exists():
        shutil.rmtree(iconset_dir)

def convert_to_icns(iconset_dir: Path, icns_path: Path) -> None:
    """Convert iconset to .icns file."""
    validate_iconutil()
    
    result = subprocess.run(
        ["iconutil", "-c", "icns", str(iconset_dir), "-o", str(icns_path)],
        capture_output=True, text=True
    )
    
    if result.returncode != 0:
        raise RuntimeError(f"iconutil failed: {result.stderr}")

def main() -> int:
    """Main execution function."""
    try:
        start_time = time.time()
        
        logger.info("Starting SysMac icon generation...")
        ICONSET_DIR.mkdir(parents=True, exist_ok=True)
        
        sizes = get_required_sizes()
        generated_count = 0
        
        logger.info("Generating icon sizes...")
        for name, size in sizes.items():
            validate_size(size)
            filepath = ICONSET_DIR / name
            
            if not should_regenerate_icon(filepath):
                logger.debug(f"Skipping existing: {name} ({size}x{size})")
                continue
            
            logger.info(f"  {name} ({size}x{size})")
            create_png(size, size, str(filepath))
            generated_count += 1
        
        if generated_count == 0:
            logger.warning("All icons already exist. Nothing to regenerate.")
        
        # Convert to .icns
        icns_path = ICON_DIR / "AppIcon.icns"
        logger.info(f"Converting to {icns_path}...")
        
        try:
            convert_to_icns(ICONSET_DIR, icns_path)
            cleanup_iconset(ICONSET_DIR)
            
            elapsed = time.time() - start_time
            size_kb = icns_path.stat().st_size / 1024
            
            logger.info(f"Done! Generated {generated_count} icons.")
            logger.info(f"Final icon size: {size_kb:.0f} KB")
            logger.info(f"Total time: {elapsed:.2f} seconds")
            
            return 0
            
        except RuntimeError as e:
            logger.error(f"Failed to convert icon: {e}")
            logger.info("Iconset preserved for manual inspection")
            return 1
    
    except ValueError as e:
        logger.error(f"Configuration error: {e}")
        return 1
    except RuntimeError as e:
        logger.error(f"Runtime error: {e}")
        return 1
    except Exception as e:
        logger.error(f"Unexpected error: {e}", exc_info=True)
        return 1

if __name__ == "__main__":
    sys.exit(main())