# The Twilight Forest — findings (jar first, then upstream source)

Probe slice, 2026-11-11. Pin `1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`, mixins
`5a64af938ec86b00d53d16a408aabc0038fb3e57`.

- **Jar:** `ports/_shared/licence-cache/twilightforest-1.21.1-4.8.3345-universal.jar`
  (27,154,121 B). `unzip -l` says **11,184 files** (the brief's number confirmed).
  Licence read from its own `META-INF/neoforge.mods.toml`:
  `license = "GNU LESSER GENERAL PUBLIC LICENSE Version 2.1"` → **LGPL-2.1, rank 1**
  (publishable under the same licence with the complete source).
- **Upstream source read** (analysis only, clean-room rewrite):
  `TeamTwilight/twilightforest`, branch `1.21.x`, at the commit that last touched each
  file before the jar's build date (2025-12-27):
  - `block/TFPortalBlock.java` @ `45bf1e3e1c855288a75e74052adc8a189fe13dcd` (2025-11-30)
  - `events/TFTickHandler.java` @ the same commit (7217 B)
  - `config/TFConfig.java` @ the same commit
  - `init/TFBlocks.java` @ `1.21.x` head `2479136accd8d7e486ca9849123b9ab32c1a26ac`
  The head of `TFPortalBlock.java` (2026-06-03) differs only in the portal-wait
  game rules (`getPortalTransitionTime`) and the `DimensionTransition` names; the
  **activation rule below is byte-identical in both**, so the rule is not at risk from
  picking the wrong revision.

## 1. What the jar actually contains (families, measured)

| Area | Count | Notes |
|---|---:|---|
| `data/twilightforest/**` | 3,141 files | the whole dimension plus progression |
| `assets/twilightforest/**` | 5,518 files | models, textures, lang, blockstates |
| `twilightforest/**/*.class` | ~1,900 | the mod's code (never decompiled here) |
| `data/twilightforest/dimension/` | **1** | `twilight_forest.json` |
| `data/twilightforest/dimension_type/` | **1** | `twilight_forest_type.json` |
| `data/twilightforest/worldgen/biome/` | **21** | `forest`, `dense_forest`, `dark_forest`, `fire_swamp`, `highlands`, `glacier`, `lake`, `stream`, … |
| `data/twilightforest/worldgen/noise_settings/` | **2** | `twilight_noise_gen`, `skylight_noise_gen` |
| `data/twilightforest/tags/block/portal/` | 5 | `fluid`, `edge`, `decoration`, `generated_decoration` (+ the block tag dir) |
| `data/twilightforest/tags/item/portal/` | 1 | `activator` |

The dimension file, verbatim from the jar:

```json
{ "type": "twilightforest:twilight_forest_type",
  "generator": { "type": "minecraft:noise",
    "biome_source": { "type": "twilightforest:twilight_biomes",
                      "terrain_data": "twilightforest:biome_grid" },
    "settings": "twilightforest:twilight_noise_gen" } }
```

Its `dimension_type` (`twilight_forest_type.json`): `fixed_time: 13000`,
`ambient_light: 0.01`, `height`/`logical_height` **288**, `min_y` **-32**,
`coordinate_scale` **0.125**, `has_skylight` true, `has_ceiling` false,
`monster_spawn_light_level` uniform 0–7, `effects: twilightforest:renderer`.

**Measured consequence:** the dimension is not a data-only shape. Its generator
names a **custom biome source type** (`twilightforest:twilight_biomes`) carrying a
`terrain_data` pointer (`twilightforest:biome_grid`) — a code-registered
`BiomeSource` codec, not a vanilla one. Even the "JSON part" of this dimension
therefore needs mod code, before the missing engine surface is reached at all.

## 2. The portal mechanic (the part this slice can carry)

Tags, verbatim from the jar:

- `item/portal/activator` → `#c:gems/diamond`
- `block/portal/fluid` (the pool) → `minecraft:water`
- `block/portal/edge` → `minecraft:farmland`, `minecraft:dirt_path`, `#minecraft:dirt`
- `block/portal/decoration` (what must stand on the edge) → 24 vanilla entries plus
  `#minecraft:flowers`, `#minecraft:leaves`, `#minecraft:saplings`, `#minecraft:crops`
  and the mod's own plants

The block (`init/TFBlocks.java:36`):

```java
register("twilight_portal", TFPortalBlock::new,
  () -> Properties.of().pushReaction(PushReaction.BLOCK).strength(-1.0F)
        .sound(SoundType.GLASS).lightLevel(state -> 11)
        .noCollission().noOcclusion().noLootTable());
```

`TFPortalBlock.java` (the constants this port's runtime implements):

- `MIN_PORTAL_SIZE = 4` pool blocks;
- `TFConfig.maxPortalSize = 64` (default) — the flood-fill's budget;
- `canFormPortal(state)` = state is the pool tag, or this portal block with
  `is_one_way = true`;
- `recursivelyValidatePortal`: from a pool block with a **sturdy face above the block
  below**, walk the four horizontal neighbours. A neighbour is
  - another pool block with a sturdy block below → marked `true`, recursed into;
  - an **edge** block (grass/dirt/farmland/dirt path) with a **nature** block above
    → marked `false` (part of the frame, not lit);
  - anything else → the whole check fails.
- `tryToCreatePortal` needs `size >= 4`; then it shrinks the catalyst item by one,
  summons lightning at the pool (`destructivePortalLightning` default true), and
  replaces every `true`-marked block with `twilightforest:twilight_portal` at
  `UPDATE_CLIENTS`.
- The trigger is `TFTickHandler.playerTick` (`PlayerTickEvent.Post`): when
  `tickCount % 20 == 0` (or `% 100` with `checkPortalPlacement` off), scan
  `level.getEntitiesOfClass(ItemEntity.class, player.getBoundingBox().inflate(32))`
  (4 blocks when portal creation is restricted to admins and above) for an item in
  the activator tag, standing in a `canFormPortal` block, **owned by that player**.
- The portal block: light 11, no collision unless `is_one_way`, outline 0–13/16 high,
  `getFluidState` = flowing water at level 1, waits 60 ticks
  (`getPortalTransitionTime`) before a traveller leaves, and
  `getPortalDestination` sends them to `twilightforest:twilight_forest` (or back to
  `TFConfig.originDimension`, default the Overworld), scaled by
  `DimensionType.getTeleportationScale`.

## 3. What the pin cannot host (measured there, not guessed)

The engine's dimension surface at this pin is **three fixed levels**, not a registry:

- `crates/horizon-client/src/game.rs:3513-3540`: `enum Dim { Overworld, Nether, End }`;
  `Dim::from_id` maps every other id to `Overworld`.
- `crates/horizon-client/src/game/commands/execute.rs:623-636`: `execute in <dim>`
  refuses any id outside the three with `argument.dimension.invalid`.
- `crates/horizon-client/src/game/levels.rs:699`: the server ticks
  `[Dim::Overworld, Dim::Nether, Dim::End]`; `Parked` levels are keyed by that enum.
- `crates/horizon-world/src/level/mod.rs:173-190`: the world crate's own
  `Dimension` enum is the same three, and `of_level` says "custom levels map to the
  Overworld".
- `docs/modpacks.md:183`: **Dimensions: dimension types, levels, chunk generators —
  0.5** (planned), "JSON part with worldgen; custom generators as mod code".
- `docs/world-generation.md` ("Data packs"): a pack's **new biomes** are *left out*
  ("the game has no id, colours or sounds for them; their tag entries and climate
  points go with them"). One of the jar's 21 biomes therefore cannot be registered
  by content either.
- The jar's data is **1.21.1** format; 26.3 reworked the worldgen registries
  (`docs/datapack-upgrade.md`), and the upgrade tool rewrites *vanilla-shaped* packs
  only — it has no rewrite for `twilightforest:twilight_biomes`.

So the brief's own proof step (`ctl command "execute in twilightforest:twilight_forest
run tp @s …"`) cannot succeed at this pin. Evidence: `ports/_shared/evidence/
twilightforest/dimension-probe.log` (the TF id refused, the Nether control accepted,
a block placed and read back there).

## 4. The slice chosen from the artefact

The portal **formation + the mod's own portal block**, on the Overworld, with the
dimension half outside this slice:

- `src/runtime.rs`: the activation rule exactly as §2 (pool/edge/nature flood fill,
  `MIN_PORTAL_SIZE`, `maxPortalSize`), pure and unit-tested (byte-identical to the
  0.1.0 helper `mixins/twilightforest/runtime.rs`, now retired);
- `src/portal.rs` (was `mixins/twilightforest/support.rs`): `player_tick_post` + the
  item-entity scan through `LevelRef::entities_in`, the fill via `LevelRef::set_block`,
  the catalyst consumed with `ItemEntityRef::set_stack` / `EntityRef::discard`;
- `assets/` + `data/` (was `content/`): the mod's own `twilightforest:twilight_portal`
  block, model, texture and lang line, carried from the jar.

At the campaign pin the dimension half was `blocked(worldgen-api)`; the SDK nightly
has the API (`register_dimension`, `register_chunk_generator`,
`register_biome_source`, `EntityRef::teleport_to_level`), so it is a re-slice now —
see `notes.md` §"The dimension".

Gaps (also in `notes.md`): the dimension and its chunk generator, biome source and
structure; the travel (`Portal#getPortalDestination` → a level that does not exist);
`neighborChanged` reversion of an unsupported portal back to water; the custom
portal renderer; the advancement lock; the `is_one_way` return-portal rule; the
`#c:gems/diamond` tag's non-vanilla members.
