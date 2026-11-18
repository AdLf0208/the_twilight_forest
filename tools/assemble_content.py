#!/usr/bin/env python3
"""Write the portal's content — every `assets/`/`data/` file it ships — from this port's own text.

Upstream's portal files were carried verbatim in the 0.1.0 slice. The jar's code is LGPL-2.1 but
the upstream repository's ``ASSET_LICENSE`` is CC-BY-NC-SA-4.0, so a package published under the
code's licence can carry **none** of those bytes (ARCHITECTURE.md §8; ``LICENSE-NOTICE.md``).
They are therefore **re-authored here**: the blockstate, the barrier model, the three portal tags
and the decoration tag express the same functional content as the jar's files — the block's two
state variants, the barrier's two planes, each tag's membership — in the port's own bytes, key
order and formatting. The reason for each is recorded in ``PROVENANCE.md``, which
``tools/check_provenance.py`` generates and proves.

What is written:

* ``blockstates/twilight_portal.json`` — the multipart the block's states need: the portal's own
  model always, the barrier's model when ``is_one_way`` is true;
* ``models/block/twilight_portal_barrier.json`` — the one-way barrier: two horizontal planes just
  under the block's 13/16 outline, no ambient occlusion, the barrier texture on both faces;
* ``models/block/twilight_portal.json`` — the port's own full-cube stand-in (the swirl renderer is
  not ported; ``notes.md`` records the deviation);
* ``tags/block/portal/{edge,fluid,generated_decoration,decoration}.json`` — the frame's, the
  pool's, the plants a portal may be built among, and what may stand on the frame: the ids the
  port's rules read, sorted, with the mod's own plants dropped because this port does not carry
  them;
* ``lang/en_us.json``, the block declaration, its loot table (empty: upstream has no loot table)
  and the two activator tags that close the jar's ``#c:gems/diamond`` defect;
* the barrier texture and its animation, from :mod:`gen_portal_texture`.

    python3 tools/assemble_content.py [--out PATH]
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from gen_portal_texture import build as build_texture

PORT = Path(__file__).resolve().parent.parent
MOD = "twilightforest"

# The block: `init/TFBlocks.java` — `strength(-1.0F)`, `sound(SoundType.GLASS)`,
# `lightLevel(state -> 11)`, `noCollission()`, `noOcclusion()`, `noLootTable()`. The template
# supplies the block class, its unbreakable hardness and its tool rules; everything the mod
# decides is declared here.
PORTAL_BLOCK = {
    "template": "minecraft:barrier",
    "item": False,
    "properties": {"light": 11, "sounds": "minecraft:glass", "hardness": -1.0, "occludes": False},
    "state_properties": {"is_one_way": {"type": "boolean"}},
    "default_state": {"is_one_way": "false"},
    # `Shapes.empty()` with `is_one_way = false`; `is_one_way = true` blocks.
    "collision_shapes": {
        "properties": ["is_one_way"],
        "values": {"true": [[0.0, 0.0, 0.0, 1.0, 0.8125, 1.0]]},
        "default": [],
    },
    # `getShape` is the same box either way: the outline is visible, the walk-through is not.
    "shapes": {"default": [[0.0, 0.0, 0.0, 1.0, 0.8125, 1.0]]},
}

# The blockstate: two variants, unconditionally the portal's model and, for the one-way state, the
# barrier's. `when` first, so a reader sees the condition before the model it selects.
BLOCKSTATE = {
    "multipart": [
        {"apply": {"model": f"{MOD}:block/twilight_portal"}},
        {
            "when": {"is_one_way": "true"},
            "apply": {"model": f"{MOD}:block/twilight_portal_barrier"},
        },
    ],
}

# The one-way barrier: two flat planes just under the block's own 13/16 outline, both faces drawn
# with the barrier texture. No model-level render type: this pin declares none (docs/mods.md
# reads `render_shape` on the block), and the texture is opaque, so neither layer choice changes
# what a player sees.
BARRIER_MODEL = {
    "parent": "block/block",
    "ambientocclusion": False,
    "textures": {
        "particle": f"{MOD}:block/portal_barrier",
        "all": f"{MOD}:block/portal_barrier",
    },
    "elements": [
        {
            "from": [0, plane, 0],
            "to": [16, plane, 16],
            "faces": {
                "up": {"uv": [0, 0, 16, 16], "texture": "#all"},
                "down": {"uv": [0, 0, 16, 16], "texture": "#all"},
            },
        }
        for plane in (11.5, 13.5)
    ],
}

# The port's own stand-in for the portal block: a full cube in vanilla's portal texture, because
# this pin has no model-level render type and the mod's `PortalOverlay` swirl renderer is not
# ported (notes.md, deviation 4).
PORTAL_MODEL = {
    "textures": {"particle": "block/nether_portal", "all": "block/nether_portal"},
    "elements": [
        {
            "from": [0, 0, 0],
            "to": [16, 16, 16],
            "faces": {
                face: {"texture": "#all"}
                for face in ("down", "up", "north", "south", "west", "east")
            },
        }
    ],
}

# The frame: soil and paths, plus the dirt tag. Sorted, so the file's order is a rule, not a copy.
EDGE_TAG = ["#minecraft:dirt", "minecraft:dirt_path", "minecraft:farmland"]
# The pool: the one block the rule fills (`portal.rs::POOL_BLOCK`).
FLUID_TAG = ["minecraft:water"]
# What may stand on the frame (`#twilightforest:portal/decoration`): vanilla's plants, sorted.
# The mod's own plants are dropped — this port does not carry them (notes.md, deviation 8).
DECORATION_TAG = sorted(
    [
        "minecraft:attached_melon_stem",
        "minecraft:attached_pumpkin_stem",
        "minecraft:bamboo",
        "minecraft:big_dripleaf",
        "minecraft:big_dripleaf_stem",
        "minecraft:brown_mushroom",
        "minecraft:chorus_flower",
        "minecraft:chorus_plant",
        "minecraft:cocoa",
        "minecraft:crimson_fungus",
        "minecraft:dead_bush",
        "minecraft:fern",
        "minecraft:glow_lichen",
        "minecraft:large_fern",
        "minecraft:moss_carpet",
        "minecraft:nether_wart",
        "minecraft:pink_petals",
        "minecraft:red_mushroom",
        "minecraft:short_grass",
        "minecraft:small_dripleaf",
        "minecraft:sugar_cane",
        "minecraft:sweet_berry_bush",
        "minecraft:tall_grass",
        "minecraft:vine",
        "minecraft:warped_fungus",
        "#minecraft:crops",
        "#minecraft:flowers",
        "#minecraft:leaves",
        "#minecraft:saplings",
    ]
)
# The plants a generated decoration pass may place: the vanilla members only, sorted.
GENERATED_DECORATION_TAG = sorted(
    [
        "minecraft:allium",
        "minecraft:azure_bluet",
        "minecraft:blue_orchid",
        "minecraft:brown_mushroom",
        "minecraft:cornflower",
        "minecraft:fern",
        "minecraft:lily_of_the_valley",
        "minecraft:orange_tulip",
        "minecraft:oxeye_daisy",
        "minecraft:pink_tulip",
        "minecraft:red_mushroom",
        "minecraft:red_tulip",
        "minecraft:short_grass",
        "minecraft:white_tulip",
    ]
)

# The port's own prose. The portal's tip line is written for this port: the 0.1.0 package carried
# the jar's sentence, and no upstream text travels now (PROVENANCE.md).
LANG = {
    "block.twilightforest.twilight_portal": "Twilight Forest Portal",
    "twilightforest.tips.twilight_portal": (
        "Drop a diamond into a pool of water ringed by flowers to open a portal."
    ),
    "block.twilightforest.deadrock": "Deadrock",
    "block.twilightforest.cracked_deadrock": "Cracked Deadrock",
    "block.twilightforest.weathered_deadrock": "Weathered Deadrock",
    "twilightforest.tips.twilight_dimension": "A Twilight Forest portal opens into the mod's own dimension.",
}

# No loot table upstream (`noLootTable()`): the block drops nothing.
LOOT = {"pools": []}

# The jar's activator tag names `#c:gems/diamond`. 26.3's base game ships no `c:` tags, so the
# jar's reference can never match there (an upstream defect against this target; notes.md,
# "Upstream defects fixed"). The port declares the tag in both places a pack may look, with the
# diamond under the convention name and the id the runtime matches.
CONVENTION_DIAMOND = {"values": ["minecraft:diamond"]}
ACTIVATOR = {"values": ["#c:gems/diamond", "minecraft:diamond"]}


def json_file(value: object) -> bytes:
    """A file in the port's own JSON style: two-space indent, one trailing newline."""
    return (json.dumps(value, indent=2) + "\n").encode()


def tag(values: list[str]) -> bytes:
    return json_file({"values": values})


def build() -> dict[str, bytes]:
    """Every file this script owns, keyed by its path inside the package."""
    files = {
        "assets/twilightforest/blockstates/twilight_portal.json": json_file(BLOCKSTATE),
        "assets/twilightforest/models/block/twilight_portal_barrier.json": json_file(BARRIER_MODEL),
        "assets/twilightforest/models/block/twilight_portal.json": json_file(PORTAL_MODEL),
        "assets/twilightforest/lang/en_us.json": json_file(LANG),
        "data/twilightforest/horizon/block/twilight_portal.json": json_file(PORTAL_BLOCK),
        "data/twilightforest/loot_table/blocks/twilight_portal.json": json_file(LOOT),
        "data/twilightforest/tags/block/portal/edge.json": tag(EDGE_TAG),
        "data/twilightforest/tags/block/portal/fluid.json": tag(FLUID_TAG),
        "data/twilightforest/tags/block/portal/decoration.json": tag(DECORATION_TAG),
        "data/twilightforest/tags/block/portal/generated_decoration.json": tag(GENERATED_DECORATION_TAG),
        "data/c/tags/item/gems/diamond.json": json_file(CONVENTION_DIAMOND),
        "data/twilightforest/tags/item/portal/activator.json": json_file(ACTIVATOR),
    }
    files.update(build_texture())
    return files


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--out", type=Path, default=PORT)
    parser.add_argument(
        "--check", action="store_true", help="compare only; exit 1 if a file differs"
    )
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
        print(f"wrote {name} ({len(body)} B)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
