# /// script
# requires-python = ">=3.10"
# dependencies = ["pillow"]
# ///
"""Generate Tauri app icons from source logo."""

from pathlib import Path
from PIL import Image
import io

def main():
    src = Path("src/assets/logo.png")
    dst = Path("src-tauri/icons")
    dst.mkdir(parents=True, exist_ok=True)

    # Open and convert to RGBA
    img = Image.open(src).convert("RGBA")
    
    # Make it square by adding padding
    size = max(img.size)
    square = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    offset = ((size - img.width) // 2, (size - img.height) // 2)
    square.paste(img, offset)

    # Generate required sizes
    sizes = {
        "32x32.png": 32,
        "128x128.png": 128,
        "icon.png": 512,
        "icon@2x.png": 512,
    }

    for name, s in sizes.items():
        resized = square.resize((s, s), Image.Resampling.LANCZOS)
        resized.save(dst / name, "PNG")
        print(f"Generated {name} ({s}x{s})")

    # Generate ICO file with multiple sizes
    ico_sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    ico_images = [square.resize(s, Image.Resampling.LANCZOS) for s in ico_sizes]
    ico_images[0].save(dst / "icon.ico", format="ICO", sizes=ico_sizes)
    print("Generated icon.ico")

if __name__ == "__main__":
    main()
