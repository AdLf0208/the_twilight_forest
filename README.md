# The Twilight Forest for Horizon — the portal and the dimension

This repository is a **partial** port of
[The Twilight Forest](https://github.com/TeamTwilight/twilightforest) to
[Horizon](https://github.com/horizon-rust/horizon) (Minecraft 26.3, Rust),
pinned to the Horizon SDK nightly `0b14f8acfd97fa2c750e42795249dcc981178ca4` and
horizon-mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, version **0.2.0**.

## What it does

**The portal.** A 4-or-more-block water pool standing on a sturdy floor, ringed by
grass/dirt/farmland/dirt-path blocks with a plant on each, is lit into
`twilightforest:twilight_portal` when a **diamond** is dropped into it. The catalyst is consumed,
lightning strikes the pool, and the block keeps upstream's light 11, no collision, glass sound and
0–13/16 outline. The rule is upstream's own (`TFPortalBlock.recursivelyValidatePortal`): the flood
fill walks the four horizontal neighbours of each pool block, accepts a frame block only when a
plant stands on it, refuses anything else, and lights at most 64 pool blocks.

**The dimension.** `twilightforest:twilight_forest` is registered with the jar's own dimension
type — `min_y -32`, height and logical height `288`, `coordinate_scale 0.125`, `ambient_light
0.01`, fixed time, no ceiling, skylight, `monster_spawn_light_level` 0–7, block-light limit 7, no
raids, beds work, no respawn anchors, piglins zombify. Its `BiomeSource` answers
`twilightforest:twilight_biomes` by reproducing the jar's whole layer stack over its `biome_grid`
in pure Rust: `src/grid.rs` is the grid (the LCG and `mixSeed`, the layer random, the key-biome
jitter, the companion/zoom/stabilize/border/seam/smooth/filtered layers, the elevation column),
and `src/dimension.rs` is the glue.

**`/twilightforest dimension [<x> <y> <z> <seed>]`** prints the level, its biome count and the
grid's answer for a cell — the readout the in-game harness asserts on.

One flat package, `twilightforest.hmod`, carries both halves: the code in `src/` and the content
in `assets/` and `data/` (the portal block, the three `deadrock` blocks the jar's surface rule
places, 14 biomes, the surface rule and the noise settings).

## What it does **not** do

- **The terrain.** The jar's density graph names two density-function types
  (`twilightforest:biome_driven_terrain`, `twilightforest:biome_driven_noise`) that this pin's
  density registry cannot decode, and the registry loads atomically. The slice therefore registers
  a chunk generator that writes nothing: the level exists, its biomes and its type are the jar's,
  and its terrain is empty. The noise settings and the material rule are carried, converted, ready
  for the slice that can express the density.
- **8 of the 22 biomes** are not carried. The grid's placement is exact for all 22; a biome outside
  the carried set answers the nearest carried biome of its own family (the port's table, documented
  in `notes.md`).
- **Decoration, carvers, structures, mobs, bosses, progression and the client effects** are not
  ported, and the biomes' spawners are empty.
- **The portal's travel** is decided (`grid::travel_target`, unit-tested) but not wired: the
  destination has no terrain at this pin.

`notes.md` has the whole gap list with file:line, the deviations, and the jar inventory.

## Building

```sh
# the pure logic and the grid (no game needed)
cargo test

# compile the glue against the pinned SDK (fetched into .horizon/sdk/ on first use)
cargo check && cargo clippy --all-targets -- -D warnings

# the package: one flat .hmod, code + content
cargo horizon-mod build && cp target/horizon-mod/twilightforest.hmod ./twilightforest.hmod
cargo horizon-mod check twilightforest.hmod && cargo horizon-mod fits twilightforest.hmod

# the content, written by the port's own tools (no upstream byte; see PROVENANCE.md)
python3 tools/assemble_content.py        # the portal: blockstate, models, tags, lang, texture
python3 tools/gen_portal_texture.py      # the barrier's swirl and its animation, drawn here
python3 tools/assemble_worldgen.py       # 14 biomes, the material rule, the noise settings
python3 tools/assemble_surface_blocks.py # the three deadrock blocks and their drawn textures
python3 tools/check_provenance.py        # assert no packaged file matches the jar; write PROVENANCE.md

# in game (needs a game the launcher built with this package)
horizon-launcher native-build . --instance <instance> --trust-native-code
GAME=<that horizon-client> tools/e2e.sh    # portal legs, then the dimension and the readout
```

`VALIDATION.md` records the results, sizes and SHA-256s; `notes.md` records how the grid was
proved against an independent Java oracle.

## Licence

The Twilight Forest's **code** is LGPL-2.1 (read from the jar's own
`META-INF/neoforge.mods.toml`); this port is a clean-room Rust rewrite of the behaviour described
in `FINDINGS.md` and `notes.md`, published under the same licence with its complete source in this
repository.

The upstream repository's `ASSET_LICENSE` is **CC-BY-NC-SA-4.0**, and the jar carries no separate
licence file. **No upstream art travels in this package:** every texture, model, blockstate, tag
and lang file is written or drawn by the port's own tools, and
`tools/check_provenance.py` proves it by comparing every packaged file's sha256 with every entry of
the jar (`PROVENANCE.md` is its per-file report). The portal's swirl is
`tools/gen_portal_texture.py`'s, the `deadrock` textures are
`tools/assemble_surface_blocks.py`'s. See `LICENSE-NOTICE.md` for the full split.

Copyright © Benimatic (Ben Mazur) and the Twilight Forest contributors.
