// The Twilight Forest for Horizon — the portal's activation rule (pure).
//
// Ported from The Twilight Forest 1.21.1-4.8.3345 (TeamTwilight/twilightforest,
// LGPL-2.1), `block/TFPortalBlock.java` @ `45bf1e3e` and
// `events/TFTickHandler.java` @ the same commit. Upstream source was read for
// behaviour only; this is a clean-room Rust rewrite. LOCAL-ONLY until the
// maintainer approves publication (ARCHITECTURE.md §8).
//
// The dimension the portal is meant to reach cannot exist at this pin (see
// `notes.md`), so this slice carries the mechanic that can be verified in game:
// a 2x2-or-larger water pool, framed by grass/dirt/farmland/dirt-path blocks
// with a plant on each, is lit into the mod's own `twilightforest:twilight_portal`
// block when a diamond is dropped in.
//
// Nothing here touches the game: the caller answers what a block is and the
// rule decides whether a portal forms and which blocks it lights.

/// Upstream `TFPortalBlock.MIN_PORTAL_SIZE`: the fewest pool blocks a portal
/// may have. Upstream reads it as `size >= MIN_PORTAL_SIZE` after the flood
/// fill, so 3 water blocks never light.
pub const MIN_PORTAL_SIZE: usize = 4;

/// Upstream `TFConfig.maxPortalSize`'s default: the flood fill's budget in pool
/// blocks. Upstream increments a counter on every visited cell and gives up
/// once it passes this, so a pool of this many lights and one more does not.
pub const DEFAULT_MAX_PORTAL_SIZE: usize = 64;

/// The block a lit pool becomes (`TFBlocks.TWILIGHT_PORTAL`).
pub const PORTAL_BLOCK: &str = "twilightforest:twilight_portal";

/// The portal block's own state property (`TFPortalBlock.DISALLOW_RETURN`):
/// a return portal counts as a pool, so a second diamond can relight across it.
pub const ONE_WAY_PROPERTY: &str = "is_one_way";

/// What the rule sees at a position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    /// A pool block (upstream's `PORTAL_POOL` tag: water) standing on something
    /// whose top face is sturdy, so it does not drain.
    Pool,
    /// A frame block (upstream's `PORTAL_EDGE` tag) with a plant above it
    /// (upstream's `PORTAL_DECO` tag). Never lit.
    Frame,
    /// Anything else: air, stone, a pool block without ground, a bare dirt
    /// wall. One of these beside a pool fails the whole check.
    Other,
}

/// The catalyst, as the jar's `#twilightforest:portal/activator` resolves in
/// vanilla 26.3: that tag is `#c:gems/diamond`, and nothing in the base game
/// joins `#c:gems/diamond` but the diamond itself. A pack that added a member
/// would need this list extended (notes.md, deviations).
pub const ACTIVATORS: &[&str] = &["minecraft:diamond"];

/// Whether a dropped item is a portal catalyst (`ItemStack.is(PORTAL_ACTIVATOR)`).
#[must_use]
pub fn is_activator(item: &str) -> bool {
    ACTIVATORS.contains(&item)
}

/// Upstream `canFormPortal`: the block a catalyst must be standing in. A pool
/// block always qualifies; the portal block itself qualifies only as a return
/// portal (`is_one_way = true`).
#[must_use]
pub fn can_form_portal(pool: bool, portal: bool, one_way: bool) -> bool {
    pool || (portal && one_way)
}

/// Whether a block's collision shape has a full top face, which is what
/// upstream asks of the block under a pool (`isFaceSturdy(level, pos, UP)`).
/// `boxes` are block-local boxes as `[min_x, min_y, min_z, max_x, max_y, max_z]`.
///
/// A full cube is a sturdy floor. A slab, a fence, air or a plant is not: the
/// pool would drain through the face it does not cover.
#[must_use]
pub fn is_sturdy_floor(boxes: &[[f64; 6]]) -> bool {
    const EPSILON: f64 = 1.0e-4;
    boxes.iter().any(|b| {
        b[0] <= EPSILON
            && b[2] <= EPSILON
            && b[3] >= 1.0 - EPSILON
            && b[5] >= 1.0 - EPSILON
            && (b[4] - 1.0).abs() <= EPSILON
    })
}

/// The rule itself: the pool blocks a candidate start would light, or the
/// neighbour that stopped it. Upstream returns `false` from the recursion at the
/// same neighbour; the position is what a game log needs.
///
/// # Errors
///
/// The first neighbour that is neither pool nor frame, the start itself when it
/// is not a pool block, or the start when the pool is too small or too large.
#[must_use = "a refused pool is the answer"]
pub fn light_pool(
    start: [i32; 3],
    max: usize,
    mut at: impl FnMut([i32; 3]) -> Cell,
) -> Result<Vec<[i32; 3]>, ([i32; 3], Cell)> {
    if at(start) != Cell::Pool {
        return Err((start, at(start)));
    }
    let mut lit = vec![start];
    let mut seen = vec![start];
    let mut queue = vec![start];
    while let Some(pos) = queue.pop() {
        for step in [[0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]] {
            let next = [pos[0] + step[0], pos[1] + step[1], pos[2] + step[2]];
            if seen.contains(&next) {
                continue;
            }
            match at(next) {
                Cell::Pool => {
                    if lit.len() >= max {
                        return Err((next, Cell::Pool));
                    }
                    seen.push(next);
                    lit.push(next);
                    queue.push(next);
                }
                Cell::Frame => seen.push(next),
                other => return Err((next, other)),
            }
        }
    }
    if lit.len() < MIN_PORTAL_SIZE {
        return Err((start, Cell::Pool));
    }
    lit.sort_unstable();
    Ok(lit)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 2x2 water pool at y = 64 over the 4x4 square whose cells the closure
    /// reports: pool where `pool` holds the column, frame on the ring around it.
    fn world(pool: &[[i32; 2]], frame: &[[i32; 2]]) -> impl FnMut([i32; 3]) -> Cell + use<> {
        let pool: Vec<[i32; 2]> = pool.to_vec();
        let frame: Vec<[i32; 2]> = frame.to_vec();
        move |pos: [i32; 3]| {
            let column = [pos[0], pos[2]];
            if pos[1] == 64 && pool.contains(&column) {
                Cell::Pool
            } else if frame.contains(&column) {
                Cell::Frame
            } else {
                Cell::Other
            }
        }
    }

    /// The rule's answer as the tests read it: the lit blocks, or nothing.
    fn lit(start: [i32; 3], max: usize, at: impl FnMut([i32; 3]) -> Cell) -> Option<Vec<[i32; 3]>> {
        light_pool(start, max, at).ok()
    }

    /// The eight columns around a 2x2 pool at (0,0)..(1,1).
    fn ring() -> Vec<[i32; 2]> {
        vec![
            [-1, -1],
            [0, -1],
            [1, -1],
            [2, -1],
            [-1, 0],
            [2, 0],
            [-1, 1],
            [2, 1],
            [-1, 2],
            [0, 2],
            [1, 2],
            [2, 2],
        ]
    }

    #[test]
    fn a_two_by_two_pool_framed_by_dirt_lights() {
        let at = world(&[[0, 0], [1, 0], [0, 1], [1, 1]], &ring());
        let lit = lit([0, 64, 0], DEFAULT_MAX_PORTAL_SIZE, at);
        assert_eq!(
            lit,
            Some(vec![[0, 64, 0], [0, 64, 1], [1, 64, 0], [1, 64, 1]])
        );
    }

    #[test]
    fn three_pool_blocks_are_too_small() {
        let pool = [[0, 0], [1, 0], [2, 0]];
        let frame = vec![
            [-1, 0],
            [3, 0],
            [0, -1],
            [1, -1],
            [2, -1],
            [0, 1],
            [1, 1],
            [2, 1],
        ];
        assert_eq!(
            lit([0, 64, 0], DEFAULT_MAX_PORTAL_SIZE, world(&pool, &frame)),
            None
        );
    }

    #[test]
    fn an_l_shaped_pool_of_four_lights() {
        // Upstream does not require a square: four connected water blocks with
        // a frame around them light (the classic 2x2 plus a corner).
        let pool = [[0, 0], [1, 0], [0, 1], [0, 2]];
        let frame = vec![
            [-1, -1],
            [0, -1],
            [1, -1],
            [2, -1],
            [-1, 0],
            [2, 0],
            [-1, 1],
            [1, 1],
            [-1, 2],
            [1, 2],
            [-1, 3],
            [0, 3],
            [1, 3],
        ];
        let lit = lit([0, 64, 0], DEFAULT_MAX_PORTAL_SIZE, world(&pool, &frame));
        assert_eq!(lit.map(|v| v.len()), Some(4));
    }

    #[test]
    fn a_gap_in_the_frame_fails() {
        // The ring minus its top-middle block: that neighbour is air, and
        // upstream's `else return false` refuses the whole pool.
        let frame: Vec<[i32; 2]> = ring().into_iter().filter(|c| *c != [0, -1]).collect();
        let at = world(&[[0, 0], [1, 0], [0, 1], [1, 1]], &frame);
        assert_eq!(lit([0, 64, 0], DEFAULT_MAX_PORTAL_SIZE, at), None);
    }

    #[test]
    fn a_start_that_is_not_pool_never_lights() {
        let at = world(&[[0, 0], [1, 0], [0, 1], [1, 1]], &ring());
        assert_eq!(lit([-1, 64, -1], DEFAULT_MAX_PORTAL_SIZE, at), None);
    }

    #[test]
    fn a_refused_pool_names_the_block_that_stopped_it() {
        let frame: Vec<[i32; 2]> = ring().into_iter().filter(|c| *c != [0, -1]).collect();
        let at = world(&[[0, 0], [1, 0], [0, 1], [1, 1]], &frame);
        let refused = light_pool([0, 64, 0], DEFAULT_MAX_PORTAL_SIZE, at);
        assert_eq!(refused, Err(([0, 64, -1], Cell::Other)));
    }

    #[test]
    fn an_unloaded_neighbour_is_other_and_fails() {
        // A pool block with nothing known around it: every neighbour is Other.
        let at = |pos: [i32; 3]| {
            if pos == [0, 64, 0] {
                Cell::Pool
            } else {
                Cell::Other
            }
        };
        assert_eq!(lit([0, 64, 0], DEFAULT_MAX_PORTAL_SIZE, at), None);
    }

    #[test]
    fn four_pool_blocks_on_one_axis_do_not_light() {
        // 4x1 water with frame at both ends: the middle blocks' long sides see
        // Other, so it fails even though it has four pool blocks.
        let pool = [[0, 0], [1, 0], [2, 0], [3, 0]];
        let frame = vec![[-1, 0], [4, 0]];
        assert_eq!(
            lit([0, 64, 0], DEFAULT_MAX_PORTAL_SIZE, world(&pool, &frame)),
            None
        );
    }

    #[test]
    fn the_budget_is_inclusive_and_one_more_fails() {
        // A 8x8 pool = 64 blocks, exactly the cap.
        let pool: Vec<[i32; 2]> = (0..8).flat_map(|x| (0..8).map(move |z| [x, z])).collect();
        let mut frame: Vec<[i32; 2]> = Vec::new();
        for i in -1..=8 {
            frame.push([-1, i]);
            frame.push([8, i]);
            frame.push([i, -1]);
            frame.push([i, 8]);
        }
        assert_eq!(
            lit([0, 64, 0], DEFAULT_MAX_PORTAL_SIZE, world(&pool, &frame)).map(|v| v.len()),
            Some(64)
        );
        // One more pool block and the 64-cell budget is passed.
        let pool: Vec<[i32; 2]> = (0..9).flat_map(|x| (0..8).map(move |z| [x, z])).collect();
        let mut frame: Vec<[i32; 2]> = Vec::new();
        for i in -1..=9 {
            frame.push([-1, i]);
            frame.push([9, i]);
        }
        for i in -1..=8 {
            frame.push([i, -1]);
            frame.push([i, 8]);
        }
        assert_eq!(
            lit([0, 64, 0], DEFAULT_MAX_PORTAL_SIZE, world(&pool, &frame)),
            None
        );
    }

    #[test]
    fn frame_blocks_are_never_lit_and_each_pool_block_appears_once() {
        let at = world(&[[0, 0], [1, 0], [0, 1], [1, 1]], &ring());
        let lit = lit([0, 64, 0], DEFAULT_MAX_PORTAL_SIZE, at).expect("framed pool lights");
        let mut sorted = lit.clone();
        sorted.sort_unstable();
        assert_eq!(lit, sorted, "the answer is sorted");
        assert_eq!(lit.len(), 4);
        assert!(
            !lit.contains(&[2, 64, 2]),
            "a diagonal frame block is not lit"
        );
        assert!(
            lit.iter().all(|p| p[1] == 64),
            "only the pool's own layer lights"
        );
    }

    #[test]
    fn a_pool_under_the_start_is_not_a_horizontal_neighbour() {
        // Upstream walks `Direction.Plane.HORIZONTAL` only: water above or
        // below the start is never part of the pool.
        let at = |pos: [i32; 3]| match pos {
            [0, 64, 0] | [0, 65, 0] | [0, 63, 0] => Cell::Pool,
            [0, 0, -1] | [0, 0, 1] | [-1, 0, 0] | [1, 0, 0] => Cell::Frame,
            _ => Cell::Other,
        };
        assert_eq!(lit([0, 64, 0], DEFAULT_MAX_PORTAL_SIZE, at), None);
    }

    #[test]
    fn a_full_cube_is_a_sturdy_floor_and_thin_blocks_are_not() {
        assert!(is_sturdy_floor(&[[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]]));
        assert!(
            !is_sturdy_floor(&[[0.0, 0.0, 0.0, 1.0, 0.5, 1.0]]),
            "a slab"
        );
        assert!(!is_sturdy_floor(&[]), "air");
        assert!(
            !is_sturdy_floor(&[[0.0, 0.0, 0.0, 0.5, 1.0, 1.0]]),
            "a half floor"
        );
        assert!(
            is_sturdy_floor(&[
                [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
                [0.25, 0.0, 0.25, 0.75, 1.5, 0.75]
            ]),
            "a full cube plus a plant box"
        );
    }

    #[test]
    fn the_catalyst_is_the_diamond_the_jar_tag_resolves_to() {
        assert!(is_activator("minecraft:diamond"));
        assert!(!is_activator("minecraft:diamond_block"));
        assert!(!is_activator("minecraft:emerald"));
        assert!(!is_activator("minecraft:air"));
    }

    #[test]
    fn a_pool_block_alone_can_form_a_portal_and_a_return_portal_only_when_one_way() {
        assert!(
            can_form_portal(true, false, false),
            "water always qualifies"
        );
        assert!(
            can_form_portal(false, true, true),
            "a one-way portal block qualifies"
        );
        assert!(
            !can_form_portal(false, true, false),
            "a two-way portal does not"
        );
        assert!(
            !can_form_portal(false, false, true),
            "stone never qualifies"
        );
    }
}
