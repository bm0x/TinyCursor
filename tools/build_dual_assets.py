from PIL import Image
import os

SPRITE_DIM = 64

def process_and_fit(img_path, target_max_dim=50):
    img = Image.open(img_path).convert('RGBA')
    w, h = img.size
    scale = target_max_dim / max(w, h)
    nw, nh = int(round(w * scale)), int(round(h * scale))
    resized = img.resize((nw, nh), Image.Resampling.LANCZOS)
    
    # Create 64x64 canvas
    canvas = Image.new('RGBA', (SPRITE_DIM, SPRITE_DIM), (0, 0, 0, 0))
    # Center it with a slight top-left bias (hotspot friendly)
    ox = (SPRITE_DIM - nw) // 2
    oy = (SPRITE_DIM - nh) // 2
    canvas.paste(resized, (ox, oy), resized)
    return canvas, ox, oy, nw, nh

def to_premul_argb_array(canvas):
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

# Process White and Black Arrows with IDENTICAL bounding boxes
w_arrow, ox_a, oy_a, nw_a, nh_a = process_and_fit('assets/white_arrow_clean.png', target_max_dim=52)
b_arrow, _, _, _, _ = process_and_fit('assets/black_arrow_clean.png', target_max_dim=52)
w_arrow.save('assets/white_arrow_64.png')
b_arrow.save('assets/black_arrow_64.png')

# Process White and Black Hands with IDENTICAL bounding boxes
w_hand, ox_h, oy_h, nw_h, nh_h = process_and_fit('assets/white_hand_clean.png', target_max_dim=50)
b_hand, _, _, _, _ = process_and_fit('assets/black_hand_clean.png', target_max_dim=50)
w_hand.save('assets/white_hand_64.png')
b_hand.save('assets/black_hand_64.png')

w_arr_pixels = to_premul_argb_array(w_arrow)
b_arr_pixels = to_premul_argb_array(b_arrow)
w_hnd_pixels = to_premul_argb_array(w_hand)
b_hnd_pixels = to_premul_argb_array(b_hand)

def format_rust_array(name, arr):
    lines = [f"pub static {name}: [u32; 64 * 64] = ["]
    for i in range(0, len(arr), 8):
        chunk = arr[i : i + 8]
        hex_str = ", ".join(f"0x{v:08X}" for v in chunk)
        lines.append(f"    {hex_str},")
    lines.append("];\n")
    return "\n".join(lines)

rust_content = f"""//! Precompiled Dual-Theme (White & Black) Cursor Sprites for TinyCursor.
//! Extracted with sub-pixel precision from the user's custom cursor canvas.

pub const SPRITE_DIM: usize = 64;

// Hotspot coordinates (center/tip reference)
pub const ARROW_HOTSPOT_X: f32 = {ox_a + nw_a * 0.5:.1f};
pub const ARROW_HOTSPOT_Y: f32 = {oy_a + nh_a * 0.5:.1f};

pub const HAND_HOTSPOT_X: f32 = {ox_h + nw_h * 0.36:.1f};
pub const HAND_HOTSPOT_Y: f32 = {oy_h + 2.0:.1f};

{format_rust_array('WHITE_ARROW_PIXELS', w_arr_pixels)}
{format_rust_array('BLACK_ARROW_PIXELS', b_arr_pixels)}
{format_rust_array('WHITE_HAND_PIXELS', w_hnd_pixels)}
{format_rust_array('BLACK_HAND_PIXELS', b_hnd_pixels)}

pub static ICON_ICO_BYTES: &[u8] = include_bytes!("../../../assets/app_icon.ico");
"""

with open('src/platform/windows/assets.rs', 'w', encoding='utf-8') as f:
    f.write(rust_content)

print(f"Generated src/platform/windows/assets.rs with White & Black Arrow and Hand sprites and App Icon!")
