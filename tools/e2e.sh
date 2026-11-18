#!/usr/bin/env bash
# In-game check for The Twilight Forest: the mod's own portal.
#
# What this proves: a framed pool of water standing on dirt, ringed by dirt with
# a flower on every ring block, is lit into `twilightforest:twilight_portal` when
# a diamond is dropped into it — by the port's `player_tick_post` scan and the
# pool rule in `src/runtime.rs` — while an identically built pool that receives a
# stick stays water.
#
# Since the SDK migration (2026-11-18) the mod is one flat `.hmod` that the
# launcher compiles into the game, carrying its own assets/ and data/. So:
#   - `GAME` must name a game the launcher built **with this package installed**:
#       horizon-launcher native-build ports/twilightforest --instance <i> --trust-native-code
#     then `GAME=<that horizon-client> tools/e2e.sh`. No `--dev-mod` is passed:
#     the block, its model and the frame tags are inside the same package, and
#     the game registers them when it loads it.
#   - `NO_MOD_GAME` must name a game of the **same Horizon revision without the
#     mod**, for the control leg. Nothing publishes one at the SDK pin yet
#     (`ports/_shared/bin/` holds `1d63e0a5` and `41f822b6` only), so the run
#     must pass one — a build of the pinned revision with no mod — rather than
#     silently compare against another revision.
#
# Legs:
#   1. DEFAULT: pool A gets a diamond (must light), pool B a stick (must stay
#      water) — the counterfactual that keeps the check from passing vacuously.
#   2. the dimension: `execute in twilightforest:twilight_forest` answers and a
#      block written there reads back, and `/twilightforest dimension 0 64 0
#      20261105` prints the registered level, its 14 carried biomes and the
#      grid's answer for that cell — the same line `src/grid.rs` and
#      `src/dimension.rs` unit-test. **NOT YET RUN**: no launcher exists at this
#      pin, so a package-built game cannot be made here (notes.md).
#   3. the no-mod control: the same pin without the package refuses
#      `setblock twilightforest:twilight_portal` and `execute in
#      twilightforest:twilight_forest` as unknown, and a diamond in a pool lights
#      nothing. Proves the block, the rule and the level came from the package.
#   4. SELFTEST=1: the DEFAULT scenario asserting that pool B (the stick pool)
#      lit — it must fail, so the assertions can fail.
#
#   tools/e2e.sh                    # legs 1 and 2
#   SELFTEST=1 tools/e2e.sh         # leg 3, must exit non-zero
set -uo pipefail
export LC_ALL=C

CLONE=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)
PORT=$CLONE/ports/twilightforest
CTL=${CTL:-$PORT/target/horizon-cargo/fast/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
NO_MOD_GAME=${NO_MOD_GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SELFTEST=${SELFTEST:-0}
PORTAL=twilightforest:twilight_portal
DIMENSION=twilightforest:twilight_forest
# A 4x4 pool: upstream's minimum is four blocks and this is 16, wide enough
# that the throw's drift still lands in the water.
POOL=4
SOCK_BASE=/tmp/opencode/twilightforest-e2e-$$.sock

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL (a launcher-built game also needs it: horizon-launcher native-build)" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; echo "it must be a game built WITH the package: horizon-launcher native-build ports/twilightforest --instance <i> --trust-native-code, then GAME=<that horizon-client> $0" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/twilightforest-e2e.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
export HORIZON_LAUNCHER_NO_SELF_UPDATE=1
mkdir -p "$XDG_CONFIG_HOME" "$(dirname "$SOCK_BASE")"

GAME_DIR=
fail() {
    echo "FAIL: $*" >&2
    # Quit first: the log file is flushed on shutdown, so a live read is short.
    if [ -n "$SOCK" ] && [ -S "$SOCK" ]; then ctl quit >/dev/null 2>&1 || true; sleep 1; fi
    echo "--- game log (tail) ---"
    if [ -n "$HORIZON_HOME" ] && [ -f "$HORIZON_HOME/logs/latest.log" ]; then
        grep -E "Twilight Forest|filled|setblock|Gave|Unknown" "$HORIZON_HOME/logs/latest.log" | tail -25
    fi
    exit 1
}
trap 'echo "EXIT:$?"' EXIT

SOCK=
ctl() { "$CTL" --socket "$SOCK" "$@"; }
launch() {
    local world=$1 game_dir=$2
    SOCK=$SOCK_BASE.$world
    rm -f "$SOCK"
    mkdir -p "$game_dir"
    GAME_DIR=$game_dir
    ctl launch --game "$GAME" --world "$world" --seed 20261111 --creative --headless \
        --wait --resources "$RES" --game-dir "$game_dir" >/dev/null || fail "launch $world"
    ctl wait-ready 120 >/dev/null || fail "the world never became ready"
    [ "$(ctl state | jq -r '.screen')" = "playing" ] || fail "not in the world"
}
stop() { ctl quit >/dev/null 2>&1 || true; rm -f "$SOCK"; }

block_at() { ctl block "$1" "$2" "$3" | jq -r '.block // "unknown"'; }

# A framed pool with its lower corner at (X, Y, Z): dirt under the water and for
# one block around it, water in the middle, a poppy on every ring block
# (upstream's PORTAL_EDGE + PORTAL_DECO tags).
# The frame is an input to the rule, so it is proved after every step that
# could touch it; a failure names the step and the block.
check_ring() {
    # Each on its own line: bash expands every word of a `local` before it
    # assigns any, so `hi=$((x + POOL - 1))` on the same line would read the
    # caller's x (this cost a run).
    local x=$1
    local y=$2
    local z=$3
    local stage=$4
    local hi=$((x + POOL - 1))
    local hz=$((z + POOL - 1))
    local cx cz name
    for cx in $(seq $((x-1)) $((hi+1))); do
        for cz in $(seq $((z-1)) $((hz+1))); do
            if [ "$cx" -ge "$x" ] && [ "$cx" -le "$hi" ] && [ "$cz" -ge "$z" ] && [ "$cz" -le "$hz" ]; then
                continue
            fi
            name=$(block_at "$cx" "$y" "$cz")
            [ "$name" = "minecraft:dirt" ] || fail "after $stage: ring $cx $y $cz is $name, not dirt"
        done
    done
}

build_pool() {
    local x=$1
    local y=$2
    local z=$3
    local hi=$((x + POOL - 1))
    local hz=$((z + POOL - 1))
    # One command at a time with a tick between them: the server applies a
    # console command on its next tick, so a burst can overtake itself.
    ctl command "fill $((x-1)) $((y-1)) $((z-1)) $((hi+1)) $((y-1)) $((hz+1)) minecraft:dirt" >/dev/null
    ctl wait 2 >/dev/null
    local reply
    reply=$(ctl command "fill $((x-1)) $y $((z-1)) $((hi+1)) $y $((hz+1)) minecraft:dirt" 2>&1)
    echo "ring fill $((x-1)) $y $((z-1))..$((hi+1)) $y $((hz+1)): $reply"
    ctl wait 2 >/dev/null
    check_ring "$x" "$y" "$z" "the ring fill"

    # Water one block at a time: a fill that floods the ring is not visible in
    # its own reply, and a source block placed in an enclosed ring cannot flow.
    local wx wz
    for wx in $(seq "$x" "$hi"); do
        for wz in $(seq "$z" "$hz"); do
            reply=$(ctl command "setblock $wx $y $wz minecraft:water" 2>&1)
            case "$reply" in
                *Successfully*|*Changed*) ;;
                *) echo "water setblock $wx $y $wz: $reply" ;;
            esac
            ctl wait 1 >/dev/null
        done
    done
    check_ring "$x" "$y" "$z" "the water"

    ctl command "fill $((x-1)) $((y+1)) $((z-1)) $((hi+1)) $((y+1)) $((hz+1)) minecraft:air" >/dev/null
    check_ring "$x" "$y" "$z" "the air above"
    # One flower per ring block, as four rows: fewer commands than 28 setblocks,
    # and no poppy is ever placed over water and broken into an item.
    ctl command "fill $((x-1)) $((y+1)) $((z-1)) $((hi+1)) $((y+1)) $((z-1)) minecraft:poppy" >/dev/null
    ctl wait 2 >/dev/null
    ctl command "fill $((x-1)) $((y+1)) $((hz+1)) $((hi+1)) $((y+1)) $((hz+1)) minecraft:poppy" >/dev/null
    ctl wait 2 >/dev/null
    ctl command "fill $((x-1)) $((y+1)) $z $((x-1)) $((y+1)) $hz minecraft:poppy" >/dev/null
    ctl wait 2 >/dev/null
    ctl command "fill $((hi+1)) $((y+1)) $z $((hi+1)) $((y+1)) $hz minecraft:poppy" >/dev/null
    ctl wait 2 >/dev/null
    check_ring "$x" "$y" "$z" "the flowers"

    local dx2 dz2 name
    for dx2 in $(seq "$x" "$hi"); do
        for dz2 in $(seq "$z" "$hz"); do
            name=$(block_at "$dx2" "$y" "$dz2")
            [ "$name" = "minecraft:water" ] || fail "pool cell $dx2 $y $dz2 is $name, not water"
        done
    done
    local rx rz
    for rx in $(seq $((x-1)) $((hi+1))); do
        for rz in $(seq $((z-1)) $((hz+1))); do
            if [ "$rx" -ge "$x" ] && [ "$rx" -le "$hi" ] && [ "$rz" -ge "$z" ] && [ "$rz" -le "$hz" ]; then
                continue
            fi
            [ "$(block_at "$rx" "$((y+1))" "$rz")" = "minecraft:poppy" ] \
                || fail "ring block $rx $y $rz has no poppy above"
        done
    done
    echo "built a framed $POOL x $POOL pool at $x $y $z (ring and flowers complete)"
}

# Every pool block's name, one per line.
pool_names() {
    local x=$1 y=$2 z=$3 dx dz
    for dx in $(seq "$x" $((x + POOL - 1))); do
        for dz in $(seq "$z" $((z + POOL - 1))); do
            block_at "$dx" "$y" "$dz"
        done
    done
}

count_named() { grep -cx "$1"; }

# Hold `item` from the hotbar (`ctl select` by item only reaches the hotbar).
hold() {
    local item=$1 index held
    index=$(ctl inventory | jq -r --arg i "$item" '[.hotbar[]? | .item] | index($i) // empty')
    [ -n "$index" ] || { ctl inventory | jq -c '{held, hotbar: [.hotbar[]? | .item]}' >&2; fail "no $item in the hotbar"; }
    ctl select $((index + 1)) >/dev/null
    held=$(ctl inventory | jq -r '.held.item // "none"')
    [ "$held" = "$item" ] || fail "holding $held, expected $item"
}

# Drop the catalyst into the pool, then step away: a player standing on the drop
# picks it back up after its pickup delay (upstream expects the diamond to be
# thrown in and left there).
drop_item() {
    local item=$1 pool_x=$2 pool_y=$3 pool_z=$4 away_x=$5 away_y=$6 away_z=$7
    # An empty inventory puts the catalyst in hotbar slot 1 whatever was picked
    # up before; the scratch world has nothing else worth keeping.
    ctl command "clear" >/dev/null
    ctl give "$item" 1 >/dev/null
    ctl wait 2 >/dev/null
    hold "$item"
    # In the middle of the pool looking straight down, so the drop falls into
    # the water instead of being thrown out of it (a drop carries the view's
    # direction); then away, so the item is not picked back up.
    ctl tp $((pool_x + 1)) $((pool_y + 1)) $((pool_z + 1)) >/dev/null
    ctl look 0 90 >/dev/null
    ctl wait 15 >/dev/null
    local pitch
    pitch=$(ctl state | jq -r '.player.pitch')
    echo "drop pitch: $pitch"
    awk -v p="$pitch" 'BEGIN { exit !(p > 80) }' || fail "the view did not point down (pitch $pitch)"
    ctl drop >/dev/null
    ctl tp "$away_x" "$away_y" "$away_z" >/dev/null
    ctl wait 60 >/dev/null
}

# --- leg 1: the portal, and the counterfactual -------------------------------
run_main() {
    launch "TwilightForestE2E-$$" "$SCRATCH/main"
    local state px py pz x y z
    state=$(ctl state)
    px=$(jq -r '.player.position[0] | floor' <<<"$state")
    py=$(jq -r '.player.position[1] | floor' <<<"$state")
    pz=$(jq -r '.player.position[2] | floor' <<<"$state")
    x=$((px + 8)); y=$((py + 40)); z=$((pz + 8))

    # Both pools first: then nothing one pool's lighting does can be blamed for
    # the other's water.
    local bx=$((x + 10)) bz=$z
    echo "## pool A: a diamond"
    build_pool "$x" "$y" "$z"
    echo "## pool B (identical frame): a stick"
    build_pool "$bx" "$y" "$bz"

    echo "## a diamond in pool A"
    drop_item minecraft:diamond $((x + 1)) $((y + 1)) $((z + 1)) $((x - 6)) $((y + 2)) $((z - 6))
    local a lit
    a=$(pool_names "$x" "$y" "$z")
    lit=$(count_named "$PORTAL" <<<"$a")
    echo "pool A block counts: $(sort <<<"$a" | uniq -c | tr '\n' ' ')"
    echo "pool A lit $lit/$((POOL * POOL))"
    [ "$lit" = $((POOL * POOL)) ] || fail "pool A lit $lit/$((POOL * POOL)) with a diamond"

    echo
    echo "## a stick in pool B"
    echo "pool B before the stick: $(sort <<<"$(pool_names "$bx" "$y" "$bz")" | uniq -c | tr '\n' ' ')"
    drop_item minecraft:stick $((bx + 1)) $((y + 1)) $((bz + 1)) $((bx + 6)) $((y + 2)) $((bz + 6))
    local b stayed
    b=$(pool_names "$bx" "$y" "$bz")
    stayed=$(count_named "minecraft:water" <<<"$b")
    echo "pool B block counts: $(sort <<<"$b" | uniq -c | tr '\n' ' ')"
    echo "pool B kept $stayed/$((POOL * POOL)) water"
    if [ "$SELFTEST" = 1 ]; then
        # A deliberately wrong expectation: the stick pool must NOT light, so
        # this assertion failing is the selftest's pass (exit non-zero).
        [ "$stayed" = 0 ] || fail "SELFTEST: the stick pool was expected to light and did not"
        exit 0
    fi
    [ "$stayed" = $((POOL * POOL)) ] || fail "pool B changed with a stick ($stayed water)"
    echo "PASS: a diamond lights the framed pool into $PORTAL; a stick does not"
    run_dimension
    stop
}

# --- leg 2: the dimension, and the readout -----------------------------------
# The level is the package's registration: `execute in` names it, a block written
# under it lands in that level, and the readout prints the dimension id, the
# carried biome count and a grid answer for a cell. The readout is asserted
# against the numbers `src/grid.rs`'s tests pin, so a regression in the grid
# fails here as well as in `cargo test`.
run_dimension() {
    local reply line
    reply=$(ctl command "execute in $DIMENSION run setblock 8 -20 8 minecraft:glass" 2>&1)
    echo "execute in $DIMENSION run setblock: $reply"
    grep -qi "unknown dimension" <<<"$reply" && fail "$DIMENSION is unknown to the package's own game"
    ctl wait 2 >/dev/null
    reply=$(ctl command "execute in $DIMENSION run setblock 9 -21 9 minecraft:glass" 2>&1)
    echo "second write in $DIMENSION: $reply"
    ctl wait 2 >/dev/null

    # `ctl chat` runs the player-chat path, which is what the mod's command table
    # hears (`ctl command` is the console path).
    line=$(ctl chat "/twilightforest dimension 0 64 0 20261105" 2>&1)
    echo "readout: $line"
    grep -q "twilightforest:twilight_forest" <<<"$line" || fail "the readout did not name the level: $line"
    grep -q "14 biomes" <<<"$line" || fail "the readout did not name the carried biome count: $line"
    grep -q "twilightforest:forest" <<<"$line" || fail "the readout did not answer the grid for 0 64 0: $line"
    echo "PASS: $DIMENSION exists, takes blocks and answers /twilightforest dimension"
}

# --- leg 2: the block and the rule come from the package ---------------------
# The same revision with no mod installed: the block is unknown and a framed pool
# stays water, so leg 1's lighting is the package's and not the engine's.
run_no_mod_control() {
    if [ -z "$NO_MOD_GAME" ] || [ ! -x "$NO_MOD_GAME" ]; then
        fail "the control leg needs NO_MOD_GAME=<a game of this revision without the mod> (got '${NO_MOD_GAME:-unset}'); no unmodded binary is published at the SDK pin yet"
    fi
    GAME=$NO_MOD_GAME
    launch "TwilightForestNoModControl-$$" "$SCRATCH/no-mod"
    local state px py pz x y z
    state=$(ctl state)
    px=$(jq -r '.player.position[0] | floor' <<<"$state")
    py=$(jq -r '.player.position[1] | floor' <<<"$state")
    pz=$(jq -r '.player.position[2] | floor' <<<"$state")
    x=$((px + 8)); y=$((py + 40)); z=$((pz + 8))

    local reply error
    reply=$(ctl command "setblock $x $y $z $PORTAL" 2>&1)
    error=$(jq -r '(.chat // []) | join(" | ")' <<<"$reply" 2>/dev/null)
    echo "setblock $PORTAL without the mod: $reply"
    grep -qi "unknown" <<<"$error" || fail "$PORTAL is known without the package: $reply"

    reply=$(ctl command "execute in $DIMENSION run setblock $x $y $z minecraft:glass" 2>&1)
    error=$(jq -r '(.chat // []) | join(" | ")' <<<"$reply" 2>/dev/null)
    echo "execute in $DIMENSION without the mod: $reply"
    grep -qi "unknown dimension" <<<"$error" || fail "$DIMENSION exists without the package: $reply"

    build_pool "$x" "$y" "$z"
    drop_item minecraft:diamond $((x + 1)) $((y + 1)) $((z + 1)) $((x - 6)) $((y + 2)) $((z - 6))
    local names stayed
    names=$(pool_names "$x" "$y" "$z")
    stayed=$(count_named "minecraft:water" <<<"$names")
    echo "pool without the mod: $(sort <<<"$names" | uniq -c | tr '\n' ' ')"
    [ "$stayed" = $((POOL * POOL)) ] || fail "a portal formed without the package ($stayed water)"
    echo "PASS: without the package the block is unknown and no portal forms"
    stop
}

case "$SELFTEST" in
    1) run_main ;;
    *) run_main; run_no_mod_control ;;
esac
echo "All legs passed"
