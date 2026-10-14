#!/usr/bin/env python3
"""Generate the original EnterGuard icon; requires Pillow.

The icon is a navy keyboard key with a mint return arrow. It contains no
third-party brand marks. Run from any directory: python assets/generate_icon.py
"""
from pathlib import Path
from PIL import Image, ImageDraw

OUT = Path(__file__).resolve().parent
SIZES = (16, 24, 32, 48, 64, 128, 256)
SCALE = 4
COLORS = {
    "body": "#0B1424",
    "face": "#192A44",
    "edge": "#344963",
    "arrow": "#91F4C8",
}


def render(size: int) -> Image.Image:
    # Supersampling preserves the round key and clean diagonals at tray sizes.
    scale = size * SCALE / 256
    image = Image.new("RGBA", (size * SCALE, size * SCALE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    box = lambda coords: tuple(round(value * scale) for value in coords)
    draw.rounded_rectangle(box((8, 8, 248, 248)), radius=round(54 * scale), fill=COLORS["body"])
    draw.rounded_rectangle(box((8, 8, 248, 230)), radius=round(54 * scale), fill=COLORS["edge"])
    draw.rounded_rectangle(box((12, 12, 244, 226)), radius=round(50 * scale), fill=COLORS["face"])

    # Draw one return-arrow bend and a broad left-pointing arrowhead.
    # Bezier bend: (181, 124) to (153, 152), with a 28-unit radius.
    path = [(181, 76), (181, 124)]
    for step in range(1, 33):
        t = step / 32
        x = (1-t)**2 * 181 + 2*(1-t)*t * 181 + t*t * 153
        y = (1-t)**2 * 124 + 2*(1-t)*t * 152 + t*t * 152
        path.append((x, y))
    path.append((76, 152))
    width = round(23 * scale)
    draw.line([box(p) for p in path], fill=COLORS["arrow"], width=width, joint="curve")
    draw.line([box(p) for p in ((106, 122), (76, 152), (106, 182))],
              fill=COLORS["arrow"], width=width, joint="curve")
    radius = width / 2
    for point in ((181, 76), (76, 152), (106, 122), (106, 182)):
        x, y = box(point)
        draw.ellipse((x-radius, y-radius, x+radius, y+radius), fill=COLORS["arrow"])
    return image.resize((size, size), Image.Resampling.LANCZOS)


SVG = '''<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256" role="img" aria-labelledby="title desc">
  <title id="title">EnterGuard app icon</title>
  <desc id="desc">A rounded navy keyboard key bearing a mint return arrow.</desc>
  <rect x="8" y="8" width="240" height="240" rx="54" fill="#0B1424"/>
  <rect x="8" y="8" width="240" height="222" rx="54" fill="#344963"/>
  <rect x="12" y="12" width="232" height="214" rx="50" fill="#192A44"/>
  <g fill="none" stroke="#91F4C8" stroke-width="23" stroke-linecap="round" stroke-linejoin="round">
    <path d="M181 76 V124 Q181 152 153 152 H76"/>
    <path d="M106 122 L76 152 L106 182"/>
  </g>
</svg>
'''


def main() -> None:
    frames = {size: render(size) for size in SIZES}
    frames[256].save(OUT / "EnterGuard.png")
    frames[256].save(OUT / "EnterGuard.ico", sizes=[(s, s) for s in SIZES],
                     append_images=[frames[s] for s in SIZES if s != 256])
    (OUT / "EnterGuard.svg").write_text(SVG, encoding="utf-8")
    with Image.open(OUT / "EnterGuard.ico") as icon:
        assert icon.ico.sizes() == {(s, s) for s in SIZES}, icon.ico.sizes()
        for size in SIZES:
            assert icon.ico.getimage((size, size)).mode == "RGBA"
    print("Created EnterGuard.ico (16/24/32/48/64/128/256), EnterGuard.png (256), EnterGuard.svg")


if __name__ == "__main__":
    main()
