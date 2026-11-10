"""Pack the generated transparent atlases for the native and browser renderers.

Run python tools/import_mythology_art.py from any directory. The source images
and the exact image generation prompts are in assets/mythology.
"""
import json
from pathlib import Path

from PIL import Image, ImageOps

from import_period_art import bgra

ASSETS = Path(__file__).resolve().parents[1] / 'assets' / 'mythology'


def cuts(mask, count):
    """Find transparent gutters near the nominal boundaries of a grid."""
    width, height = mask.size
    step = width / count
    boundaries = [0]
    for i in range(1, count):
        center = round(i * step)
        radius = max(1, round(step * 0.13))
        def cost(x):
            pixels = mask.crop((x - 1, 0, x + 2, height)).histogram()[255]
            return pixels + abs(x - center) / radius * height * 0.05
        boundaries.append(min(range(center - radius, center + radius + 1), key=cost))
    return [*boundaries, width]


def sprites(path, cols, rows, size):
    image = Image.open(path)
    if 'A' not in image.getbands() or image.getchannel('A').getextrema()[0] != 0:
        raise ValueError(f'{path.name} needs an actual transparent alpha channel')
    image = image.convert('RGBA')
    alpha = image.getchannel('A').point(lambda a: 255 if a > 40 else 0)
    xs = cuts(alpha, cols)
    ys_by_column = [
        cuts(alpha.crop((left, 0, right, image.height)).transpose(Image.Transpose.TRANSPOSE), rows)
        for left, right in zip(xs, xs[1:])
    ]
    for row in range(rows):
        for col in range(cols):
            y = ys_by_column[col]
            box = (xs[col], y[row], xs[col + 1], y[row + 1])
            tile = image.crop(box)
            bounds = alpha.crop(box).getbbox()
            if bounds is None or min(bounds[2] - bounds[0], bounds[3] - bounds[1]) < 32:
                raise ValueError(f'{path.name} has an empty slot at row {row + 1}, column {col + 1}')
            tile = ImageOps.contain(tile.crop(bounds), (size - 10, size - 10), Image.Resampling.LANCZOS)
            out = Image.new('RGBA', (size, size))
            out.paste(tile, ((size - tile.width) // 2, size - 5 - tile.height))
            yield out


def main():
    manifest = json.loads((ASSETS / 'generation.json').read_text(encoding='utf-8'))
    for atlas in manifest['atlases']:
        output = ASSETS / atlas['output']
        size = atlas['sprite_size']
        data = bytearray()
        for sheet in atlas['sheets']:
            path = ASSETS / (sheet['name'] + '.png')
            for tile in sprites(path, sheet['columns'], sheet['rows'], size):
                data.extend(bgra(tile))
        if len(data) != atlas['sprite_count'] * size * size * 4:
            raise ValueError(f'Wrong sprite count for {output.name}')
        output.write_bytes(data)


if __name__ == '__main__':
    main()
