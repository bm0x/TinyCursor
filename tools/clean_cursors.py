from PIL import Image
from collections import deque

def clean_cursor(img_path, is_white_cursor=True):
    img = Image.open(img_path).convert('RGBA')
    w, h = img.size
    pixels = img.load()

    # Determine if pixel is definitely background
    # Grid background is light grayish: r > 180, g > 180, b > 180, and not pure white (if white cursor)
    def is_foreground(r, g, b, a):
        if a == 0:
            return False
        lum = 0.299 * r + 0.587 * g + 0.114 * b
        if is_white_cursor:
            # Foreground is either white body (r>240, g>240, b>240) OR dark bevel edge (lum < 185)
            # Background is 190 <= lum <= 238
            return lum < 190 or lum > 242
        else:
            # Black cursor is very dark (lum < 150)
            return lum < 150

    # 1. Binary foreground map
    fg = [[False] * h for _ in range(w)]
    for x in range(w):
        for y in range(h):
            r, g, b, a = pixels[x, y]
            if is_foreground(r, g, b, a):
                fg[x][y] = True

    # 2. Find connected components of foreground
    visited = [[False] * h for _ in range(w)]
    components = []
    for x in range(w):
        for y in range(h):
            if fg[x][y] and not visited[x][y]:
                comp = []
                q = deque([(x, y)])
                visited[x][y] = True
                while q:
                    cx, cy = q.popleft()
                    comp.append((cx, cy))
                    for dx, dy in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, 1), (-1, 1), (1, -1)]:
                        nx, ny = cx + dx, cy + dy
                        if 0 <= nx < w and 0 <= ny < h and fg[nx][ny] and not visited[nx][ny]:
                            visited[nx][ny] = True
                            q.append((nx, ny))
                components.append(comp)

    if not components:
        return img

    # Largest component is the cursor
    components.sort(key=len, reverse=True)
    main_comp = set(components[0])

    # If white cursor, fill inside if there were any hollows
    # Build result image
    out = Image.new('RGBA', (w, h), (0, 0, 0, 0))
    out_pixels = out.load()
    min_x, min_y, max_x, max_y = w, h, 0, 0
    for (x, y) in main_comp:
        out_pixels[x, y] = pixels[x, y]
        min_x = min(min_x, x)
        min_y = min(min_y, y)
        max_x = max(max_x, x)
        max_y = max(max_y, y)

    # For white cursor, any pixel inside the convex hull / enclosed by the bevel that is white/light is part of the cursor
    if is_white_cursor:
        # Simple flood fill from outside of bounding box to find true exterior
        ext_visited = [[False] * h for _ in range(w)]
        eq = deque()
        for x in range(w):
            eq.append((x, 0))
            eq.append((x, h - 1))
            ext_visited[x][0] = True
            ext_visited[x][h - 1] = True
        for y in range(h):
            eq.append((0, y))
            eq.append((w - 1, y))
            ext_visited[0][y] = True
            ext_visited[w - 1][y] = True
        
        while eq:
            cx, cy = eq.popleft()
            for dx, dy in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
                nx, ny = cx + dx, cy + dy
                if 0 <= nx < w and 0 <= ny < h and not ext_visited[nx][ny]:
                    if (nx, ny) not in main_comp:
                        # Check if background-like
                        r, g, b, _ = pixels[nx, ny]
                        lum = 0.299 * r + 0.587 * g + 0.114 * b
                        if lum < 245: # not interior white
                            ext_visited[nx][ny] = True
                            eq.append((nx, ny))
        
        # Any pixel not reached by exterior is interior!
        for x in range(w):
            for y in range(h):
                if not ext_visited[x][y]:
                    out_pixels[x, y] = pixels[x, y]
                    min_x = min(min_x, x)
                    min_y = min(min_y, y)
                    max_x = max(max_x, x)
                    max_y = max(max_y, y)

    # Crop to bounds with 4px padding
    pad = 4
    box = (max(0, min_x - pad), max(0, min_y - pad), min(w, max_x + pad), min(h, max_y + pad))
    return out.crop(box)

for name, is_w in [
    ('white_arrow', True),
    ('black_arrow', False),
    ('white_hand', True),
    ('black_hand', False)
]:
    cleaned = clean_cursor(f'assets/{name}_raw.png', is_white_cursor=is_w)
    cleaned.save(f'assets/{name}_clean.png')
    print(f"Saved assets/{name}_clean.png: {cleaned.size}")
