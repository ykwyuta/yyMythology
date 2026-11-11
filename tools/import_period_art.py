"""Compile imagegen atlases to the engine's premultiplied BGRA format.

Only grid slicing, transparent-bound cropping and size normalization happen here.
Background extraction and all creative changes must be made with imagegen.
Run: python tools/import_period_art.py
"""
from pathlib import Path
import argparse
import json
from PIL import Image, ImageOps, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / 'assets' / 'period'


def cells(path, cols, rows, size, alpha_floor=0, row_edges=None):
    src = Image.open(path).convert('RGBA')
    if alpha_floor:
        # Discard residual near-transparent pixels during sprite normalization.
        alpha = src.getchannel('A').point(
            lambda value: round(255 * max(0, value - alpha_floor) / (255 - alpha_floor))
        )
        src.putalpha(alpha)
    if row_edges is None:
        row_edges = [round(y * src.height / rows) for y in range(rows + 1)]
    if len(row_edges) != rows + 1 or row_edges[0] != 0 or row_edges[-1] != src.height:
        raise ValueError(f'{path}: invalid row boundaries')
    if any(a >= b for a, b in zip(row_edges, row_edges[1:])):
        raise ValueError(f'{path}: row boundaries must increase')
    result = []
    for y in range(rows):
        for x in range(cols):
            tile = src.crop((round(x*src.width/cols), row_edges[y],
                             round((x+1)*src.width/cols), row_edges[y+1]))
            # Ignore barely visible antialiasing when finding the content bounds.
            bounds = tile.getchannel('A').point(lambda a: 255 if a > 24 else 0).getbbox()
            if bounds:
                tile = tile.crop(bounds)
            tile = ImageOps.contain(tile, (size-12, size-12), Image.Resampling.LANCZOS)
            out = Image.new('RGBA', (size, size))
            out.paste(tile, ((size-tile.width)//2, size-6-tile.height))
            result.append(out)
    return result


def bgra(img):
    data = bytearray()
    for r, g, b, a in img.getdata():
        data.extend(((b*a+127)//255, (g*a+127)//255, (r*a+127)//255, a))
    return data


def main():
    manifest = json.loads((DEST / 'generation.json').read_text(encoding='utf-8'))
    preview = Image.new('RGB', (12*112, len(manifest['atlases'])*138), '#10202b')
    draw = ImageDraw.Draw(preview)
    for row, item in enumerate(manifest['atlases']):
        name = item['name']
        tiles = cells(DEST / (name+'.png'), item['columns'], item['rows'], 192)
        (DEST / (name+'.bgra')).write_bytes(b''.join(bgra(t) for t in tiles))
        draw.text((8, row*138+3), name.upper(), fill='#d1bb89')
        for i, tile in enumerate(tiles):
            small = tile.resize((108,108), Image.Resampling.LANCZOS)
            preview.paste(small, (i*112, row*138+23), small)
    preview.save(DEST / 'catalog.jpg', quality=92)
    heroes = cells(DEST / 'heroes.png', 8, 6, 128)[:46]
    (DEST / 'heroes.bgra').write_bytes(b''.join(bgra(t) for t in heroes))
    # Shared command/resource icons also consume period artwork rather than the inherited assets.
    steppe = cells(DEST / 'steppe.png', 4, 3, 160)
    for i, name in enumerate(['worker','infantry','archer','chariot','cavalry']):
        tile = steppe[i]
        tile.save(ROOT / 'assets' / 'units' / (name+'.png'))
        (ROOT / 'assets' / 'units' / (name+'.bgra')).write_bytes(bgra(tile))
    buildings = cells(DEST / 'steppe.png', 4, 3, 192)
    for i, name in enumerate(['capital','town','barracks','farm','fort']):
        tile = buildings[6+i]
        tile.save(ROOT / 'assets' / 'map' / (name+'.png'))
        (ROOT / 'assets' / 'map' / (name+'.bgra')).write_bytes(bgra(tile))
    # One identity for the desktop executable, browser installation and touch icon.
    icon = Image.new('RGBA', (512,512), '#091923')
    mark = ImageOps.contain(buildings[6], (448,448), Image.Resampling.LANCZOS)
    icon.alpha_composite(mark, ((512-mark.width)//2, (512-mark.height)//2))
    for filename, size in [('icon-512.png',512), ('icon-192.png',192), ('apple-touch-icon.png',180)]:
        icon.resize((size,size), Image.Resampling.LANCZOS).save(ROOT / 'web' / filename)
    icon.save(ROOT / 'assets' / 'app_icon.ico', sizes=[(16,16),(32,32),(48,48),(128,128),(256,256)])


def import_hero_atlas(stem):
    folder = ROOT / 'assets' / 'mythology'
    manifest = json.loads((folder / f'{stem}.json').read_text(encoding='utf-8'))
    size = manifest['cell_size']
    sprites = []
    for sheet in manifest['atlases']:
        tiles = cells(folder / sheet['file'], sheet['columns'], sheet['rows'], size, alpha_floor=24)
        if len(tiles) != len(sheet['heroes']) or any(not tile.getchannel('A').getbbox() for tile in tiles):
            raise ValueError(f"Unexpected sprite layout in {sheet['file']}")
        sprites.extend(bgra(tile) for tile in tiles)
    (folder / f'{stem}.bgra').write_bytes(b''.join(sprites))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group()
    group.add_argument('--regional-heroes', dest='hero_atlas', action='store_const', const='regional-heroes')
    group.add_argument('--faction-heroes', dest='hero_atlas', action='store_const', const='faction-heroes')
    atlas = parser.parse_args().hero_atlas
    if atlas:
        import_hero_atlas(atlas)
    else:
        main()
