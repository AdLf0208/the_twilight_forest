#!/usr/bin/env bash
# Measured probe: can a fourth dimension be entered at pin 1d63e0a?
#
# This is the port's *blocking* evidence, not a mod scenario. It runs the
# shared, unmodded Horizon game (ports/_shared/bin/horizon-1d63e0a5) so nothing
# but the engine answers:
#
#   1. `execute in twilightforest:twilight_forest run tp @s …` is refused
#      (`argument.dimension.invalid`) and the player stays in the Overworld;
#   2. the same line with `minecraft:the_nether` succeeds, `ctl state` reports
#      the Nether, and a block placed there reads back (the positive control —
#      without it the first assertion could pass on a broken command path);
#   3. `setblock` addressed at the Twilight Forest id is refused too.
#
# It can fail: a pin that learned custom dimensions would turn step 1 into a
# success and the script would print FAIL.
set -uo pipefail
export LC_ALL=C

CLONE=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)
SOCK=${SOCK:-/tmp/opencode/twilightforest-dimension.sock}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
WORLD=${WORLD:-TwilightForestDimensionProbe-$$}
TF_DIM=twilightforest:twilight_forest

# shellcheck disable=SC1091
eval "$("$CLONE/ports/tools/shared-bin.sh" env)"
CTL=${CTL:?shared horizon-ctl not found}
GAME=${GAME:?shared horizon-client not found}

SCRATCH=$(mktemp -d /tmp/opencode/twilightforest-dim.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
export HORIZON_LAUNCHER_NO_SELF_UPDATE=1
mkdir -p "$XDG_CONFIG_HOME" "$(dirname "$SOCK")"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; cleanup 1; }
cleanup() {
    local code=${1:-0}
    ctl quit >/dev/null 2>&1 || true
    rm -f "$SOCK"
    echo "EXIT:$code"
    exit "$code"
}
trap 'cleanup $?' EXIT

# A scratch game folder: saves, options and caches stay out of the repo (AGENTS.md).
GAME_DIR=$SCRATCH/game-dir
mkdir -p "$GAME_DIR"
ctl launch --game "$GAME" --world "$WORLD" --seed 20261111 --creative --headless \
    --wait --resources "$RES" --game-dir "$GAME_DIR" >/dev/null || fail "launch"
ctl wait-ready 120 >/dev/null || fail "the world never became ready"

echo "## 1. a Twilight Forest dimension: $TF_DIM"
state_before=$(ctl state)
dim_before=$(jq -r '.dimension' <<<"$state_before")
pos_before=$(jq -c '.player.position' <<<"$state_before")
echo "state before: $state_before"
reply=$(ctl command "execute in $TF_DIM run tp @s 0 100 0" 2>&1)
echo "command reply: $reply"
# The ctl transport answers ok:true even when the command itself is refused;
# the refusal is the chat line, so assert on the parsed text (AGENTS.md quirk).
error=$(jq -r '(.chat // []) | join(" | ")' <<<"$reply" 2>/dev/null)
echo "command error line: $error"
state_after=$(ctl state)
dim_after=$(jq -r '.dimension' <<<"$state_after")
pos_after=$(jq -c '.player.position' <<<"$state_after")
echo "state after: $state_after"
[ "$dim_after" = "$dim_before" ] || fail "the player left $dim_before"
# The player may fall (spawn), so compare against the requested target, not the
# previous tick's position.
at_target=$(jq -r '(.player.position[0] | fabs) < 1 and ((.player.position[1] - 100) | fabs) < 1 and (.player.position[2] | fabs) < 1' <<<"$state_after")
[ "$at_target" = "false" ] || fail "the player reached 0 100 0, the tp worked"
grep -qi "unknown dimension" <<<"$error" || fail "the command was not refused: $reply"
echo "PASS: $TF_DIM is refused and the player stays in $dim_before"

echo
echo "## 2. the same line for minecraft:the_nether (positive control)"
reply=$(ctl command "execute in minecraft:the_nether run tp @s 0 100 0" 2>&1)
echo "command reply: $reply"
ctl wait 20 >/dev/null
state=$(ctl state)
dim=$(jq -r '.dimension' <<<"$state")
echo "state: $state"
[ "$dim" = "minecraft:the_nether" ] || fail "expected the nether, state says $dim"
# A block a few blocks away from the falling player, so the placement is not
# inside an entity; read it back after a tick.
reply=$(ctl setblock 3 100 3 minecraft:glass 2>&1)
echo "setblock reply: $reply"
ctl wait 5 >/dev/null
block=$(ctl block 3 100 3 | jq -r '.block')
echo "block at 3 100 3 in the nether: $block"
[ "$block" = "minecraft:glass" ] || fail "expected glass, read $block"
echo "PASS: the nether control moved the player and placed a block that read back"

echo
echo "## 3. a block written into the Twilight Forest dimension"
reply=$(ctl command "execute in $TF_DIM run setblock 0 100 0 minecraft:glass" 2>&1)
echo "command reply: $reply"
error=$(jq -r '(.chat // []) | join(" | ")' <<<"$reply" 2>/dev/null)
echo "command error line: $error"
grep -qi "unknown dimension" <<<"$error" || fail "setblock into $TF_DIM was accepted: $reply"
echo "PASS: $TF_DIM takes no blocks either"

echo
echo "All three legs agree: at this pin a dimension is one of three fixed levels."
cleanup 0
