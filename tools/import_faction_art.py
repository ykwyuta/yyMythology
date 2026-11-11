"""Pack the faction artwork into the engine's premultiplied BGRA format."""

import hashlib
import json
from pathlib import Path

from PIL import Image

from import_period_art import bgra, cells

ROOT = Path(__file__).resolve().parents[1]
ART = ROOT / 'assets' / 'faction-art'
MYTHOLOGY = ROOT / 'assets' / 'mythology'


def legacy_buildings(source, size):
    path = ROOT / source['path']
    cell_size = source['cell_size']
    cell_bytes = cell_size * cell_size * 4
    with path.open('rb') as file:
        file.seek(source['first_cell'] * cell_bytes)
        for _ in range(6):
            rgba = Image.frombytes('RGBA', (cell_size, cell_size), file.read(cell_bytes), 'raw', 'BGRA')
            straight_alpha = Image.frombytes('RGBa', rgba.size, rgba.tobytes()).convert('RGBA')
            yield straight_alpha.resize((size, size), Image.Resampling.LANCZOS)


def pack(kind, config, faction_count):
    size = config['cell_size']
    slots = config['sprites_per_faction']
    factions = {}

    def add(faction, sprites):
        if faction in factions:
            raise ValueError(f'{kind}: duplicate faction {faction}')
        if len(sprites) != slots:
            raise ValueError(f'{kind}: faction {faction} has {len(sprites)} sprites, expected {slots}')
        for sprite in sprites:
            alpha = sprite.getchannel('A')
            if not alpha.point(lambda value: 255 if value > 128 else 0).getbbox():
                raise ValueError(f'{kind}: empty sprite for faction {faction}')
        factions[faction] = sprites

    for source in config.get('legacy', []):
        add(source['faction'], list(legacy_buildings(source, size)))

    for sheet in config['sheets']:
        path = ART / sheet['file']
        alpha_floor = sheet.get('alpha_floor', 24)
        with Image.open(path) as source:
            alpha = source.convert('RGBA').getchannel('A')
            if sum(alpha.histogram()[:alpha_floor + 1]) < alpha.width * alpha.height / 3:
                raise ValueError(f'{path.name}: sprite sheet background is not sufficiently transparent')
        sprites = cells(path, sheet['columns'], sheet['rows'], size, alpha_floor, sheet.get('row_edges'))
        if len(sprites) != len(sheet['factions']) * slots:
            raise ValueError(f"{sheet['file']}: atlas dimensions do not match the faction list")
        for offset, faction in enumerate(sheet['factions']):
            add(faction, sprites[offset * slots:(offset + 1) * slots])

    if set(factions) != set(range(faction_count)):
        raise ValueError(f'{kind}: faction atlas does not cover the full roster')
    encoded = [bgra(sprite) for faction in range(faction_count) for sprite in factions[faction]]
    if len({hashlib.sha256(sprite).digest() for sprite in encoded}) != len(encoded):
        raise ValueError(f'{kind}: at least one sprite is shared')
    (MYTHOLOGY / f'faction-{kind}.bgra').write_bytes(b''.join(encoded))


def main():
    manifest = json.loads((ART / 'generation.json').read_text(encoding='utf-8'))
    faction_count = len(manifest['factions'])
    for kind in ('buildings', 'units'):
        pack(kind, manifest[kind], faction_count)


if __name__ == '__main__':
    main()
