from PIL import Image

img = Image.open(r'C:\Users\rafy2\Downloads\Gemini_Generated_Image_wsedjqwsedjqwsed.jpg')
w, h = img.size
print(f"Image dimensions: {w}x{h}")

# The image is divided into left (white cursors) and right (black cursors)
# Width is 2400: left is 0..1200, right is 1200..2400.
# Height is 1792: 7 rows -> ~256px per row.

# Let's crop:
# Row 1, Col 1 (White Arrow): approx x: 50..350, y: 50..300
# Row 1, Col 2 (White Hand): approx x: 350..620, y: 50..300
# Row 1, Col 5 (Black Arrow): approx x: 1250..1550, y: 50..300
# Row 1, Col 6 (Black Hand): approx x: 1550..1820, y: 50..300

crops = {
    'white_arrow_raw.png': (60, 60, 340, 320),
    'white_hand_raw.png': (350, 60, 620, 320),
    'black_arrow_raw.png': (1260, 60, 1540, 320),
    'black_hand_raw.png': (1550, 60, 1820, 320),
}

for name, box in crops.items():
    c = img.crop(box)
    c.save(f"assets/{name}")
    print(f"Saved assets/{name} with size {c.size}")
