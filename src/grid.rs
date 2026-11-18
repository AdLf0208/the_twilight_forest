// The Twilight Forest for Horizon — the dimension's biome grid (pure).
//
// Ported from The Twilight Forest 1.21.1-4.8.3345 (TeamTwilight/twilightforest, LGPL-2.1):
// `world/components/layer/*.java` and `world/components/layer/vanillalegacy/**` (the
// `BiomeDensitySource` / `TFBiomeProvider` stack the jar's
// `data/twilightforest/twilight/biome_layer_stack/*.json` and
// `data/twilightforest/twilight/biome_terrain_data/biome_grid.json` name), read at branch
// `1.21.x`. Upstream source was read for behaviour only; this is a clean-room Rust rewrite.
// LOCAL-ONLY until the maintainer approves publication (ARCHITECTURE.md §8).
//
// Nothing here touches the game: [`key_biome`] answers the grid's landscape cell and
// [`column_biome`] the biome at an elevation, both from the jar's own numbers.
//
// The slice carries **14** of the jar's 22 biomes. The placement is exact for all 22 (the whole
// layer stack is reproduced); a biome the slice does not carry answers the nearest carried biome
// of its own family ([`carried`]), and `notes.md` states that simplification.

/// The jar's biome ids, as the grid can answer them (`biome_landscape`'s keys and their layers).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Biome {
    Clearing,
    DarkForest,
    DarkForestCenter,
    DenseForest,
    DenseMushroomForest,
    EnchantedForest,
    FinalPlateau,
    FireSwamp,
    FireflyForest,
    Forest,
    Glacier,
    Highlands,
    HighlandsUnderground,
    Lake,
    MushroomForest,
    OakSavannah,
    SnowyForest,
    SpookyForest,
    Stream,
    Swamp,
    Thornlands,
    Underground,
}

impl Biome {
    /// The biome's id as the jar declares it.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Clearing => "twilightforest:clearing",
            Self::DarkForest => "twilightforest:dark_forest",
            Self::DarkForestCenter => "twilightforest:dark_forest_center",
            Self::DenseForest => "twilightforest:dense_forest",
            Self::DenseMushroomForest => "twilightforest:dense_mushroom_forest",
            Self::EnchantedForest => "twilightforest:enchanted_forest",
            Self::FinalPlateau => "twilightforest:final_plateau",
            Self::FireSwamp => "twilightforest:fire_swamp",
            Self::FireflyForest => "twilightforest:firefly_forest",
            Self::Forest => "twilightforest:forest",
            Self::Glacier => "twilightforest:glacier",
            Self::Highlands => "twilightforest:highlands",
            Self::HighlandsUnderground => "twilightforest:highlands_underground",
            Self::Lake => "twilightforest:lake",
            Self::MushroomForest => "twilightforest:mushroom_forest",
            Self::OakSavannah => "twilightforest:oak_savannah",
            Self::SnowyForest => "twilightforest:snowy_forest",
            Self::SpookyForest => "twilightforest:spooky_forest",
            Self::Stream => "twilightforest:stream",
            Self::Swamp => "twilightforest:swamp",
            Self::Thornlands => "twilightforest:thornlands",
            Self::Underground => "twilightforest:underground",
        }
    }
}

impl std::fmt::Display for Biome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.id())
    }
}

/// What this slice carries, in the order `possible_biomes` lists them (the index
/// [`carried_index`] answers).
pub const CARRIED: [Biome; 14] = [
    Biome::Forest,
    Biome::Clearing,
    Biome::DarkForestCenter,
    Biome::FinalPlateau,
    Biome::Glacier,
    Biome::FireSwamp,
    Biome::Thornlands,
    Biome::Highlands,
    Biome::HighlandsUnderground,
    Biome::Lake,
    Biome::SnowyForest,
    Biome::Stream,
    Biome::Swamp,
    Biome::Underground,
];

/// The dimension's own biome, used as `DimensionSettings::biome` (the field is unread while a
/// `mod_biome_source` is set, and `forest` is the landscape's most common biome).
pub const DEFAULT: Biome = Biome::Forest;

/// The jar's `biome_landscape` entry for `key`: `(key, depth, scale, weight, biome_layers)`.
///
/// `depth`, `scale` and `weight` are the numbers `biome_grid.json` carries; `biome_layers` is the
/// elevation map, and a quart Y between two keys takes the nearer one (a tie keeps the lower key,
/// as upstream's `TerrainColumn.getBiome` reduce does).
pub struct Landscape {
    pub key: Biome,
    pub depth: f64,
    pub scale: f64,
    pub weight: f64,
    pub layers: &'static [(f64, Biome)],
}

/// The jar's 20 landscape entries, verbatim from `biome_grid.json`.
pub const LANDSCAPE: [Landscape; 20] = [
    Landscape {
        key: Biome::Clearing,
        depth: 0.05,
        scale: 1.15,
        weight: 1.0,
        layers: &[(-3.0, Biome::Underground), (-1.0, Biome::Clearing)],
    },
    Landscape {
        key: Biome::DarkForest,
        depth: 0.1,
        scale: 1.25,
        weight: 0.5,
        layers: &[(-3.0, Biome::Underground), (-1.0, Biome::DarkForest)],
    },
    Landscape {
        key: Biome::DarkForestCenter,
        depth: 0.1,
        scale: 1.125,
        weight: 1.0,
        layers: &[(-3.0, Biome::Underground), (-1.0, Biome::DarkForestCenter)],
    },
    Landscape {
        key: Biome::DenseForest,
        depth: -0.8,
        scale: 3.9,
        weight: 1.0,
        layers: &[(-3.8, Biome::Underground), (-1.8, Biome::DenseForest)],
    },
    Landscape {
        key: Biome::DenseMushroomForest,
        depth: 0.0,
        scale: 1.75,
        weight: 1.0,
        layers: &[
            (-3.0, Biome::Underground),
            (-1.0, Biome::DenseMushroomForest),
        ],
    },
    Landscape {
        key: Biome::EnchantedForest,
        depth: -0.5,
        scale: 4.0,
        weight: 1.0,
        layers: &[(-3.5, Biome::Underground), (-1.5, Biome::EnchantedForest)],
    },
    Landscape {
        key: Biome::FinalPlateau,
        depth: 12.0,
        scale: 0.75,
        weight: 1.0,
        layers: &[(0.0, Biome::FinalPlateau)],
    },
    Landscape {
        key: Biome::FireSwamp,
        depth: 0.2,
        scale: 1.25,
        weight: 1.0,
        layers: &[(-3.0, Biome::Underground), (-1.0, Biome::FireSwamp)],
    },
    Landscape {
        key: Biome::FireflyForest,
        depth: -0.75,
        scale: 4.15,
        weight: 1.0,
        layers: &[(-3.75, Biome::Underground), (-1.75, Biome::FireflyForest)],
    },
    Landscape {
        key: Biome::Forest,
        depth: -0.7,
        scale: 4.2,
        weight: 1.0,
        layers: &[(-3.7, Biome::Underground), (-1.7, Biome::Forest)],
    },
    Landscape {
        key: Biome::Glacier,
        depth: -0.05,
        scale: 1.75,
        weight: 1.0,
        layers: &[(-3.05, Biome::Underground), (-1.05, Biome::Glacier)],
    },
    Landscape {
        key: Biome::Highlands,
        depth: 3.0,
        scale: 2.25,
        weight: 0.135,
        layers: &[
            (-3.0, Biome::HighlandsUnderground),
            (-1.0, Biome::Highlands),
        ],
    },
    Landscape {
        key: Biome::Lake,
        depth: -2.2,
        scale: 2.0,
        weight: 1.0,
        layers: &[(-5.2, Biome::Underground), (-3.2, Biome::Lake)],
    },
    Landscape {
        key: Biome::MushroomForest,
        depth: 0.0,
        scale: 2.0,
        weight: 1.0,
        layers: &[(-3.0, Biome::Underground), (-1.0, Biome::MushroomForest)],
    },
    Landscape {
        key: Biome::OakSavannah,
        depth: -0.05,
        scale: 2.0,
        weight: 1.0,
        layers: &[(-3.05, Biome::Underground), (-1.05, Biome::OakSavannah)],
    },
    Landscape {
        key: Biome::SnowyForest,
        depth: 0.0,
        scale: 2.45,
        weight: 0.5,
        layers: &[(-3.0, Biome::Underground), (-1.0, Biome::SnowyForest)],
    },
    Landscape {
        key: Biome::SpookyForest,
        depth: 0.0,
        scale: 2.25,
        weight: 1.0,
        layers: &[(-3.0, Biome::Underground), (-1.0, Biome::SpookyForest)],
    },
    Landscape {
        key: Biome::Stream,
        depth: -0.1,
        scale: 0.001,
        weight: 1.35,
        layers: &[(-3.1, Biome::Underground), (-1.1, Biome::Stream)],
    },
    Landscape {
        key: Biome::Swamp,
        depth: -0.6,
        scale: 1.7,
        weight: 1.0,
        layers: &[(-3.6, Biome::Underground), (-1.6, Biome::Swamp)],
    },
    Landscape {
        key: Biome::Thornlands,
        depth: 5.5,
        scale: 1.75,
        weight: 1.15,
        layers: &[(0.0, Biome::Thornlands)],
    },
];

/// The landscape entry for a key biome the grid answered.
///
/// # Panics
///
/// Only a [`Biome`] that is not a landscape key panics; every key the layer stack can return has
/// an entry here, and the tests pin that.
#[must_use]
pub fn landscape(key: Biome) -> &'static Landscape {
    LANDSCAPE
        .iter()
        .find(|entry| entry.key == key)
        .expect("every key biome the layer stack answers has a landscape entry")
}

/// The slice's bounded answer for a biome it does not carry: its own family's member.
///
/// The jar's `random_biomes` layer's common and rare lists are the forest family; the one
/// companion pair that leaves it is `dark_forest`, which the layer stack derives from
/// `dark_forest_center`. A carried biome answers itself.
#[must_use]
pub fn carried(biome: Biome) -> Biome {
    match biome {
        Biome::DarkForest => Biome::DarkForestCenter,
        Biome::DenseForest
        | Biome::DenseMushroomForest
        | Biome::EnchantedForest
        | Biome::FireflyForest
        | Biome::MushroomForest
        | Biome::OakSavannah
        | Biome::SpookyForest => Biome::Forest,
        other => other,
    }
}

/// Where a biome sits in [`CARRIED`], which is the index
/// [`noise_biome`](mod_api::BiomeSource::noise_biome) answers.
#[must_use]
pub fn carried_index(biome: Biome) -> usize {
    let wanted = carried(biome);
    CARRIED
        .iter()
        .position(|carried| *carried == wanted)
        .expect("every carried family has a member in CARRIED")
}

/// The level a Twilight Forest portal leads to (and back from).
pub const DIMENSION: &str = "twilightforest:twilight_forest";

/// `LinearCongruentialGenerator.next`: the 64-bit step (wrapping, as Java's `long` arithmetic)
/// every layer random and every layer seed is built from.
#[must_use]
pub fn linear_congruential_next(seed: i64, salt: i64) -> i64 {
    seed.wrapping_mul(
        seed.wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407),
    )
    .wrapping_add(salt)
}

/// `LazyAreaContext`'s seed for one layer: `mixSeed(worldSeed, salt)`.
fn mix_seed(world_seed: i64, salt: i64) -> i64 {
    let mut i = linear_congruential_next(salt, salt);
    i = linear_congruential_next(i, salt);
    i = linear_congruential_next(i, salt);
    let mut j = linear_congruential_next(world_seed, i);
    j = linear_congruential_next(j, i);
    linear_congruential_next(j, i)
}

/// `RandomContext`: the layer's own random, seeded per layer and re-seeded per cell.
struct LayerRandom {
    seed: i64,
    value: i64,
}

impl LayerRandom {
    fn new(seed: i64) -> Self {
        Self { seed, value: 0 }
    }

    /// `initRandom(x, z)`: four steps of the layer seed with the cell's coordinates.
    fn init(&mut self, x: i64, z: i64) {
        let mut i = self.seed;
        i = linear_congruential_next(i, x);
        i = linear_congruential_next(i, z);
        i = linear_congruential_next(i, x);
        i = linear_congruential_next(i, z);
        self.value = i;
    }

    /// `nextRandom(limit)`: a value in `0..limit`, then advance.
    fn next_random(&mut self, limit: i32) -> i32 {
        let result = (self.value >> 24).rem_euclid(i64::from(limit)) as i32;
        self.value = linear_congruential_next(self.value, self.seed);
        result
    }

    fn pick(&mut self, a: Biome, b: Biome) -> Biome {
        if self.next_random(2) == 0 { a } else { b }
    }

    fn pick4(&mut self, a: Biome, b: Biome, c: Biome, d: Biome) -> Biome {
        match self.next_random(4) {
            0 => a,
            1 => b,
            2 => c,
            _ => d,
        }
    }
}

/// `java.util.Random`, which `KeyBiomesLayer` uses directly (not the layer random).
struct JavaRandom {
    seed: i64,
}

impl JavaRandom {
    const MULTIPLIER: i64 = 0x5_DEEC_E66D;
    const ADDEND: i64 = 0xB;
    const MASK: i64 = (1 << 48) - 1;

    fn new(seed: i64) -> Self {
        let mut random = Self { seed: 0 };
        random.set_seed(seed);
        random
    }

    fn set_seed(&mut self, seed: i64) {
        self.seed = (seed ^ Self::MULTIPLIER) & Self::MASK;
    }

    fn next(&mut self, bits: u32) -> i32 {
        self.seed = self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::ADDEND)
            & Self::MASK;
        (self.seed >> (48 - bits)) as i32
    }

    /// `Random.nextInt(bound)` for bound 2 and 3, the two the key layer asks for.
    fn next_int(&mut self, bound: i32) -> i32 {
        debug_assert!(bound > 0);
        let mut result = self.next(31);
        let mask = bound - 1;
        if bound & mask == 0 {
            result = ((i64::from(bound) * i64::from(result)) >> 31) as i32;
        } else {
            let mut candidate = result;
            loop {
                result = candidate % bound;
                if candidate - result + mask >= 0 {
                    break;
                }
                candidate = self.next(31);
            }
        }
        result
    }
}

/// The jar's `random_biomes` layer: a common or a rare biome, one draw each.
const COMMONS: [Biome; 5] = [
    Biome::Forest,
    Biome::DenseForest,
    Biome::MushroomForest,
    Biome::OakSavannah,
    Biome::FireflyForest,
];
const RARES: [Biome; 5] = [
    Biome::Lake,
    Biome::DenseMushroomForest,
    Biome::EnchantedForest,
    Biome::Clearing,
    Biome::SpookyForest,
];
/// Upstream's `rare_biome_chance`.
const RARE_CHANCE: i32 = 15;
/// The jar's `key_biomes` layer: the four landmarks, rotated over 8x8 regions.
const KEYS: [Biome; 4] = [
    Biome::Glacier,
    Biome::FireSwamp,
    Biome::DarkForestCenter,
    Biome::FinalPlateau,
];
/// The jar's `companion_biomes` layer.
const COMPANIONS: [(Biome, Biome); 4] = [
    (Biome::FireSwamp, Biome::Swamp),
    (Biome::Glacier, Biome::SnowyForest),
    (Biome::DarkForestCenter, Biome::DarkForest),
    (Biome::FinalPlateau, Biome::Highlands),
];
/// The jar's `seam` layer: what may not border what without a stream between.
const SEAM_NEIGHBOURS: [Biome; 4] = [
    Biome::Lake,
    Biome::Thornlands,
    Biome::Clearing,
    Biome::OakSavannah,
];
const SEAM_INTERSECTIONS: [(Biome, Biome); 5] = [
    (Biome::SnowyForest, Biome::Glacier),
    (Biome::MushroomForest, Biome::DenseMushroomForest),
    (Biome::Swamp, Biome::FireSwamp),
    (Biome::DarkForest, Biome::DarkForestCenter),
    (Biome::Highlands, Biome::FinalPlateau),
];

/// One transformer of the layer stack.
#[derive(Clone, Copy)]
enum Layer {
    Random,
    Key,
    Companion,
    Zoom { fuzzy: bool },
    Stabilize,
    Border { target: Biome, border: Biome },
    Seam,
    Smooth,
    Filtered { keep: Biome },
}

#[derive(Clone, Copy)]
struct Node {
    salt: i64,
    layer: Layer,
    parents: [Option<usize>; 2],
}

/// The grid: the jar's layer stack, evaluated per call and memoised per `(node, x, z)`.
///
/// A `Grid` belongs to one world seed; the memo is an optimisation only, so the same seed and
/// cell always answer the same biome.
pub struct Grid {
    nodes: Vec<Node>,
    root: usize,
    seed: i64,
    memo: std::collections::HashMap<(u32, i32, i32), Biome>,
}

impl Grid {
    /// The jar's stack: `random_forest_biomes` under `biomes_along_streams`.
    #[must_use]
    pub fn new(seed: i64) -> Self {
        let mut nodes: Vec<Node> = Vec::with_capacity(14);
        let mut push = |salt: i64, layer: Layer, parents: [Option<usize>; 2]| {
            nodes.push(Node {
                salt,
                layer,
                parents,
            });
            nodes.len() - 1
        };
        let random = push(1, Layer::Random, [None, None]);
        let key = push(1000, Layer::Key, [Some(random), None]);
        let companion = push(1000, Layer::Companion, [Some(key), None]);
        let mut zoom = push(1000, Layer::Zoom { fuzzy: false }, [Some(companion), None]);
        zoom = push(1001, Layer::Zoom { fuzzy: false }, [Some(zoom), None]);
        let stabilize = push(700, Layer::Stabilize, [Some(zoom), None]);
        let border = push(
            500,
            Layer::Border {
                target: Biome::FinalPlateau,
                border: Biome::Thornlands,
            },
            [Some(stabilize), None],
        );
        let mut zoom = push(1002, Layer::Zoom { fuzzy: false }, [Some(border), None]);
        zoom = push(1003, Layer::Zoom { fuzzy: false }, [Some(zoom), None]);
        zoom = push(1004, Layer::Zoom { fuzzy: false }, [Some(zoom), None]);
        let landscape = push(1005, Layer::Zoom { fuzzy: false }, [Some(zoom), None]);
        let seams = push(1, Layer::Seam, [Some(landscape), None]);
        let smooth = push(7000, Layer::Smooth, [Some(seams), None]);
        let root = push(
            100,
            Layer::Filtered {
                keep: Biome::Stream,
            },
            [Some(landscape), Some(smooth)],
        );
        Self {
            nodes,
            root,
            seed,
            memo: std::collections::HashMap::new(),
        }
    }

    /// The landscape cell at `(x, z)`: the jar's `BiomeDensitySource.getBiomeColumnKey`.
    pub fn key_biome(&mut self, x: i32, z: i32) -> Biome {
        let root = self.root;
        self.at(root, x, z)
    }

    fn at(&mut self, node: usize, x: i32, z: i32) -> Biome {
        if let Some(cached) = self.memo.get(&(node as u32, x, z)) {
            return *cached;
        }
        let entry = self.nodes[node];
        let seed = mix_seed(self.seed, entry.salt);
        let biome = match entry.layer {
            Layer::Random => {
                let mut random = LayerRandom::new(seed);
                random.init(i64::from(x), i64::from(z));
                if random.next_random(RARE_CHANCE) == 0 {
                    let pick = random.next_random(RARES.len() as i32) as usize;
                    RARES[pick]
                } else {
                    let pick = random.next_random(COMMONS.len() as i32) as usize;
                    COMMONS[pick]
                }
            }
            Layer::Key => {
                let region = self.seed.wrapping_add(
                    i64::from(x & -4)
                        .wrapping_mul(25_117)
                        .wrapping_add(i64::from(z & -4).wrapping_mul(151_121)),
                );
                let mut random = JavaRandom::new(region);
                let offset_x = random.next_int(2) + 1;
                let offset_z = random.next_int(2) + 1;
                let region = self.seed.wrapping_add(
                    i64::from(x / 8)
                        .wrapping_mul(25_117)
                        .wrapping_add(i64::from(z / 8).wrapping_mul(151_121)),
                );
                random.set_seed(region);
                let offset = random.next_int(3);
                if (x & 3) == offset_x && (z & 3) == offset_z {
                    let index = match ((x & 4) == 0, (z & 4) == 0) {
                        (true, true) => offset,
                        (true, false) => offset + 1,
                        (false, true) => offset + 2,
                        (false, false) => offset + 3,
                    };
                    KEYS[(index & 0b11) as usize]
                } else {
                    self.parent(node, 0, x, z)
                }
            }
            Layer::Companion => {
                let up = self.parent(node, 0, x, z - 1);
                let right = self.parent(node, 0, x + 1, z);
                let down = self.parent(node, 0, x, z + 1);
                let left = self.parent(node, 0, x - 1, z);
                let center = self.parent(node, 0, x, z);
                COMPANIONS
                    .iter()
                    .find(|(key, _)| {
                        *key != center
                            && (right == *key || left == *key || up == *key || down == *key)
                    })
                    .map_or(center, |(_, companion)| *companion)
            }
            Layer::Zoom { fuzzy } => {
                let i = self.parent(node, 0, x >> 1, z >> 1);
                let mut random = LayerRandom::new(seed);
                random.init(i64::from(x >> 1 << 1), i64::from(z >> 1 << 1));
                let j = x & 1;
                let k = z & 1;
                if j == 0 && k == 0 {
                    i
                } else {
                    let l = self.parent(node, 0, x >> 1, (z + 1) >> 1);
                    let i1 = random.pick(i, l);
                    if j == 0 && k == 1 {
                        i1
                    } else {
                        let j1 = self.parent(node, 0, (x + 1) >> 1, z >> 1);
                        let k1 = random.pick(i, j1);
                        if j == 1 && k == 0 {
                            k1
                        } else {
                            let l1 = self.parent(node, 0, (x + 1) >> 1, (z + 1) >> 1);
                            if fuzzy {
                                random.pick4(i, j1, l, l1)
                            } else {
                                mode_or_random(&mut random, i, j1, l, l1)
                            }
                        }
                    }
                }
            }
            Layer::Stabilize => {
                let off_x = x.wrapping_shl(4) & 3;
                let off_z = z.wrapping_shl(4) & 3;
                let center_x = ((x + off_x + 1) & !3) - off_x;
                let center_z = ((z + off_z + 1) & !3) - off_z;
                if x <= center_x + 1 && x >= center_x - 1 && z <= center_z + 1 && z >= center_z - 1
                {
                    self.parent(node, 0, center_x, center_z)
                } else {
                    self.parent(node, 0, x, z)
                }
            }
            Layer::Border { target, border } => {
                let north = self.parent(node, 0, x, z - 1);
                let west = self.parent(node, 0, x + 1, z);
                let south = self.parent(node, 0, x, z + 1);
                let east = self.parent(node, 0, x - 1, z);
                let center = self.parent(node, 0, x, z);
                let north_west = self.parent(node, 0, x + 1, z - 1);
                let south_west = self.parent(node, 0, x + 1, z + 1);
                let south_east = self.parent(node, 0, x - 1, z + 1);
                let north_east = self.parent(node, 0, x - 1, z - 1);
                if on_border(target, center, east, west, north, south)
                    || on_border(
                        target, center, north_east, north_west, south_east, south_west,
                    )
                {
                    border
                } else {
                    center
                }
            }
            Layer::Seam => {
                let north = self.parent(node, 0, x, z - 1);
                let east = self.parent(node, 0, x + 1, z);
                let south = self.parent(node, 0, x, z + 1);
                let west = self.parent(node, 0, x - 1, z);
                let center = self.parent(node, 0, x, z);
                if should_partition(center, west)
                    || should_partition(center, east)
                    || should_partition(center, south)
                    || should_partition(center, north)
                {
                    Biome::Stream
                } else {
                    center
                }
            }
            Layer::Smooth => {
                let up = self.parent(node, 0, x, z - 1);
                let right = self.parent(node, 0, x + 1, z);
                let down = self.parent(node, 0, x, z + 1);
                let left = self.parent(node, 0, x - 1, z);
                let center = self.parent(node, 0, x, z);
                let x_match = right == left;
                let z_match = up == down;
                if x_match == z_match {
                    if x_match {
                        let mut random = LayerRandom::new(seed);
                        random.init(i64::from(x), i64::from(z));
                        if random.next_random(2) == 0 { left } else { up }
                    } else {
                        center
                    }
                } else if x_match {
                    left
                } else {
                    up
                }
            }
            Layer::Filtered { keep } => {
                let filtered = self.parent(node, 1, x, z);
                if filtered == keep {
                    filtered
                } else {
                    self.parent(node, 0, x, z)
                }
            }
        };
        self.memo.insert((node as u32, x, z), biome);
        biome
    }

    fn parent(&mut self, node: usize, slot: usize, x: i32, z: i32) -> Biome {
        let parent = self.nodes[node].parents[slot].expect("the node has this parent");
        self.at(parent, x, z)
    }
}

/// Whether a cell's biome must be separated from its neighbour by a stream.
fn should_partition(one: Biome, other: Biome) -> bool {
    if one == other {
        return false;
    }
    if SEAM_NEIGHBOURS
        .iter()
        .any(|held| *held == one || *held == other)
    {
        return false;
    }
    if SEAM_INTERSECTIONS
        .iter()
        .any(|(a, b)| (*a == one && *b == other) || (*a == other && *b == one))
    {
        return false;
    }
    true
}

fn on_border(
    subject: Biome,
    center: Biome,
    first: Biome,
    second: Biome,
    third: Biome,
    fourth: Biome,
) -> bool {
    center != subject
        && (first == subject || second == subject || third == subject || fourth == subject)
}

/// `ZoomLayer.modeOrRandom`: the mode of the four, else a draw.
///
/// Upstream spells the twenty-one cases out one per line; the ones that answer the same biome are
/// merged here, in the same order, so the first matching answer is still the one Java returns.
fn mode_or_random(random: &mut LayerRandom, a: Biome, b: Biome, c: Biome, d: Biome) -> Biome {
    // `b == c && c == d` is asked first: on an all-equal undirected pair Java answers `b`.
    if b == c && c == d {
        return b;
    }
    let picks_a = (a == b && (a == c || a == d || c != d))
        || (a == c && (a == d || b != d))
        || (a == d && b != c);
    if picks_a {
        return a;
    }
    let picks_b = (b == c && a != d) || (b == d && a != c);
    if picks_b {
        return b;
    }
    if c == d && a != b {
        return c;
    }
    random.pick4(a, b, c, d)
}

/// The landscape cell at `(x, z)` for one world seed (`BiomeDensitySource.getBiomeColumnKey`).
#[must_use]
pub fn key_biome(seed: i64, x: i32, z: i32) -> Biome {
    Grid::new(seed).key_biome(x, z)
}

/// The biome at quart elevation `quart_y` of a landscape column: `TerrainColumn.getBiome`,
/// the nearest layer key with a tie keeping the lower key.
#[must_use]
pub fn column_biome(key: Biome, quart_y: i32) -> Biome {
    let mut best: Option<(f64, Biome)> = None;
    for (at, biome) in landscape(key).layers {
        best = match best {
            None => Some((*at, *biome)),
            Some((best_at, best_biome)) => {
                let best_delta = (best_at - f64::from(quart_y)).abs();
                let delta = (*at - f64::from(quart_y)).abs();
                if best_delta <= delta {
                    Some((best_at, best_biome))
                } else {
                    Some((*at, *biome))
                }
            }
        };
    }
    best.map_or(key, |(_, biome)| biome)
}

/// The biome at a quart cell, as the jar's own provider answers `getNoiseBiome`: the landscape
/// cell's column at that elevation, still one of the jar's 22 ids. The source narrows it to a
/// carried biome with [`carried`].
#[must_use]
pub fn biome_at(seed: i64, quart: [i32; 3]) -> Biome {
    let key = key_biome(seed, quart[0], quart[2]);
    column_biome(key, quart[1])
}

/// The `/twilightforest dimension` line: the level, its biome count, the cell, its seed, the
/// biome and the landscape cell it came from.
#[must_use]
pub fn status_line(dimension: &str, pos: [i32; 3], seed: i64, biome: Biome, key: Biome) -> String {
    let landscape = landscape(key);
    format!(
        "{dimension}: {} biomes; cell {} {} {} seed {seed} -> {biome} (grid {key}, depth {}, scale {})",
        CARRIED.len(),
        pos[0],
        pos[1],
        pos[2],
        landscape.depth,
        landscape.scale
    )
}

/// Where a portal in `level` leads: the level's own counterpart, or nowhere.
#[must_use]
pub fn travel_target(level: &str) -> Option<&'static str> {
    match level {
        "minecraft:overworld" => Some(DIMENSION),
        DIMENSION => Some("minecraft:overworld"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Java oracle's table (`ports/_shared/evidence/twilightforest/oracle/grid-golden.txt`):
    /// seed, cell x, cell z, key biome, and the biome at quart Y -5..=1.
    type Golden = (i64, i32, i32, &'static str, [&'static str; 7]);
    const GOLDEN: [Golden; 60] = [
        (
            20261105,
            0,
            0,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            20261105,
            1,
            0,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            20261105,
            0,
            1,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            20261105,
            16,
            16,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            20261105,
            -33,
            77,
            "twilightforest:mushroom_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
            ],
        ),
        (
            20261105,
            1000,
            -1000,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            20261105,
            12345,
            6789,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            20261105,
            -1,
            -1,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            20261105,
            3,
            3,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            20261105,
            255,
            255,
            "twilightforest:forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:forest",
                "twilightforest:forest",
                "twilightforest:forest",
                "twilightforest:forest",
            ],
        ),
        (
            20261105,
            -256,
            512,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            20261105,
            4096,
            8192,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            20261105,
            -20000,
            30000,
            "twilightforest:dense_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
            ],
        ),
        (
            20261105,
            65536,
            -65536,
            "twilightforest:forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:forest",
                "twilightforest:forest",
                "twilightforest:forest",
                "twilightforest:forest",
            ],
        ),
        (
            20261105,
            2047,
            2048,
            "twilightforest:enchanted_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:enchanted_forest",
                "twilightforest:enchanted_forest",
                "twilightforest:enchanted_forest",
                "twilightforest:enchanted_forest",
            ],
        ),
        (
            1,
            0,
            0,
            "twilightforest:mushroom_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
            ],
        ),
        (
            1,
            1,
            0,
            "twilightforest:mushroom_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
            ],
        ),
        (
            1,
            0,
            1,
            "twilightforest:mushroom_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
            ],
        ),
        (
            1,
            16,
            16,
            "twilightforest:mushroom_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
            ],
        ),
        (
            1,
            -33,
            77,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            1,
            1000,
            -1000,
            "twilightforest:mushroom_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
            ],
        ),
        (
            1,
            12345,
            6789,
            "twilightforest:fire_swamp",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:fire_swamp",
                "twilightforest:fire_swamp",
                "twilightforest:fire_swamp",
            ],
        ),
        (
            1,
            -1,
            -1,
            "twilightforest:mushroom_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
            ],
        ),
        (
            1,
            3,
            3,
            "twilightforest:mushroom_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
            ],
        ),
        (
            1,
            255,
            255,
            "twilightforest:mushroom_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
            ],
        ),
        (
            1,
            -256,
            512,
            "twilightforest:firefly_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
            ],
        ),
        (
            1,
            4096,
            8192,
            "twilightforest:forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:forest",
                "twilightforest:forest",
                "twilightforest:forest",
                "twilightforest:forest",
            ],
        ),
        (
            1,
            -20000,
            30000,
            "twilightforest:dense_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
            ],
        ),
        (
            1,
            65536,
            -65536,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            1,
            2047,
            2048,
            "twilightforest:forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:forest",
                "twilightforest:forest",
                "twilightforest:forest",
                "twilightforest:forest",
            ],
        ),
        (
            42,
            0,
            0,
            "twilightforest:firefly_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
            ],
        ),
        (
            42,
            1,
            0,
            "twilightforest:firefly_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
            ],
        ),
        (
            42,
            0,
            1,
            "twilightforest:firefly_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
            ],
        ),
        (
            42,
            16,
            16,
            "twilightforest:firefly_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
            ],
        ),
        (
            42,
            -33,
            77,
            "twilightforest:mushroom_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
            ],
        ),
        (
            42,
            1000,
            -1000,
            "twilightforest:firefly_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
            ],
        ),
        (
            42,
            12345,
            6789,
            "twilightforest:glacier",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:glacier",
                "twilightforest:glacier",
                "twilightforest:glacier",
                "twilightforest:glacier",
            ],
        ),
        (
            42,
            -1,
            -1,
            "twilightforest:firefly_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
            ],
        ),
        (
            42,
            3,
            3,
            "twilightforest:firefly_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
            ],
        ),
        (
            42,
            255,
            255,
            "twilightforest:mushroom_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
            ],
        ),
        (
            42,
            -256,
            512,
            "twilightforest:mushroom_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
            ],
        ),
        (
            42,
            4096,
            8192,
            "twilightforest:dense_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
            ],
        ),
        (
            42,
            -20000,
            30000,
            "twilightforest:dense_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
            ],
        ),
        (
            42,
            65536,
            -65536,
            "twilightforest:lake",
            [
                "twilightforest:underground",
                "twilightforest:lake",
                "twilightforest:lake",
                "twilightforest:lake",
                "twilightforest:lake",
                "twilightforest:lake",
                "twilightforest:lake",
            ],
        ),
        (
            42,
            2047,
            2048,
            "twilightforest:forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:forest",
                "twilightforest:forest",
                "twilightforest:forest",
                "twilightforest:forest",
            ],
        ),
        (
            -7,
            0,
            0,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            -7,
            1,
            0,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            -7,
            0,
            1,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            -7,
            16,
            16,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            -7,
            -33,
            77,
            "twilightforest:mushroom_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
                "twilightforest:mushroom_forest",
            ],
        ),
        (
            -7,
            1000,
            -1000,
            "twilightforest:dense_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
            ],
        ),
        (
            -7,
            12345,
            6789,
            "twilightforest:dark_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:dark_forest",
                "twilightforest:dark_forest",
                "twilightforest:dark_forest",
            ],
        ),
        (
            -7,
            -1,
            -1,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            -7,
            3,
            3,
            "twilightforest:oak_savannah",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
                "twilightforest:oak_savannah",
            ],
        ),
        (
            -7,
            255,
            255,
            "twilightforest:spooky_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:spooky_forest",
                "twilightforest:spooky_forest",
                "twilightforest:spooky_forest",
            ],
        ),
        (
            -7,
            -256,
            512,
            "twilightforest:enchanted_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:enchanted_forest",
                "twilightforest:enchanted_forest",
                "twilightforest:enchanted_forest",
                "twilightforest:enchanted_forest",
            ],
        ),
        (
            -7,
            4096,
            8192,
            "twilightforest:forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:forest",
                "twilightforest:forest",
                "twilightforest:forest",
                "twilightforest:forest",
            ],
        ),
        (
            -7,
            -20000,
            30000,
            "twilightforest:dense_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
            ],
        ),
        (
            -7,
            65536,
            -65536,
            "twilightforest:firefly_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
                "twilightforest:firefly_forest",
            ],
        ),
        (
            -7,
            2047,
            2048,
            "twilightforest:dense_forest",
            [
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:underground",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
                "twilightforest:dense_forest",
            ],
        ),
    ];

    #[test]
    fn the_golden_cells_match_the_java_oracle() {
        for (seed, x, z, key, column) in GOLDEN {
            assert_eq!(key_biome(seed, x, z).id(), key, "seed {seed} cell {x} {z}");
            for (offset, expected) in column.into_iter().enumerate() {
                let quart_y = offset as i32 - 5;
                assert_eq!(
                    biome_at(seed, [x, quart_y, z]).id(),
                    expected,
                    "seed {seed} cell {x} {z} quart y {quart_y}"
                );
            }
        }
    }

    /// Java's `String.hashCode`, which the oracle's digest folds in.
    fn java_hash(id: &str) -> i32 {
        id.encode_utf16().fold(0i32, |hash, unit| {
            hash.wrapping_mul(31).wrapping_add(i32::from(unit))
        })
    }

    #[test]
    fn a_wide_sample_matches_the_oracles_digest_and_distribution() {
        // The oracle walks the same 4,096 spread cells
        // (`ports/_shared/evidence/twilightforest/oracle/grid-golden.txt`); a window this wide is
        // what reaches the key biomes, because one base cell of the stack is 64 cells across.
        const COUNTS: [(&str, u32); 20] = [
            ("twilightforest:clearing", 34),
            ("twilightforest:dark_forest", 256),
            ("twilightforest:dark_forest_center", 56),
            ("twilightforest:dense_forest", 498),
            ("twilightforest:dense_mushroom_forest", 32),
            ("twilightforest:enchanted_forest", 29),
            ("twilightforest:final_plateau", 72),
            ("twilightforest:fire_swamp", 49),
            ("twilightforest:firefly_forest", 496),
            ("twilightforest:forest", 446),
            ("twilightforest:glacier", 49),
            ("twilightforest:highlands", 197),
            ("twilightforest:lake", 30),
            ("twilightforest:mushroom_forest", 487),
            ("twilightforest:oak_savannah", 514),
            ("twilightforest:snowy_forest", 257),
            ("twilightforest:spooky_forest", 27),
            ("twilightforest:stream", 202),
            ("twilightforest:swamp", 284),
            ("twilightforest:thornlands", 81),
        ];
        let mut counts: std::collections::BTreeMap<&'static str, u32> =
            std::collections::BTreeMap::new();
        let mut digest: i64 = 1_125_899_906_842_597;
        let mut walk: i64 = 88_172_645_463_325_252;
        // One grid per 64 cells keeps the memo small; the answer does not depend on it.
        let mut grid = Grid::new(20261105);
        for index in 0..4096 {
            if index % 64 == 0 {
                grid = Grid::new(20261105);
            }
            walk = linear_congruential_next(walk, i64::from(index));
            let x = walk.rem_euclid(16_384) as i32 - 8192;
            walk = linear_congruential_next(walk, i64::from(index) + 1);
            let z = walk.rem_euclid(16_384) as i32 - 8192;
            let biome = grid.key_biome(x, z);
            *counts.entry(biome.id()).or_insert(0) += 1;
            digest = digest
                .wrapping_mul(31)
                .wrapping_add(i64::from(java_hash(biome.id())));
        }
        assert_eq!(digest, 7_233_979_160_456_314_091, "the oracle's digest");
        let expected: std::collections::BTreeMap<&'static str, u32> = COUNTS.into_iter().collect();
        assert_eq!(counts, expected);
    }

    #[test]
    fn the_grid_is_deterministic_and_seed_dependent() {
        assert_eq!(key_biome(7, 123, -456), key_biome(7, 123, -456));
        let a: Vec<Biome> = (0..64).map(|x| key_biome(1, x * 16, 0)).collect();
        let b: Vec<Biome> = (0..64).map(|x| key_biome(2, x * 16, 0)).collect();
        assert_ne!(a, b, "a different seed gives a different landscape");
    }

    #[test]
    fn a_biome_outside_the_slice_answers_its_own_family() {
        assert_eq!(carried(Biome::Forest), Biome::Forest);
        assert_eq!(carried(Biome::DarkForest), Biome::DarkForestCenter);
        for family in [
            Biome::DenseForest,
            Biome::DenseMushroomForest,
            Biome::EnchantedForest,
            Biome::FireflyForest,
            Biome::MushroomForest,
            Biome::OakSavannah,
            Biome::SpookyForest,
        ] {
            assert_eq!(carried(family), Biome::Forest, "{family}");
        }
        assert_eq!(carried(Biome::FinalPlateau), Biome::FinalPlateau);
    }

    #[test]
    fn every_carried_biome_answers_itself_and_a_distinct_index() {
        for (index, biome) in CARRIED.into_iter().enumerate() {
            assert_eq!(carried(biome), biome, "{biome}");
            assert_eq!(carried_index(biome), index, "{biome}");
        }
        assert_eq!(CARRIED.len(), 14);
    }

    #[test]
    fn the_column_takes_the_nearest_layer_and_a_tie_keeps_the_lower_key() {
        // `clearing` is -1.0 / -3.0; a quart Y of -2 is equidistant and upstream's reduce keeps
        // the first (lower) key, so `underground` wins.
        assert_eq!(column_biome(Biome::Clearing, -2), Biome::Underground);
        assert_eq!(column_biome(Biome::Clearing, -1), Biome::Clearing);
        assert_eq!(column_biome(Biome::Clearing, 0), Biome::Clearing);
        assert_eq!(column_biome(Biome::Clearing, -3), Biome::Underground);
        // `forest` is -1.7 / -3.7: -2 is nearer -1.7.
        assert_eq!(column_biome(Biome::Forest, -1), Biome::Forest);
        assert_eq!(column_biome(Biome::Forest, -2), Biome::Forest);
        // -3 is nearer the -3.7 layer than the -1.7 one, and -4 is past both.
        assert_eq!(column_biome(Biome::Forest, -3), Biome::Underground);
        assert_eq!(column_biome(Biome::Forest, -4), Biome::Underground);
        // `final_plateau` has one layer, so every elevation answers it.
        assert_eq!(column_biome(Biome::FinalPlateau, -8), Biome::FinalPlateau);
        // `highlands` has its own underground.
        assert_eq!(
            column_biome(Biome::Highlands, -3),
            Biome::HighlandsUnderground
        );
    }

    #[test]
    fn every_landscape_entry_is_a_distinct_key_with_ordered_layers() {
        for (index, entry) in LANDSCAPE.iter().enumerate() {
            assert!(
                !LANDSCAPE[..index]
                    .iter()
                    .any(|other| other.key == entry.key),
                "duplicate key {}",
                entry.key
            );
            assert!(!entry.layers.is_empty(), "{} has no layers", entry.key);
            for pair in entry.layers.windows(2) {
                assert!(
                    pair[0].0 < pair[1].0,
                    "{} layers are not ascending",
                    entry.key
                );
            }
        }
        assert_eq!(LANDSCAPE.len(), 20);
    }

    #[test]
    fn the_linear_congruential_step_is_the_vanilla_one() {
        assert_eq!(linear_congruential_next(0, 0), 0);
        assert_eq!(linear_congruential_next(0, 5), 5);
        assert_eq!(linear_congruential_next(1, 0), 7_806_831_264_735_756_412);
    }

    #[test]
    fn the_readout_names_the_dimension_the_count_the_cell_and_the_grid() {
        let line = status_line(
            "twilightforest:twilight_forest",
            [0, 64, 0],
            20261105,
            Biome::Forest,
            Biome::OakSavannah,
        );
        assert!(line.contains("twilightforest:twilight_forest"), "{line}");
        assert!(line.contains("14 biomes"), "{line}");
        assert!(line.contains("0 64 0"), "{line}");
        assert!(line.contains("twilightforest:forest"), "{line}");
        assert!(line.contains("twilightforest:oak_savannah"), "{line}");
        assert!(line.contains("20261105"), "{line}");
    }

    #[test]
    fn the_portal_returns_to_the_overworld_and_leaves_it() {
        assert_eq!(
            travel_target("minecraft:overworld"),
            Some("twilightforest:twilight_forest")
        );
        assert_eq!(
            travel_target("twilightforest:twilight_forest"),
            Some("minecraft:overworld")
        );
        assert_eq!(travel_target("minecraft:the_nether"), None);
    }
}
