//! The Twilight Forest for Horizon — the portal and the dimension, as one `impl Hooks`.
//!
//! The portal rule is [`runtime`]'s (upstream `TFPortalBlock.recursivelyValidatePortal`): a water
//! pool of at least four blocks, each standing on a sturdy floor, ringed by
//! `#twilightforest:portal/edge` blocks with `#twilightforest:portal/decoration` above them, is lit
//! into `twilightforest:twilight_portal` when a diamond is dropped in. [`portal`] is the glue that
//! asks the game and writes back; this file registers the moment.
//!
//! The dimension is [`dimension`]'s: `twilightforest:twilight_forest` with the jar's dimension
//! type, and a `BiomeSource` that answers `twilightforest:twilight_biomes` from the jar's own
//! biome grid ([`grid`], pure and unit-tested against a Java oracle). The terrain is a recorded
//! gap; `notes.md` says what is missing and why.
//!
//! LOCAL-ONLY until the maintainer approves publication (ARCHITECTURE.md §8): LGPL-2.1, rank 1,
//! publishable under the same licence with this source.

pub mod dimension;
pub mod grid;
pub mod portal;
pub mod runtime;

use std::collections::BTreeMap;
use std::sync::Mutex;

use mod_api::{
    self, Flow, Hooks, HostSide, PlayerTickPostEvent, RegisterCommandsEvent, Registrar,
    RegistrationContext, RunModCommandEvent, ServerHost,
};

/// The mod's session state: how many server ticks each player has had, so the 20-tick cadence of
/// upstream's scan counts per player and survives across calls.
///
/// `Hooks` is `Any + Send + Sync` and the macro boxes one `Default` instance for the session, so
/// the state is a `Mutex` field, not a `RefCell` or a `static`.
#[derive(Default)]
struct Mod {
    ticks: Mutex<BTreeMap<u128, u32>>,
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Once, at startup: the dimension's biome source, its chunk generator and the level itself.
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        dimension::register(r);
    }

    /// `/twilightforest dimension [<x> <y> <z> <seed>]`: the readout the harness asserts on.
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        dimension::register_commands(event);
    }

    /// Answer the readout, and let every other usage through.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        dimension::run_command(host, event)
    }

    /// After a player's tick on both logical sides (NeoForge `PlayerTickEvent.Post`): every 20
    /// ticks, scan 32 blocks around that player for a dropped catalyst standing in a framed pool.
    ///
    /// The client half does nothing: the block write and the lightning are the server's.
    fn player_tick_post(&self, host: HostSide, event: &mut PlayerTickPostEvent) {
        let HostSide::Server(host) = host else {
            return;
        };
        portal::on_player_tick(host, event.player, &self.ticks);
    }
}
