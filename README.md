# The Twilight Forest — a Horizon port

**The Twilight Forest** is a Minecraft mod. Twilight Forest repository. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [TeamTwilight/twilightforest](https://github.com/TeamTwilight/twilightforest) — licence **LGPL-2.1-only**.

| | |
|---|---|
| Upstream mod | The Twilight Forest ([source](https://github.com/TeamTwilight/twilightforest)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `twilightforest.hmod` (install it with the Horizon launcher) |
| Licence | **LGPL-2.1-only** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

- **Migrate in this order and do not interleave:** scaffold → `cmp` `runtime.rs` into `src/` →
- **`runtime.rs` is the line that makes it a migration.** `cp`, then `cmp`, then `cargo test`
- **`lib.rs` registers, a module glues.** With N hooks: one `impl Hooks` with N one-line methods,
- **The helper→hook table is the whole review.** Write it before the code: every old helper, its
- **Handles, not host areas** (trap 4). Read the `impl <Handle>` blocks in
- **`HostSide` decides the half.** Any `side = "both"` hook (player ticks, level ticks, block
- **State on the struct, not in statics.** `Mod { … : Mutex<…> }` with `&self` hook methods. A
- **Content stops being a trick.** The `content_root`/embedded-`.hmod` pattern (§1.0 of

## Not yet ported

- `Registrar::register_dimension(name, DimensionSettings)` — “every world then has the level
- `register_chunk_generator::<T>(name, generator)` and
- `EntityRef::teleport_to_level(host, dimension, pos, rotation)` — the portal's travel, which
- `RuntimeLevels::open/unload/delete` — mod-owned levels at run time (cap 64), not what a static
- **`blocked(density-function type: twilightforest:biome_driven_terrain)`** and
- **`blocked(terrain)`** — because of the two types above: the level has no noise terrain. The
- **`blocked(fixed_time value)`** — `dimension_type/twilight_forest_type.json:5` (`13000`): the SDK
- **`blocked(effects)`** — `dimension_type/twilight_forest_type.json:4` (`effects:

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `twilightforest.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

The Twilight Forest was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **LGPL-2.1-only**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/TeamTwilight/twilightforest

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run twilightforest` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/twilightforest/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
