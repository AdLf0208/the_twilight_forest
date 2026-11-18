#!/usr/bin/env python3
"""Declare the three blocks the jar's surface rule places, with original textures.

The jar's ``surface_rule`` (``worldgen/material_rule/twilight_noise_gen.json``) paints
``twilightforest:deadrock``, ``cracked_deadrock`` and ``weathered_deadrock``. This script declares
those three blocks, their models, blockstates, items, loot and the ``#twilightforest:deadrock`` tag,
and *draws its own 16x16 textures* (a deterministic speckle from a fixed palette): the upstream
repository's ``ASSET_LICENSE`` is CC-BY-NC-SA-4.0, so no upstream art is copied into this package
(``notes.md``, licence section).

The block behaviour is the vanilla stone template; the jar's own hardness and sounds are Java
code and are not carried (``notes.md``).

    python3 tools/assemble_surface_blocks.py [--out PATH] [--check]
"""

from __future__ import annotations

import argparse
import json
import random
import struct
import zlib
from pathlib import Path

PORT = Path(__file__).resolve().parent.parent

BLOCKS = {
    "deadrock": {
        "name": "Deadrock",
        # palette: a cold, dark stone (the jar's own art is not used)
        "palette": [(58, 56, 52), (66, 63, 58), (49, 47, 44), (74, 71, 65)],
        "cracks": 0,
    },
    "cracked_deadrock": {
        "name": "Cracked Deadrock",
        "palette": [(58, 56, 52), (66, 63, 58), (49, 47, 44), (40, 38, 36)],
        "cracks": 9,
    },
    "weathered_deadrock": {
        "name": "Weathered Deadrock",
        "palette": [(84, 82, 74), (94, 92, 83), (72, 70, 64), (104, 101, 91)],
        "cracks": 3,
    },
}


def png(pixels: list[list[tuple[int, int, int]]]) -> bytes:
    """A 16x16 RGB PNG, encoded by hand so the script needs no image library."""
    height = len(pixels)
    width = len(pixels[0])
    raw = b"".join(b"\x00" + b"".join(bytes(pixel) for pixel in row) for row in pixels)

    def chunk(kind: bytes, body: bytes) -> bytes:
        return (
            struct.pack(">I", len(body))
            + kind
            + body
            + struct.pack(">I", zlib.crc32(kind + body) & 0xFFFFFFFF)
        )

    header = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def texture(name: str, spec: dict) -> bytes:
    """A deterministic speckle, one of the palette's colours per pixel, plus optional cracks."""
    random_source = random.Random(f"twilightforest:{name}")
    palette = spec["palette"]
    pixels = [
        [palette[random_source.randrange(len(palette))] for _ in range(16)] for _ in range(16)
    ]
    for _ in range(spec["cracks"]):
        x, y = random_source.randrange(16), random_source.randrange(16)
        for _ in range(random_source.randrange(4, 9)):
            pixels[y % 16][x % 16] = palette[0] if palette[0] != (40, 38, 36) else (40, 38, 36)
            if random_source.random() < 0.5:
                x += 1
            else:
                y += 1
    return png(pixels)


def block_state(name: str) -> dict:
    return {"variants": {"": [{"model": f"twilightforest:block/{name}"}]}}


def block_model(name: str) -> dict:
    return {
        "parent": "minecraft:block/cube_all",
        "textures": {"all": f"twilightforest:block/{name}"},
    }


def item_model(name: str) -> dict:
    return {"model": {"type": "minecraft:model", "model": f"twilightforest:block/{name}"}}


def loot(name: str) -> dict:
    return {
        "type": "minecraft:block",
        "pools": [
            {
                "bonus_rolls": 0.0,
                "conditions": [{"condition": "minecraft:survives_explosion"}],
                "entries": [{"type": "minecraft:item", "name": f"twilightforest:{name}"}],
                "rolls": 1.0,
            }
        ],
    }


def declaration(name: str) -> dict:
    return {
        # The jar's own hardness/sound/map colour are Java code; the stone template supplies a
        # stone-like block and the port does not guess the numbers (notes.md).
        "template": "minecraft:stone",
        "item": True,
    }


def build() -> dict[str, bytes]:
    files: dict[str, bytes] = {}
    for name, spec in BLOCKS.items():
        files[f"data/twilightforest/horizon/block/{name}.json"] = (
            json.dumps(declaration(name), indent=2) + "\n"
        ).encode()
        files[f"data/twilightforest/loot_table/blocks/{name}.json"] = (
            json.dumps(loot(name), indent=2) + "\n"
        ).encode()
        files[f"assets/twilightforest/blockstates/{name}.json"] = (
            json.dumps(block_state(name), indent=2) + "\n"
        ).encode()
        files[f"assets/twilightforest/models/block/{name}.json"] = (
            json.dumps(block_model(name), indent=2) + "\n"
        ).encode()
        files[f"assets/twilightforest/items/{name}.json"] = (
            json.dumps(item_model(name), indent=2) + "\n"
        ).encode()
        files[f"assets/twilightforest/textures/block/{name}.png"] = texture(name, spec)
    files["data/twilightforest/tags/block/deadrock.json"] = (
        json.dumps(
            {
                "values": [
                    "twilightforest:deadrock",
                    "twilightforest:cracked_deadrock",
                    "twilightforest:weathered_deadrock",
                ]
            },
            indent=2,
        )
        + "\n"
    ).encode()
    files["data/minecraft/tags/block/mineable/pickaxe.json"] = (
        json.dumps(
            {
                "values": [
                    "twilightforest:deadrock",
                    "twilightforest:cracked_deadrock",
                    "twilightforest:weathered_deadrock",
                ]
            },
            indent=2,
        )
        + "\n"
    ).encode()
    return files


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=PORT)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    files = build()
    if args.check:
        differences = [
            name
            for name, body in files.items()
            if not (args.out / name).is_file() or (args.out / name).read_bytes() != body
        ]
        print(f"{len(files)} files written; {len(differences)} differ")
        for name in differences:
            print(f"  differs: {name}")
        return 1 if differences else 0
    for name, body in files.items():
        target = args.out / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(body)
    print(f"wrote {len(files)} files into {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
