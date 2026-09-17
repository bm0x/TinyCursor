from PIL import Image

img = Image.open('assets/white_arrow_clean.png').convert('RGBA')

# Let's test rotating by:
# 1. 0 deg (as is)
# 2. -135 deg (tip points top-left)
# 3. 180 deg (tip points left)
# 4. -90 deg (tip points up)

rotations = {
    'rot_as_is.png': img,
    'rot_top_left.png': img.rotate(-135, resample=Image.Resampling.BICUBIC, expand=True),
    'rot_left.png': img.rotate(180, resample=Image.Resampling.BICUBIC, expand=True),
    'rot_up.png': img.rotate(90, resample=Image.Resampling.BICUBIC, expand=True),
}

for name, r in rotations.items():
    r.save(f"assets/{name}")
    print(f"Saved assets/{name} size: {r.size}")
