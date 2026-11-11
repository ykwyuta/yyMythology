"""Pack the YYMythology brand source images for Windows and the web (Pillow)."""

from pathlib import Path

from PIL import Image, ImageOps


ROOT = Path(__file__).resolve().parents[1]
BRAND = ROOT / "assets" / "brand"
WEB = ROOT / "web"
LANCZOS = Image.Resampling.LANCZOS


def main():
    with Image.open(BRAND / "icon.png") as source:
        icon = ImageOps.fit(source.convert("RGB"), (512, 512), LANCZOS)
    for name, size in (("icon-192.png", 192), ("icon-512.png", 512), ("apple-touch-icon.png", 180)):
        icon.resize((size, size), LANCZOS).save(WEB / name, optimize=True)
    icon.save(
        ROOT / "assets" / "app_icon.ico",
        sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )

    with Image.open(BRAND / "startup.png") as source:
        scene = source.convert("RGB")
    scene.thumbnail((1600, 1000), LANCZOS)
    scene.save(WEB / "key-art.jpg", quality=84, optimize=True, progressive=True)
    title = ImageOps.fit(scene, (344, 122), LANCZOS, centering=(0.5, 0.1)).convert("RGBA")
    (BRAND / "title-art.bgra").write_bytes(title.tobytes("raw", "BGRA"))


if __name__ == "__main__":
    main()
