//! The dimension's game-side glue: the biome source, the level's registration and the readout.
//!
//! [`crate::grid`] decides; this module answers the game's `BiomeSource` calls from it, registers
//! the level and answers `/twilightforest dimension`. Every number below is the jar's, read from
//! `data/twilightforest/dimension_type/twilight_forest_type.json` and
//! `data/twilightforest/dimension/twilight_forest.json`; `notes.md` records the two fields the
//! SDK cannot express and the terrain gap.
//!
//! Upstream: `world/components/biomesources/TFBiomeProvider` (a Java `BiomeSource` over
//! `twilightforest:twilight_biomes`, whose `terrain_data` is `twilightforest:biome_grid`) and
//! `world/components/layer/BiomeDensitySource` (the grid behind it).

use std::collections::HashMap;
use std::sync::Mutex;

use mod_api::{
    BiomeSource, ChunkAccess, ChunkGenerator, Command, CommandArgumentValue, DimensionSettings,
    Flow, RegisterCommandsEvent, Registrar, RunModCommandEvent, ServerHost, Text,
};

use crate::grid::{self, Biome, CARRIED};

/// The level the jar ships (`data/twilightforest/dimension/twilight_forest.json`).
pub const DIMENSION: &str = grid::DIMENSION;
/// The mod's biome source (`"biome_source": {"type": "twilightforest:twilight_biomes"}`).
pub const BIOME_SOURCE: &str = "twilightforest:twilight_biomes";
/// The chunk generator this slice registers in place of the jar's expressible terrain.
pub const CHUNK_GENERATOR: &str = "twilightforest:twilight_terrain";
/// The jar's noise settings, carried converted for the surface rule and a later terrain slice.
pub const NOISE_SETTINGS: &str = "twilightforest:twilight_noise_gen";

/// How many landscape columns the source remembers. Upstream's `LazyArea` keeps a bounded cache
/// for the same reason: the layer stack is evaluated per quart cell and a chunk asks for many.
const COLUMN_CACHE: usize = 8192;

/// The readout's default seed, so `/twilightforest dimension 0 0 0` answers the same line the
/// unit tests assert. The biome source itself uses the level's real seed.
pub const PROBE_SEED: i64 = 20_261_105;

/// The mod's biome source: the jar's grid, narrowed with [`grid::carried`] to the biomes this
/// slice carries.
///
/// A biome source keeps no mutable state the game can observe: the cache only remembers answers,
/// so the same seed and cell always give the same biome.
#[derive(Default)]
pub struct TwilightBiomes {
    columns: Mutex<HashMap<(i64, i32, i32), Biome>>,
}

impl TwilightBiomes {
    /// The landscape cell at `(x, z)`, cached per `(seed, x, z)`.
    fn key_biome(&self, seed: i64, x: i32, z: i32) -> Biome {
        if let Ok(cache) = self.columns.lock()
            && let Some(found) = cache.get(&(seed, x, z))
        {
            return *found;
        }
        let key = grid::key_biome(seed, x, z);
        if let Ok(mut cache) = self.columns.lock() {
            if cache.len() >= COLUMN_CACHE {
                cache.clear();
            }
            cache.insert((seed, x, z), key);
        }
        key
    }
}

impl BiomeSource for TwilightBiomes {
    fn possible_biomes(&self) -> Vec<String> {
        CARRIED.iter().map(|biome| biome.id().to_owned()).collect()
    }

    fn noise_biome(&self, quart: [i32; 3], seed: i64) -> u16 {
        let key = self.key_biome(seed, quart[0], quart[2]);
        let biome = grid::carried(grid::column_biome(key, quart[1]));
        grid::carried_index(biome) as u16
    }
}

/// The level's terrain.
///
/// The jar's terrain is the `minecraft:noise` generator over `twilightforest:twilight_noise_gen`,
/// whose density graph names two density-function types this pin has no code for
/// (`twilightforest:biome_driven_terrain` and `twilightforest:biome_driven_noise`). A density
/// registry loads atomically, so the graph cannot be carried as data, and this slice registers a
/// chunk generator instead: the level exists, its biomes and its dimension type are the jar's,
/// and its terrain is empty until a slice can express the density (notes.md).
pub struct TwilightTerrain;

impl ChunkGenerator for TwilightTerrain {
    fn build_terrain(&self, _chunk: &mut dyn ChunkAccess) {}
}

/// The jar's `twilight_forest_type`, field for field.
///
/// `has_fixed_time` is the jar's `fixed_time: 13000`; the value itself has no field here and
/// `day_cycle: "none"` keeps the clock still. The jar's `effects: twilightforest:renderer` is a
/// client effect id with no field either. Both are recorded in `notes.md`.
#[must_use]
pub fn settings() -> DimensionSettings {
    DimensionSettings {
        // "min_y": -32, "height": 288, "logical_height": 288
        min_y: -32,
        height: 288,
        logical_height: 288,
        // "has_skylight": true, "has_ceiling": false
        has_skylight: true,
        has_ceiling: false,
        // "generator": {"type": "minecraft:noise", "settings": "twilightforest:twilight_noise_gen"},
        // with the biome source below; see `TwilightTerrain` for why the generator is this mod's.
        generator: CHUNK_GENERATOR.to_owned(),
        noise_settings: NOISE_SETTINGS.to_owned(),
        biome: grid::DEFAULT.id().to_owned(),
        mod_biome_source: Some(BIOME_SOURCE.to_owned()),
        // "fixed_time": 13000
        has_fixed_time: true,
        day_cycle: "none".to_owned(),
        // "coordinate_scale": 0.125
        coordinate_scale: 0.125,
        // "ambient_light": 0.01
        ambient_light: 0.01,
        // "infiniburn": "#minecraft:infiniburn_overworld"
        infiniburn: "#minecraft:infiniburn_overworld".to_owned(),
        // "monster_spawn_light_level": uniform 0..=7, "monster_spawn_block_light_limit": 7
        monster_spawn_light_level: (0, 7),
        monster_spawn_block_light_limit: 7,
        // "has_raids": false
        can_start_raid: false,
        // "ultrawarm": false
        water_evaporates: false,
        fast_lava: false,
        // "respawn_anchor_works": false
        respawn_anchor_works: false,
        // "piglin_safe": false
        piglins_zombify: true,
        // "bed_works": true
        bed_can_sleep: "when_dark".to_owned(),
        bed_can_set_spawn: "always".to_owned(),
        bed_destroy_on_use: false,
        // The jar's `noise_settings` say `"sea_level": 0`, and a mod generator has to state it:
        // the game cannot read a sea level from a generator's terrain.
        sea_level: Some(0),
        ..DimensionSettings::default()
    }
}

/// Once, at startup: the biome source, the chunk generator and the dimension, in that order, so
/// the dimension can name both.
pub fn register(r: &mut dyn Registrar) {
    r.register_biome_source(BIOME_SOURCE, TwilightBiomes::default());
    r.register_chunk_generator(CHUNK_GENERATOR, TwilightTerrain);
    r.register_dimension("twilight_forest", settings());
}

/// `/twilightforest dimension [<x> <y> <z> <seed>]`: the registered level, its biome count and the
/// grid's answer for a cell.
pub fn register_commands(event: &mut RegisterCommandsEvent<'_>) {
    event.commands.push(Command::new(
        "twilightforest dimension [<x:integer> <y:integer> <z:integer> <seed:integer>]",
    ));
}

/// Answer `dimension`, and let every other usage through.
pub fn run_command(host: &mut dyn ServerHost, event: &mut RunModCommandEvent<'_>) -> Flow {
    const NAME: &str = "twilightforest";
    if event.command.literals.first().map(String::as_str) != Some(NAME) {
        return Flow::Continue;
    }
    if event.command.literals.get(1).map(String::as_str) != Some("dimension") {
        return Flow::Continue;
    }
    let (pos, seed) = match (
        integer(event.command, "x"),
        integer(event.command, "y"),
        integer(event.command, "z"),
        integer(event.command, "seed"),
    ) {
        (Some(x), Some(y), Some(z), Some(seed)) => ([x, y, z], i64::from(seed)),
        // No position: the invocation's own, as `/execute positioned` set it.
        _ => {
            let at = event.command.position;
            (
                [
                    at[0].floor() as i32,
                    at[1].floor() as i32,
                    at[2].floor() as i32,
                ],
                PROBE_SEED,
            )
        }
    };
    let quart = [pos[0] >> 2, pos[1] >> 2, pos[2] >> 2];
    let biome = grid::carried(grid::biome_at(seed, quart));
    let key = grid::key_biome(seed, quart[0], quart[2]);
    let line = grid::status_line(DIMENSION, pos, seed, biome, key);
    let _ = host.commands().feedback(Text::literal(line));
    Flow::Cancel
}

fn integer(command: &mod_api::CommandInvocation, name: &str) -> Option<i32> {
    match command.argument(name) {
        Some(CommandArgumentValue::Integer(value)) => Some(*value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dimension_type_is_the_jars() {
        let settings = settings();
        assert_eq!(settings.min_y, -32);
        assert_eq!(settings.height, 288);
        assert_eq!(settings.logical_height, 288);
        assert!(settings.has_skylight);
        assert!(!settings.has_ceiling);
        assert!(settings.has_fixed_time);
        assert!((settings.coordinate_scale - 0.125).abs() < f64::EPSILON);
        assert!((settings.ambient_light - 0.01).abs() < f32::EPSILON);
        assert_eq!(settings.infiniburn, "#minecraft:infiniburn_overworld");
        assert_eq!(settings.monster_spawn_light_level, (0, 7));
        assert_eq!(settings.monster_spawn_block_light_limit, 7);
        assert!(!settings.can_start_raid);
        assert!(!settings.respawn_anchor_works);
        assert_eq!(settings.sea_level, Some(0));
        assert_eq!(settings.mod_biome_source.as_deref(), Some(BIOME_SOURCE));
        assert_eq!(settings.generator, CHUNK_GENERATOR);
    }

    #[test]
    fn the_registered_settings_pass_the_sdks_own_validation() {
        let settings = settings();
        assert_eq!(settings.invalid_field(), None);
        assert_eq!(settings.invalid_biome_source(), None);
    }

    #[test]
    fn the_source_lists_the_carried_biomes_and_answers_the_grid() {
        let source = TwilightBiomes::default();
        let biomes = source.possible_biomes();
        assert_eq!(biomes.len(), CARRIED.len());
        assert!(biomes.contains(&"twilightforest:forest".to_owned()));
        // The same cell the readout uses, through the source's own path.
        let index = usize::from(source.noise_biome([0, 16, 0], PROBE_SEED));
        assert!(index < biomes.len());
        assert_eq!(biomes[index], "twilightforest:forest");
        // A cached answer is the same answer.
        assert_eq!(source.noise_biome([0, 16, 0], PROBE_SEED), index as u16);
    }
}
