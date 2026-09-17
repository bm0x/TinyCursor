import zlib, struct, os
from collections import deque

def load_png(filename):
    with open(filename, 'rb') as f:
        f.read(8)
        idat = b''
        w = h = 0
        while True:
            hdr = f.read(8)
            if not hdr: break
            length, chunk_type = struct.unpack('>I4s', hdr)
            data = f.read(length)
            f.read(4)
            if chunk_type == b'IHDR':
                w, h = struct.unpack('>II', data[:8])
            elif chunk_type == b'IDAT':
                idat += data
            elif chunk_type == b'IEND':
                break

    raw = zlib.decompress(idat)
    stride = 1 + w * 4
    prev = bytearray(w * 4)
    pixels = bytearray(w * h * 4)
    for y in range(h):
        f_t = raw[y * stride]
        row = bytearray(raw[y * stride + 1 : y * stride + stride])
        curr = bytearray(w * 4)
        if f_t == 0:
            curr = row
        elif f_t == 1:
            for x in range(w * 4):
                curr[x] = (row[x] + (curr[x - 4] if x >= 4 else 0)) & 0xFF
        elif f_t == 2:
            for x in range(w * 4):
                curr[x] = (row[x] + prev[x]) & 0xFF
        elif f_t == 3:
            for x in range(w * 4):
                curr[x] = (row[x] + (((curr[x - 4] if x >= 4 else 0) + prev[x]) >> 1)) & 0xFF
        elif f_t == 4:
            for x in range(w * 4):
                a = curr[x - 4] if x >= 4 else 0
                b = prev[x]
                c = prev[x - 4] if x >= 4 else 0
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pr = a if pa <= pb and pa <= pc else (b if pb <= pc else c)
                curr[x] = (row[x] + pr) & 0xFF
        prev = curr
        pixels[y * w * 4 : (y + 1) * w * 4] = curr
    return w, h, pixels

def fill_interior_pure_white(w, h, pixels):
    # Flood-fill from outer image boundaries to find all exterior background pixels
    visited = bytearray(w * h)
    q = deque()
    for x in range(w):
        q.append((x, 0)); visited[0 * w + x] = 1
        q.append((x, h - 1)); visited[(h - 1) * w + x] = 1
    for y in range(h):
        q.append((0, y)); visited[y * w + 0] = 1
        q.append((w - 1, y)); visited[y * w + w - 1] = 1

    while q:
        cx, cy = q.popleft()
        for nx, ny in ((cx + 1, cy), (cx - 1, cy), (cx, cy + 1), (cx, cy - 1)):
            if 0 <= nx < w and 0 <= ny < h:
                n_idx = ny * w + nx
                if not visited[n_idx]:
                    idx = n_idx * 4
                    a = pixels[idx + 3]
                    r = pixels[idx]
                    # Dark border threshold
                    is_border = (a > 180 and r < 60)
                    if not is_border:
                        visited[n_idx] = 1
                        q.append((nx, ny))

    # All unvisited, non-border pixels inside the cursor become SOLID PURE WHITE
    filled_pixels = bytearray(pixels)
    for y in range(h):
        for x in range(w):
            n_idx = y * w + x
            idx = n_idx * 4
            if not visited[n_idx]:
                r = pixels[idx]
                a = pixels[idx + 3]
                is_border = (a > 180 and r < 60)
                if not is_border:
                    # Solid pure white with full opacity
                    filled_pixels[idx] = 255
                    filled_pixels[idx + 1] = 255
                    filled_pixels[idx + 2] = 255
                    filled_pixels[idx + 3] = 255

    return filled_pixels

def sample_area(pixels, src_w, src_h, x0, y0, x1, y1):
    r_sum = g_sum = b_sum = a_sum = total_weight = 0.0
    start_x = max(0, int(x0))
    end_x = min(src_w, int(x1) + 1)
    start_y = max(0, int(y0))
    end_y = min(src_h, int(y1) + 1)
    for sy in range(start_y, end_y):
        overlap_y = max(0.0, min(float(sy + 1), y1) - max(float(sy), y0))
        if overlap_y <= 0: continue
        for sx in range(start_x, end_x):
            overlap_x = max(0.0, min(float(sx + 1), x1) - max(float(sx), x0))
            if overlap_x <= 0: continue
            weight = overlap_x * overlap_y
            idx = (sy * src_w + sx) * 4
            r = pixels[idx]
            g = pixels[idx + 1]
            b = pixels[idx + 2]
            a = pixels[idx + 3]
            r_sum += r * (a / 255.0) * weight
            g_sum += g * (a / 255.0) * weight
            b_sum += b * (a / 255.0) * weight
            a_sum += a * weight
            total_weight += weight
    if total_weight > 0 and a_sum > 0:
        out_a = (a_sum / total_weight)
        alpha_norm = (out_a / 255.0)
        out_r = (r_sum / total_weight) / alpha_norm
        out_g = (g_sum / total_weight) / alpha_norm
        out_b = (b_sum / total_weight) / alpha_norm
        return int(min(255, max(0, out_r))), int(min(255, max(0, out_g))), int(min(255, max(0, out_b))), int(min(255, max(0, out_a)))
    return 0, 0, 0, 0

def resample_crop(pixels, src_w, src_h, src_box, dst_w, dst_h):
    bx0, by0, bx1, by1 = src_box
    box_w = bx1 - bx0
    box_h = by1 - by0
    out = bytearray(dst_w * dst_h * 4)
    for dy in range(dst_h):
        sy0 = by0 + (dy / float(dst_h)) * box_h
        sy1 = by0 + ((dy + 1) / float(dst_h)) * box_h
        for dx in range(dst_w):
            sx0 = bx0 + (dx / float(dst_w)) * box_w
            sx1 = bx0 + ((dx + 1) / float(dst_w)) * box_w
            r, g, b, a = sample_area(pixels, src_w, src_h, sx0, sy0, sx1, sy1)
            idx = (dy * dst_w + dx) * 4
            out[idx] = r
            out[idx + 1] = g
            out[idx + 2] = b
            out[idx + 3] = a
    return out

def make_ico(png_data):
    icondir = struct.pack('<HHH', 0, 1, 1)
    w, h = 32, 32
    offset = 6 + 16
    direntry = struct.pack('<BBBBHHII', w, h, 0, 0, 1, 32, len(png_data), offset)
    return icondir + direntry + png_data

def encode_png_rgba(width, height, data):
    raw_lines = bytearray()
    for y in range(height):
        raw_lines.append(0)
        raw_lines.extend(data[y * width * 4 : (y + 1) * width * 4])
    compressed = zlib.compress(bytes(raw_lines), 9)

    def chunk(chunk_type, chunk_data):
        crc = zlib.crc32(chunk_type + chunk_data) & 0xFFFFFFFF
        return struct.pack('>I', len(chunk_data)) + chunk_type + chunk_data + struct.pack('>I', crc)

    ihdr = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
    out = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', ihdr) + chunk(b'IDAT', compressed) + chunk(b'IEND', b'')
    return out

def main():
    src_w, src_h, raw_pixels = load_png('modern_white.png')

    # Fill interior with solid pure white
    pixels = fill_interior_pure_white(src_w, src_h, raw_pixels)

    arrow_crop = (72, 20, 297, 306)
    hand_crop = (330, 20, 597, 306)

    arrow_rgba = resample_crop(pixels, src_w, src_h, arrow_crop, 64, 64)
    hand_rgba = resample_crop(pixels, src_w, src_h, hand_crop, 64, 64)

    os.makedirs('assets', exist_ok=True)

    arrow_png = encode_png_rgba(64, 64, arrow_rgba)
    hand_png = encode_png_rgba(64, 64, hand_rgba)
    with open('assets/arrow.png', 'wb') as f: f.write(arrow_png)
    with open('assets/hand.png', 'wb') as f: f.write(hand_png)

    icon_rgba = resample_crop(pixels, src_w, src_h, arrow_crop, 32, 32)
    icon_png = encode_png_rgba(32, 32, icon_rgba)
    icon_data = make_ico(icon_png)
    with open('assets/app_icon.ico', 'wb') as f: f.write(icon_data)
    with open('assets/app_icon.png', 'wb') as f: f.write(icon_png)

    def to_argb_premul_u32(rgba_bytes, count):
        arr = []
        for i in range(count):
            r = rgba_bytes[i * 4]
            g = rgba_bytes[i * 4 + 1]
            b = rgba_bytes[i * 4 + 2]
            a = rgba_bytes[i * 4 + 3]
            af = a / 255.0
            pr = int(r * af)
            pg = int(g * af)
            pb = int(b * af)
            val = (a << 24) | (pr << 16) | (pg << 8) | pb
            arr.append(f"0x{val:08X}")
        return arr

    arrow_u32 = to_argb_premul_u32(arrow_rgba, 64 * 64)
    hand_u32 = to_argb_premul_u32(hand_rgba, 64 * 64)

    with open('src/platform/windows/assets.rs', 'w', encoding='utf-8') as f:
        f.write("//! Precompiled Modern White Cursor Sprites with Solid White Interior.\n\n")
        f.write("pub const SPRITE_DIM: usize = 64;\n\n")
        f.write(f"pub const ARROW_HOTSPOT_X: f32 = 1.4;\n")
        f.write(f"pub const ARROW_HOTSPOT_Y: f32 = 1.1;\n\n")
        f.write(f"pub const HAND_HOTSPOT_X: f32 = 23.7;\n")
        f.write(f"pub const HAND_HOTSPOT_Y: f32 = 1.0;\n\n")

        f.write("pub static ARROW_PIXELS: [u32; 64 * 64] = [\n")
        for i in range(0, len(arrow_u32), 8):
            f.write("    " + ", ".join(arrow_u32[i:i+8]) + ",\n")
        f.write("];\n\n")

        f.write("pub static HAND_PIXELS: [u32; 64 * 64] = [\n")
        for i in range(0, len(hand_u32), 8):
            f.write("    " + ", ".join(hand_u32[i:i+8]) + ",\n")
        f.write("];\n\n")

        f.write(f"pub static ICON_ICO_BYTES: &[u8] = include_bytes!(\"../../../assets/app_icon.ico\");\n")
        f.write(f"pub static ICON_PNG_BYTES: &[u8] = include_bytes!(\"../../../assets/app_icon.png\");\n")

    print("Regenerated Modern White sprites with solid pure white interior!")

if __name__ == '__main__':
    main()
