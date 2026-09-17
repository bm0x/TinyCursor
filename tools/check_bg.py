from PIL import Image

img = Image.open('assets/white_arrow_raw.png').convert('RGB')
w, h = img.size
print("Top-left:", img.getpixel((0, 0)))
print("Top-right:", img.getpixel((w-1, 0)))
print("Bottom-left:", img.getpixel((0, h-1)))
print("Bottom-right:", img.getpixel((w-1, h-1)))
print("Sample background (10, 10):", img.getpixel((10, 10)))
