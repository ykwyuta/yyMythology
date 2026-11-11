"""Pack war elephant sprites to the premultiplied BGRA used by the game."""

import json

from import_period_art import ROOT, bgra, cells


def main():
    folder = ROOT / "assets" / "faction-art"
    config = json.loads((folder / "war-elephants.json").read_text(encoding="utf-8"))
    sprites = cells(folder / config["file"], config["columns"], config["rows"], config["cell_size"], alpha_floor=24)
    if len(sprites) != len(config["slots"]):
        raise ValueError("War elephant atlas does not match its slot list")
    for sprite in sprites:
        alpha = sprite.getchannel("A")
        if alpha.getextrema()[0] != 0 or alpha.getextrema()[1] <= 128:
            raise ValueError("War elephant sprite requires transparent padding and visible art")
    packed = [bytes(bgra(sprite)) for sprite in sprites]
    if len(set(packed)) != len(packed):
        raise ValueError("War elephant sprites must be distinct")
    (ROOT / "assets" / "mythology" / "war-elephants.bgra").write_bytes(b"".join(packed))


if __name__ == "__main__":
    main()
