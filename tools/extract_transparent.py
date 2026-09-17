from PIL import Image, ImageFilter
from collections import deque

def extract_transparent_cursor(img_path, is_white_cursor=True, crop_box=None):
    img = Image.open(img_path).convert('RGBA')
    if crop_box:
        img = img.crop(crop_box)
    w, h = img.size
    pixels = img.load()

    # Visited grid for flood fill
    visited = [[False] * h for _ in range(w)]
    queue = deque()

    # Add all border pixels to queue
    for x in range(w):
        queue.append((x, 0))
        queue.append((x, h - 1))
        visited[x][0] = True
        visited[x][h - 1] = True
    for y in range(h):
        queue.append((0, y))
        queue.append((w - 1, y))
        visited[0][y] = True
        visited[w - 1][y] = True

    # Background detection function
    def is_bg(r, g, b):
        lum = 0.299 * r + 0.587 * g + 0.114 * b
        if is_white_cursor:
            # Background is light gray ~210..235, but NOT white (>250) and NOT bevel edge (<185)
            # The grid line is ~210..215. The bevel edge is <= 185.
            return 200 <= lum <= 245
        else:
            # For black cursor, background is light gray (> 180). Cursor is very dark (< 80).
            return lum >= 170

    bg_mask = [[False] * h for _ in range(w)]

    while queue:
        x, y = queue.popleft()
        r, g, b, _ = pixels[x, y]
        if is_bg(r, g, b):
            bg_mask[x][y] = True
            for dx, dy in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
                nx, ny = x + dx, y + dy
                if 0 <= nx < w and 0 <= ny < h and not visited[nx][ny]:
                    visited[nx][ny] = True
                    queue.append((nx, ny))

    # Apply alpha: 0 for background, 255 for cursor
    res = Image.new('RGBA', (w, h), (0, 0, 0, 0))
    res_pixels = res.load()
    min_x, min_y, max_x, max_y = w, h, 0, 0
    for x in range(w):
        for y in range(h):
            if not bg_mask[x][y]:
                res_pixels[x, y] = pixels[x, y]
                min_x = min(min_x, x)
                min_y = min(min_y, y)
                max_x = max(max_x, x)
                max_y = max(max_y, y)

    # Tight crop
    if min_x < max_x and min_y < max_y:
        # add 2px margin
        box = (max(0, min_x - 2), max(0, min_y - 2), min(w, max_x + 3), min(h, max_y + 3))
        res = res.crop(box)

    return res

# Extract both
white_arrow = extract_transparent_cursor('assets/white_arrow_raw.png', is_white_cursor=True)
white_arrow.save('assets/extracted_white_arrow.png')
print(f"Saved extracted_white_arrow.png: {white_arrow.size}")

black_arrow = extract_transparent_cursor('assets/black_arrow_raw.png', is_white_cursor=False)
black_arrow.save('assets/extracted_black_arrow.png')
print(f"Saved extracted_black_arrow.png: {black_arrow.size}")

white_hand = extract_transparent_cursor('assets/white_hand_raw.png', is_white_cursor=True)
white_hand.save('assets/extracted_white_hand.png')
print(f"Saved extracted_white_hand.png: {white_hand.size}")

black_hand = extract_transparent_cursor('assets/black_hand_raw.png', is_white_cursor=False)
black_hand.save('assets/extracted_black_hand.png')
print(f"Saved extracted_black_hand.png: {black_hand.size}")
