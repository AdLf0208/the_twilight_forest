# Validation — The Twilight Forest

Two records: the **SDK package** delivered now (§0), and the **retired 0.1.0 slice** at campaign pin
`1d63e0a` (§1–§7) whose in-game evidence still stands for the rule. §0b is the dimension slice and
§0c the licence-clean pass; §0c's package identity is the current one. Every step ran inside a
`ports/tools/build-slot.sh` lease with `CARGO_INCREMENTAL=0`.

## 0. The SDK package (2026-11-18)

Pin **Horizon `0b14f8acfd97fa2c750e42795249dcc981178ca4`**, mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, launcher 0.2.78. The retired container and its tools
are under `ports/_shared/evidence/twilightforest/retired-hnmod-0.1.0/`.

| Step | Command | Result |
|---|---|---|
| native compile | `cargo check` (the SDK at `.horizon/sdk/`) | **EXIT 0** — this is the gate `cargo horizon-mod build` does *not* provide |
| runtime tests | `cargo test` | **14 passed, 0 failed**, 0 ignored (`evidence/…/sdk-test.log`) |
| runtime identity | `cmp src/runtime.rs retired…/mixins/twilightforest/runtime.rs` | identical, sha256 `942ba93f85dc21251f9a7533cd70885b07674acce96fb28fb74067e62c5d82e0` |
| clippy | `cargo clippy --all-targets --offline -- -D warnings` | **EXIT 0**, 0 warnings (`sdk-clippy.log`) |
| formatting | `cargo fmt --check` (not `--all`) | **EXIT 0** (`sdk-fmt.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `target/horizon-mod/twilightforest.hmod` → `./twilightforest.hmod` (`sdk-build.log`) |
| repeatability | `rm -rf .horizon target`, rebuild | same **44,311 B**, same sha256 `9f29089d8ec03378da5e42398338356628ba9d34ee4f091ce503c5328a117515` |
| registry check | `cargo horizon-mod check twilightforest.hmod` | `twilightforest 0.1.0 (native-mod, format native-1/0b14f8a…/e59da90…, for 26.3)` **EXIT 0** (`sdk-check.log`, `sdk-check-json.log`) |
| surface used | `cargo horizon-mod check twilightforest.hmod --uses` | 86 entries (`sdk-uses.log`, `sdk-uses.json`) |
| fits the build | `cargo horizon-mod fits twilightforest.hmod --build nightly` | `0b14f8a made for`, **EXIT 0** (`sdk-fits.log`) |
| content carry | `python3 tools/assemble_content.py` in a scratch copy, then `diff -r` | `assets/` and `data/` byte-identical to the port's |
| in-game leg | `tools/e2e.sh` | **not run**: no `horizon-launcher` here to build a game carrying the mod, and no `0b14f8a` unmodded game for the control leg (`blocked(harness)`), see notes.md “Undone in this slice” |

The package, entry by entry:

```
horizon-mod.toml, Cargo.toml, LICENSE, LICENSE-NOTICE.md, README.md,
assets/twilightforest/{blockstates/twilight_portal.json, lang/en_us.json,
  models/block/{twilight_portal,twilight_portal_barrier}.json,
  textures/block/portal_barrier.png(+.mcmeta)},
data/c/tags/item/gems/diamond.json,
data/twilightforest/{horizon/block/twilight_portal.json,
  loot_table/blocks/twilight_portal.json,
  tags/block/portal/{decoration,edge,fluid,generated_decoration}.json,
  tags/item/portal/activator.json},
src/{lib.rs, portal.rs, runtime.rs}, uses.json
```

## 0b. The dimension slice — package 0.2.0 (2026-11-18)

Same pin, same SDK. This slice adds the dimension: the jar's dimension type, a `BiomeSource` over
the jar's own biome grid, 14 carried biomes, the surface rule and the three blocks it places, and
the `/twilightforest dimension` readout. Every step ran inside a `ports/tools/build-slot.sh` lease
with `CARGO_INCREMENTAL=0`.

| Step | Command | Result |
|---|---|---|
| tests | `cargo test --offline` | **27 passed, 0 failed** — 14 portal (unchanged) + 11 grid + 2 dimension (`evidence/twilightforest/dimension-test.log`) |
| the portal runtime | `sha256sum src/runtime.rs` | `942ba93f85dc21251f9a7533cd70885b07674acce96fb28fb74067e62c5d82e0` — the 0.1.0 file, untouched (`cmp` clean) |
| the grid vs an independent oracle | `javac TFForestGridOracle.java && java TFForestGridOracle` then the tests | 60 seeded cell/elevation samples across 4 seeds, a 4,096-cell distribution and its digest (`7233979160456314091`) all match (`evidence/twilightforest/oracle/`) |
| clippy | `cargo clippy --all-targets --offline -- -D warnings` | **EXIT 0** (`dimension-clippy.log`) |
| formatting | `cargo fmt --check` | **EXIT 0** (`dimension-fmt.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | **78,861 B**, sha256 `c60f8e4b986e72c71a787f1919c684218888eed84e2661095a5c83d22c5b2d70`, **61 entries**, `LICENSE` + `LICENSE-NOTICE.md` inside, no `Cargo.lock` (`dimension-build.log`) |
| repeatability | a second `cargo horizon-mod build`, `cmp` | byte-identical, same sha256 (`dimension-build-2.log`) |
| registry check | `cargo horizon-mod check twilightforest.hmod` | `twilightforest 0.2.0 (native-mod, format native-1/0b14f8a…/e59da90…, for 26.3)` **EXIT 0** (`dimension-check.log`); `--json` `ok:true` with the known native-package `capabilities: []` / `side: null` (`dimension-check-json.log`) |
| surface used | `cargo horizon-mod check twilightforest.hmod --uses` | **235** entries (`dimension-uses.log`, `dimension-uses.json`) |
| fits the build | `cargo horizon-mod fits twilightforest.hmod --build nightly` | `0b14f8a made for`, **EXIT 0** (`dimension-fits.log`) |
| content carry | `python3 tools/assemble_worldgen.py --check`, `python3 tools/assemble_surface_blocks.py --check` | 16 and 20 files, **0 differ** |
| in-game leg | `tools/e2e.sh` leg 2 | **not run**: no `horizon-launcher` here to build a game carrying the package (`blocked(harness)`), see `notes.md` |

The package, entry by entry (61):

```
horizon-mod.toml, Cargo.toml, LICENSE, LICENSE-NOTICE.md, README.md, uses.json
assets/twilightforest/blockstates/{twilight_portal,deadrock,cracked_deadrock,weathered_deadrock}.json
assets/twilightforest/items/{deadrock,cracked_deadrock,weathered_deadrock}.json
assets/twilightforest/models/block/{twilight_portal,twilight_portal_barrier,deadrock,cracked_deadrock,weathered_deadrock}.json
assets/twilightforest/textures/block/{portal_barrier.png(+.mcmeta),deadrock.png,cracked_deadrock.png,weathered_deadrock.png}
assets/twilightforest/lang/en_us.json
data/c/tags/item/gems/diamond.json
data/minecraft/tags/block/mineable/pickaxe.json
data/twilightforest/horizon/block/{twilight_portal,deadrock,cracked_deadrock,weathered_deadrock}.json
data/twilightforest/loot_table/blocks/{twilight_portal,deadrock,cracked_deadrock,weathered_deadrock}.json
data/twilightforest/tags/block/deadrock.json
data/twilightforest/tags/block/portal/{decoration,edge,fluid,generated_decoration}.json
data/twilightforest/tags/item/portal/activator.json
data/twilightforest/worldgen/biome/*.json (14)
data/twilightforest/worldgen/material_rule/twilight_noise_gen.json
data/twilightforest/worldgen/noise_settings/twilight_noise_gen.json
src/{lib.rs,grid.rs,dimension.rs,portal.rs,runtime.rs}
```

## 0c. The licence-clean pass — package 0.2.0, no upstream byte (2026-11-18)

Same pin, same SDK, same version. This pass removes the last carried files: the upstream
repository's `ASSET_LICENSE` is CC-BY-NC-SA-4.0, so the seven jar-identical files of §0b are
re-authored or regenerated by the port, and `tools/check_provenance.py` proves the result. The
package identity below **supersedes §0b's** (`78,861 B` / `c60f8e4b…`); `src/runtime.rs` and the
27 tests are unchanged.

| Step | Command | Result |
|---|---|---|
| provenance, before | `python3 tools/check_provenance.py --check` on the delivered 0.2.0 package | **FAIL — 7 of 61** identical, exit 1; the same seven paths are reproduced by a counterfactual in `licence-provenance-red.log` (the gate is fail-capable) |
| provenance, after | `python3 tools/check_provenance.py --check` | **PROVENANCE OK: 61 packaged files, 0 identical to any of the jar's 10,437 entries**, exit 0 (`licence-provenance.log`) |
| the report | `python3 tools/check_provenance.py` | `PROVENANCE.md`, one line per packaged file with its bytes and provenance (`licence-provenance.log`) |
| generators | `assemble_content.py --check` 14 files, `gen_portal_texture.py --check` 2, `assemble_surface_blocks.py --check` 20, `assemble_worldgen.py --check` 16 | **0 differ** in all four (`licence-generators.log`) |
| tests | `cargo test --offline` | **27 passed, 0 failed** — 14 portal + 11 grid + 2 dimension (`licence-test.log`) |
| the portal runtime | `sha256sum src/runtime.rs` | `942ba93f…82e0` — unchanged (`cmp` clean) |
| clippy | `cargo clippy --offline --all-targets -- -D warnings` | **EXIT 0**, 0 warnings (`licence-clippy.log`) |
| formatting | `cargo fmt --check` | **EXIT 0** (`licence-fmt.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | **78,272 B**, sha256 `87cae848f944892453c4261c8309bdf4f88d2836b008cd1dd134a43b582a4c0f`, **61 entries**, `LICENSE` + `LICENSE-NOTICE.md` inside, no `Cargo.lock` (`licence-build.log`) |
| repeatability | a second build, `cmp` | byte-identical (`licence-build.log`); a third build of the tree after the documentation edits also `cmp`s equal to the delivered file |
| registry check | `cargo horizon-mod check twilightforest.hmod` | `twilightforest 0.2.0 (native-mod, …)` **EXIT 0**; `--json` `ok:true`; **235** `uses.json` entries; `fits … --build nightly` → `0b14f8a made for`, **EXIT 0** (`licence-check.log`) |
| in-game leg | `tools/e2e.sh` (unchanged, sha256 `b5a46b71…0936`) | **not run** — `blocked(harness)`, no launcher here; the licence pass touches no rule and claims no game run |

The seven files, and what replaced each, are in `notes.md` §“Licence-clean pass”; the per-file
provenance is `PROVENANCE.md`.

## 1. The pure runtime

| Step | Command | Result |
|---|---|---|
| red | `cargo test --offline` with the rule stubbed (`unimplemented!`) | **12 failed, 0 passed** — `ports/_shared/evidence/twilightforest/runtime-red.log` |
| green | `cargo test --offline` | **14 passed, 0 failed** — `runtime-green.log` |
| fmt | `cargo fmt --check` | clean |
| clippy | `cargo clippy --all-targets --offline -- -D warnings` | clean, no warnings |

The two tests added after the red run are `a_refused_pool_names_the_block_that_stopped_it`
and `the_catalyst_is_the_diamond_the_jar_tag_resolves_to`.

## 2. The staged ladder (retired format)

| Step | Command | Result |
|---|---|---|
| stage | `python3 tools/horizon.py stage` | staged, `staging_seconds` printed |
| check | `python3 tools/horizon.py check` | `cargo_exit: 0` |
| test | `python3 tools/horizon.py test` | **14 passed**, `cargo_exit: 0` (embedded: `game::twilightforest::tests`) |
| build | `python3 tools/horizon.py build` | `cargo_exit: 0`, **0 warnings** in `target/horizon-build.log` |

## 3. In-game legs (`tools/ladder-e2e.sh`, one lease, rebuild included)

`ports/_shared/evidence/twilightforest/e2e.log` — default legs:

```
pool A block counts:      16 twilightforest:twilight_portal
pool A lit 16/16
pool B block counts:      16 minecraft:water
pool B kept 16/16 water
PASS: a diamond lights the framed pool into twilightforest:twilight_portal; a stick does not
setblock twilightforest:twilight_portal with the content blocked: {"chat":["Unknown block type 'twilightforest:twilight_portal'", ...]}
pool with the content blocked:      16 minecraft:water
PASS: without the bundle's content the block is unknown and no portal forms
All legs passed
EXIT:0
```

`ports/_shared/evidence/twilightforest/e2e-selftest.log` — the leg that must fail:

```
FAIL: SELFTEST: the stick pool was expected to light and did not
EXIT:1
```

`python3 ports/tools/check-e2e-logs.py twilightforest --strict` → **exit 0**,
`logs=2 ok=2`.

What the legs prove, and how they could fail:

- a diamond dropped into a framed 4x4 water pool lights **all 16** blocks into
  `twilightforest:twilight_portal` (the block only exists because the bundle's
  embedded content loaded);
- an identically built pool that receives a **stick** stays water — the
  counterfactual: a scenario that ignored the catalyst would light both;
- with `.horizon-cache` blocked (a plain file where the bundle writes its
  content) the block is **unknown** and no portal forms — so the check cannot
  pass without the content half;
- the selftest asserts the opposite of the truth and exits 1, so the assertions
  are capable of failing.

Every pool is verified as an input before the catalyst is spent: the ring is
proved dirt after the ring fill, after the water, after the air fill and after
the flowers, and every pool cell is proved water.

## 4. The dimension block (the 0.1.0 negative result — superseded by §0b)

`ports/_shared/evidence/twilightforest/dimension-probe.log` — the shared,
unmodded 1d63e0a game driven by `horizon-ctl`:

```
command reply: {"chat":["Unknown dimension 'twilightforest:twilight_forest'"],"ok":true}
PASS: twilightforest:twilight_forest is refused and the player stays in minecraft:overworld
command reply: {"chat":["Teleported Player13831 to 0.500000, 100.000000, 0.500000"],"ok":true}
setblock reply: {"block":"minecraft:glass","ok":true}
block at 3 100 3 in the nether: minecraft:glass
PASS: the nether control moved the player and placed a block that read back
PASS: twilightforest:twilight_forest takes no blocks either
EXIT:0
```

The Nether control is what makes the first assertion evidence rather than an
error message: the same `execute in … run tp` path moves a player and writes a
block when the dimension exists.

## 5. Deliverables (retired format)

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `ports/twilightforest/twilightforest.hnmod` | 27,553 B | `e7adff5010e95bc2baf94bd6a839befceff5064d3399c9b320de6d8adbb5711f` | 7 |
| `ports/twilightforest/content/twilightforest.hmod` | 19,044 B | `6d1a4b7e8b8f705361c60dad43d99d48f6f345a1bba1af905e187ba3c89f4f41` | 15 |

`tools/package.sh` run twice gives the same `.hnmod` SHA-256 (deterministic
packaging: fixed dates, sorted entries). The `.hmod` is also at
`mixins/twilightforest/data/twilightforest-content.hmod`, byte-identical (the
packager refuses a stale copy). `cargo horizon-mod check` reports `ok: true` for
both, the `.hnmod` as `native-mod` at format
`native-1/1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9/5a64af938ec86b00d53d16a408aabc0038fb3e57`.

## 6. Gates

| Gate | Result |
|---|---|
| `check-e2e-logs.py twilightforest --strict` | **exit 0** |
| `check-skills.py --notes ports/twilightforest` | **PASS** |
| `check_provenance.py --check` | **exit 0**, 0 identical files (§0c) |
| `upstream_ready.py twilightforest --copyleft` | **exit 1** — the only reason: “status is pending, not ported”. The ledger row is the orchestrator's to change; nothing about the port is missing |

## 7. Evidence files

```
ports/_shared/evidence/twilightforest/jar-inventory.log
ports/_shared/evidence/twilightforest/ladder-check.log
ports/_shared/evidence/twilightforest/ladder-test.log
ports/_shared/evidence/twilightforest/ladder-build.log
ports/_shared/evidence/twilightforest/dimension-probe.log
ports/_shared/evidence/twilightforest/runtime-red.log
ports/_shared/evidence/twilightforest/runtime-green.log
ports/_shared/evidence/twilightforest/e2e.log
ports/_shared/evidence/twilightforest/e2e-selftest.log
```
