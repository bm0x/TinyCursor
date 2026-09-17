from PIL import Image
import os

SPRITE_DIM = 64

# Specification for each cursor: target dimension in 64x64 canvas, and hotspot type
cursors_spec = {
    'arrow': {'target_dim': 52, 'hotspot_type': 'arrow_tip'},
    'hand': {'target_dim': 50, 'hotspot_type': 'hand_tip'},
    'ibeam': {'target_dim': 50, 'hotspot_type': 'center'},
    'crosshair': {'target_dim': 50, 'hotspot_type': 'center'},
    'resize_ns': {'target_dim': 50, 'hotspot_type': 'center'},
    'resize_we': {'target_dim': 50, 'hotspot_type': 'center'},
    'resize_nwse': {'target_dim': 50, 'hotspot_type': 'center'},
    'resize_nesw': {'target_dim': 50, 'hotspot_type': 'center'},
    'move': {'target_dim': 50, 'hotspot_type': 'center'},
    'zoom_in': {'target_dim': 50, 'hotspot_type': 'lens'},
    'zoom_out': {'target_dim': 50, 'hotspot_type': 'lens'},
    'wait': {'target_dim': 50, 'hotspot_type': 'center'},
    'help': {'target_dim': 52, 'hotspot_type': 'arrow_tip'},
    'unavailable': {'target_dim': 50, 'hotspot_type': 'center'},
}

def fit_to_64(img_path, target_dim, hotspot_type):
    img = Image.open(img_path).convert('RGBA')
    w, h = img.size
    scale = target_dim / max(w, h)
    nw, nh = int(round(w * scale)), int(round(h * scale))
    resized = img.resize((nw, nh), Image.Resampling.LANCZOS)
    canvas = Image.new('RGBA', (SPRITE_DIM, SPRITE_DIM), (0, 0, 0, 0))
    
    ox = (SPRITE_DIM - nw) // 2
    oy = (SPRITE_DIM - nh) // 2
    canvas.paste(resized, (ox, oy), resized)
    
    # Calculate hotspot
    if hotspot_type == 'arrow_tip':
        hx = ox + nw * 0.50
        hy = oy + nh * 0.50
    elif hotspot_type == 'hand_tip':
        hx = ox + nw * 0.36
        hy = oy + 2.0
    elif hotspot_type == 'center':
        hx = SPRITE_DIM / 2.0
        hy = SPRITE_DIM / 2.0
    elif hotspot_type == 'lens':
        hx = ox + nw * 0.38
        hy = oy + nh * 0.38
    else:
        hx = SPRITE_DIM / 2.0
        hy = SPRITE_DIM / 2.0
        
    return canvas, hx, hy

def to_premul(canvas):
    w, h = canvas.size
    pixels = canvas.load()
    arr = []
    for y in range(h):
        for x in range(w):
            r, g, b, a = pixels[x, y]
            if a == 0:
                arr.append(0)
            else:
                pr = (r * a + 127) // 255
                pg = (g * a + 127) // 255
                pb = (b * a + 127) // 255
                val = (a << 24) | (pr << 16) | (pg << 8) | pb
                arr.append(val)
    return arr

def format_rust_array(name, arr):
    lines = [f"pub static {name}: [u32; 64 * 64] = ["]
    for i in range(0, len(arr), 8):
        chunk = arr[i : i + 8]
        hex_str = ", ".join(f"0x{v:08X}" for v in chunk)
        lines.append(f"    {hex_str},")
    lines.append("];\n")
    return "\n".join(lines)

rust_parts = [
    "//! Precompiled Full Suite of Dual-Theme (White & Black) Cursor Sprites for TinyCursor.\n",
    "//! Extracted with sub-pixel precision from the user custom cursor canvas.\n\n",
    "pub const SPRITE_DIM: usize = 64;\n\n",
]

for name, spec in cursors_spec.items():
    w_canv, hx, hy = fit_to_64(f"assets/clean/white_{name}.png", spec['target_dim'], spec['hotspot_type'])
    b_canv, _, _ = fit_to_64(f"assets/clean/black_{name}.png", spec['target_dim'], spec['hotspot_type'])
    
    w_arr = to_premul(w_canv)
    b_arr = to_premul(b_canv)
    
    uname = name.upper()
    rust_parts.append(f"pub const {uname}_HOTSPOT_X: f32 = {hx:.1f};\n")
    rust_parts.append(f"pub const {uname}_HOTSPOT_Y: f32 = {hy:.1f};\n\n")
    
    rust_parts.append(format_rust_array(f"WHITE_{uname}_PIXELS", w_arr))
    rust_parts.append(format_rust_array(f"BLACK_{uname}_PIXELS", b_arr))

rust_parts.append('pub static ICON_ICO_BYTES: &[u8] = include_bytes!("../../../assets/app_icon.ico");\n')

with open("src/platform/windows/assets.rs", "w", encoding="utf-8") as f:
    f.writelines(rust_parts)

print(f"Generated src/platform/windows/assets.rs with all {len(cursors_spec)} cursor pairs!")
