#!/usr/bin/env python3
"""Carry the jar's world generation for the dimension slice into the package.

Reads the mod's own jar (never a decompiled or copied class) and writes, under the port's
``data/twilightforest/``:

* the 14 biomes this slice carries, converted from the 1.21.1 biome shape to 26.3's
  (``attributes`` for the colours and the natural spawns, ``effects`` for the water colour,
  ``features``/``carvers`` empty because decoration, carvers and the mod's entities are out of
  the slice's scope);
* the jar's ``surface_rule`` as a 26.3 ``material_rule`` (same node types; ``result_state``
  becomes a block state id);
* the jar's ``twilight_noise_gen`` noise settings with 26.3's key names and a decodable density
  router standing in for the two density-function types this pin has no code for.

Everything it drops is printed, so the port's ``notes.md`` can record it. Run:

    python3 tools/assemble_worldgen.py [--jar PATH] [--out PATH] [--check]

``--check`` only compares the files on disk with what the jar produces.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import zipfile
from pathlib import Path

PORT = Path(__file__).resolve().parent.parent
DEFAULT_JAR = (
    PORT.parent / "_shared" / "licence-cache" / "twilightforest-1.21.1-4.8.3345-universal.jar"
)

# The biomes this slice carries: the jar's own surface rule names these, and they are the grid's
# landmarks and its edge. notes.md states the simplification for the eight it does not carry.
BIOMES = [
    "clearing",
    "dark_forest_center",
    "final_plateau",
    "glacier",
    "fire_swamp",
    "thornlands",
    "highlands",
    "highlands_underground",
    "lake",
    "snowy_forest",
    "stream",
    "swamp",
    "forest",
    "underground",
]

# 26.3's natural-spawn categories, in the vanilla biome files' order.
CATEGORIES = [
    "ambient",
    "axolotls",
    "creature",
    "misc",
    "monster",
    "underground_water_creature",
    "water_ambient",
    "water_creature",
]


def hex_color(value: int) -> str:
    return f"#{value & 0xFFFFFF:06x}"


def convert_block_state(state: dict) -> dict | str:
    """1.21.1 ``{Name, Properties}`` to 26.3 ``"id"`` or ``{id, properties}``."""
    name = state["Name"]
    properties = state.get("Properties")
    if not properties:
        return name
    return {"id": name, "properties": dict(properties)}


def convert_rule(node: dict) -> dict:
    """A ``surface_rule`` node as a 26.3 ``material_rule`` node."""
    out = {"type": node["type"]}
    for key, value in node.items():
        if key == "type":
            continue
        if key == "result_state":
            out[key] = convert_block_state(value)
        elif isinstance(value, dict) and "type" in value:
            out[key] = convert_rule(value)
        elif isinstance(value, list):
            out[key] = [convert_rule(item) if isinstance(item, dict) and "type" in item else item for item in value]
        else:
            out[key] = value
    return out


def convert_biome(name: str, source: dict) -> dict:
    effects = source.get("effects", {})
    attributes: dict = {
        # The spawners name the mod's own entity types, which this slice does not carry; the
        # overlay declares every category empty so nothing comes from the dimension's defaults.
        "minecraft:gameplay/natural_mob_spawns": {
            "argument": {
                "spawn_costs": {},
                "spawns_by_category": {category: [] for category in CATEGORIES},
            },
            "modifier": "overlay",
        },
    }
    if "sky_color" in effects:
        attributes["minecraft:visual/sky_color"] = hex_color(effects["sky_color"])
    if "fog_color" in effects:
        attributes["minecraft:visual/fog_color"] = hex_color(effects["fog_color"])
    if "water_fog_color" in effects:
        attributes["minecraft:visual/water_fog_color"] = hex_color(effects["water_fog_color"])
    out_effects: dict = {"water_color": hex_color(effects.get("water_color", 4159204))}
    for key in ("foliage_color", "grass_color", "dry_foliage_color"):
        if key in effects:
            out_effects[key] = hex_color(effects[key])
    return {
        "attributes": attributes,
        # `twilightforest:tf_caves` is a configured carver the slice does not carry.
        "carvers": [],
        "downfall": source["downfall"],
        "effects": out_effects,
        # Decoration is out of the slice's scope.
        "features": [],
        "has_precipitation": source["has_precipitation"],
        "temperature": source["temperature"],
    }


def noise_settings(source: dict, rule: str) -> dict:
    """The jar's settings with 26.3's key names.

    ``aquifers`` is omitted: the jar disables them (``"aquifers_enabled": false``) and an absent
    ``aquifers`` object is how 26.3 says the same. The router's eight roots are constants because
    the jar's two Java density-function types cannot be decoded at this pin (notes.md).
    """
    del source["noise"]["size_horizontal"], source["noise"]["size_vertical"]
    return {
        "default_block": source["default_block"]["Name"],
        "default_fluid": source["default_fluid"]["Name"],
        "disable_mob_generation": source["disable_mob_generation"],
        "legacy_random_source": source["legacy_random_source"],
        "material_rule": rule,
        "noise": source["noise"],
        "noise_router": {
            "chunk_surface_level": 0.0,
            "continents": 0.0,
            "depth": 0.0,
            "erosion": 0.0,
            "final_density": 0.0,
            "ridges": 0.0,
            "temperature": 0.0,
            "vegetation": 0.0,
        },
        "sea_level": source["sea_level"],
        "spawn_target": [],
    }


def read(jar: zipfile.ZipFile, name: str) -> dict:
    return json.loads(jar.read(name))


def build(jar_path: Path) -> dict[str, str]:
    """The files this carry writes, as ``relative path -> text``."""
    files: dict[str, str] = {}
    with zipfile.ZipFile(jar_path) as jar:
        for name in BIOMES:
            source = read(jar, f"data/twilightforest/worldgen/biome/{name}.json")
            files[f"data/twilightforest/worldgen/biome/{name}.json"] = json.dumps(
                convert_biome(name, source), indent=2
            ) + "\n"
        settings = read(jar, "data/twilightforest/worldgen/noise_settings/twilight_noise_gen.json")
        files["data/twilightforest/worldgen/material_rule/twilight_noise_gen.json"] = (
            json.dumps(convert_rule(settings["surface_rule"]), indent=2) + "\n"
        )
        files["data/twilightforest/worldgen/noise_settings/twilight_noise_gen.json"] = (
            json.dumps(
                noise_settings(settings, "twilightforest:twilight_noise_gen"), indent=2
            )
            + "\n"
        )
    return files


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--jar", type=Path, default=DEFAULT_JAR)
    parser.add_argument("--out", type=Path, default=PORT)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if not args.jar.is_file():
        print(f"missing jar: {args.jar}", file=sys.stderr)
        return 1
    files = build(args.jar)
    if args.check:
        differences = [
            name
            for name, text in files.items()
            if not (args.out / name).is_file() or (args.out / name).read_text() != text
        ]
        print(f"{len(files)} files carried; {len(differences)} differ")
        for name in differences:
            print(f"  differs: {name}")
        return 1 if differences else 0
    for name, text in files.items():
        target = args.out / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text)
    print(f"wrote {len(files)} files into {args.out}")
    print("not carried: the 8 other biomes, the jar's carver, its features and its spawners,")
    print("its client grass-colour modifiers, its fixed-time value and its effects id, and the")
    print("density functions whose types this pin cannot decode (notes.md).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
