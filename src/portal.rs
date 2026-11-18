//! The portal's game-side glue: the 20-tick scan, the block reads, the fill and the catalyst.
//!
//! [`crate::runtime`] decides; this module asks the game what is where and writes back. It holds
//! no rule of its own — every number and predicate below is either upstream's or the rule's.
//!
//! Upstream: `events/TFTickHandler.playerTick` (`PlayerTickEvent.Post`, every 20 ticks, a 32-block
//! scan for an item in `#twilightforest:portal/activator` standing in a `canFormPortal` block owned
//! by that player) and `block/TFPortalBlock.tryToCreatePortal` (the rule in `runtime`).

use std::collections::BTreeMap;
use std::sync::Mutex;

use mod_api::{
    BlockKey, BlockShapeKind, BlockUpdateFlags, Compound, EntityRef, Host, ItemEntityRef, LevelRef,
    PlayerRef, ServerHost,
};

use crate::runtime::{
    Cell, DEFAULT_MAX_PORTAL_SIZE, ONE_WAY_PROPERTY, PORTAL_BLOCK, can_form_portal, is_activator,
    is_sturdy_floor, light_pool,
};

/// `#twilightforest:portal/edge`: what the frame is made of.
const EDGE_TAG: &str = "twilightforest:portal/edge";
/// `#twilightforest:portal/decoration`: what stands on the frame.
const DECORATION_TAG: &str = "twilightforest:portal/decoration";
/// The pool's own block (`#twilightforest:portal/fluid`).
const POOL_BLOCK: &str = "minecraft:water";
/// The bolt `causeLightning` adds.
const LIGHTNING: &str = "minecraft:lightning_bolt";

/// Upstream checks every 20 ticks per player with placement checking on
/// (`TFConfig.checkPortalPlacement`'s default; with it off, every 100 ticks).
const CHECK_EVERY: u32 = 20;
/// `player.getBoundingBox().inflate(range)`: 32 blocks, or 4 when portal
/// creation is restricted to admins and above (`portalCreationPermission`).
const RANGE: f64 = 32.0;
const ADMIN_RANGE: f64 = 4.0;
/// `Commands.LEVEL_ADMINS`.
const LEVEL_ADMINS: u8 = 2;
/// `TFConfig.portalCreationPermission`'s default: 0, every player may light one.
const PORTAL_CREATION_PERMISSION: u8 = 0;

/// A player's own server tick (`player_tick_post`): every 20 ticks, the upstream scan and, for
/// the first qualifying catalyst, the rule.
pub(crate) fn on_player_tick(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    ticks: &Mutex<BTreeMap<u128, u32>>,
) {
    if !tick_counter(ticks, player.uuid()).is_multiple_of(CHECK_EVERY) {
        return;
    }
    let permission = player.permission_level(host).unwrap_or(0);
    let Some(range) = scan_range(permission, PORTAL_CREATION_PERMISSION) else {
        return;
    };
    let Some(level) = player.level(host) else {
        return;
    };
    let Some(dimension) = player.dimension(host) else {
        return;
    };
    let Some([x, y, z]) = player.entity().position(host) else {
        return;
    };
    let Some(entities) = level.entities_in(
        host,
        [x - range, y - range, z - range],
        [x + range, y + range, z + range],
    ) else {
        return;
    };
    // `getEntitiesOfClass(ItemEntity.class, box)`: the box holds every entity kind, so narrow it
    // to dropped items; another kind is skipped, not a failure.
    let dropped: Vec<(EntityRef, ItemEntityRef)> = entities
        .into_iter()
        .filter_map(|entity| host.item_entity(entity).map(|item| (entity, item)))
        .collect();
    let dimension = dimension.to_string();
    for (entity, item) in dropped {
        let Some(stack) = item.stack(host) else {
            continue;
        };
        if !is_activator(&stack.item().id(host)) {
            continue;
        }
        let Some(position) = entity.position(host) else {
            continue;
        };
        let pos = block_pos(position);
        let block = level.block(host, pos);
        let pool = block.is_some_and(|block| Some(block) == host.block_key(POOL_BLOCK));
        let portal = block.is_some_and(|block| Some(block) == host.block_key(PORTAL_BLOCK));
        let one_way =
            portal && level.block_property(host, pos, ONE_WAY_PROPERTY).as_deref() == Some("true");
        if !can_form_portal(pool, portal, one_way) {
            continue;
        }
        // The rule runs against an immutable view: it only asks what is where.
        let light = {
            let view: &dyn Host = &*host;
            light_pool(pos, DEFAULT_MAX_PORTAL_SIZE, |at| cell(view, level, at))
        };
        let Ok(lit) = light else {
            continue;
        };
        let Some(state) = host.block_state_id(PORTAL_BLOCK) else {
            continue;
        };
        let mut lit_all = true;
        for at in &lit {
            lit_all &= level
                .set_block(
                    host,
                    *at,
                    state,
                    BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS,
                )
                .unwrap_or(false);
        }
        if !lit_all {
            continue;
        }
        // `causeLightning(pos, destructivePortalLightning)`: `destructive` defaults to true
        // upstream, and a bolt at the pool's own position is what that builds.
        let centre = [
            f64::from(pos[0]) + 0.5,
            f64::from(pos[1]),
            f64::from(pos[2]) + 0.5,
        ];
        let _ = host.summon(&dimension, LIGHTNING, centre, Compound::new());
        // `catalyst.getItem().shrink(1)`: the last item is the entity's removal, not an empty
        // stack (the write refuses one, and vanilla discards it on the next tick anyway).
        match stack.shrink(1) {
            Some(rest) => {
                let _ = item.set_stack(host, rest);
            }
            None => {
                let _ = entity.discard(host);
            }
        }
        return;
    }
}

/// How many server ticks this player has had, counting from one on their first.
fn tick_counter(ticks: &Mutex<BTreeMap<u128, u32>>, uuid: u128) -> u32 {
    let mut guard = ticks
        .lock()
        .expect("the tick counter lock is held for one wrapping add");
    let count = guard.entry(uuid).or_insert(0);
    *count = count.wrapping_add(1);
    *count
}

/// The scan radius for a player, or `None` when they may not light a portal: upstream's
/// `player.hasPermissions(PORTAL_CREATION_PERMISSION)`, then
/// `portalCreationPermission >= LEVEL_ADMINS ? 4 : 32`.
///
/// Both sides are *configured* values: the radius comes from the configured permission, not the
/// player's. They are parameters so the configured defaults stay visible — at `0` every player
/// passes, and a config page can make the gate real without touching this function.
fn scan_range(permission: u8, configured: u8) -> Option<f64> {
    (permission >= configured).then_some(if configured >= LEVEL_ADMINS {
        ADMIN_RANGE
    } else {
        RANGE
    })
}

fn block_pos(position: [f64; 3]) -> [i32; 3] {
    [
        position[0].floor() as i32,
        position[1].floor() as i32,
        position[2].floor() as i32,
    ]
}

fn below(pos: [i32; 3]) -> [i32; 3] {
    [pos[0], pos[1] - 1, pos[2]]
}

fn above(pos: [i32; 3]) -> [i32; 3] {
    [pos[0], pos[1] + 1, pos[2]]
}

/// A block against the server's merged block tags: an unknown tag answers false, as the API
/// documents; an unavailable tag is treated as "not a member", so a pool is never lit on a guess.
fn in_block_tag(host: &dyn Host, block: BlockKey, tag: &str) -> bool {
    host.block_in_tag(block, tag).unwrap_or(false)
}

/// Whether the block's collision shape has a full top face, which is what upstream asks of the
/// block under a pool (`isFaceSturdy(level, pos, UP)`). The boxes go to
/// [`crate::runtime::is_sturdy_floor`], so the measured rule stays the one the 0.1.0 e2e proved.
fn sturdy_floor(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> bool {
    let boxes = level
        .block_state(host, pos)
        .and_then(|state| level.block_shape(host, state, pos, BlockShapeKind::Collision))
        .unwrap_or_default();
    is_sturdy_floor(&boxes)
}

/// What the rule sees at `pos`. Upstream compares against the start block's own state; this port
/// treats the pool block (water) and a one-way portal block as pool, which accepts a mixed pool
/// upstream would refuse. Recorded in notes.md.
fn cell(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> Cell {
    let Some(block) = level.block(host, pos) else {
        return Cell::Other;
    };
    let portal = Some(block) == host.block_key(PORTAL_BLOCK);
    let one_way =
        portal && level.block_property(host, pos, ONE_WAY_PROPERTY).as_deref() == Some("true");
    if Some(block) == host.block_key(POOL_BLOCK) || can_form_portal(false, portal, one_way) {
        return if sturdy_floor(host, level, below(pos)) {
            Cell::Pool
        } else {
            Cell::Other
        };
    }
    let decorated = level
        .block(host, above(pos))
        .is_some_and(|up| in_block_tag(host, up, DECORATION_TAG));
    if in_block_tag(host, block, EDGE_TAG) && decorated {
        return Cell::Frame;
    }
    Cell::Other
}
