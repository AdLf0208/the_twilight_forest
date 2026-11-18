#!/usr/bin/env python3
"""Draw the portal barrier's swirl — this port's own art — and its animation.

The barrier is the two-plane model the one-way portal uses (``models/block/twilight_portal_barrier.json``).
Upstream's ``portal_barrier.png`` is 14,392 B of its own animation, and the upstream repository's
``ASSET_LICENSE`` is CC-BY-NC-SA-4.0, so it cannot travel in a package published under the code's
LGPL-2.1 (ARCHITECTURE.md §8, the MineColonies precedent: code may be ported, art must be
regenerated). This generator **draws a new one** from the palette below and a closed-form spiral;
it reads no upstream file, holds no upstream pixel, and uses no randomness.

Design (every number below is the recipe, so the art is auditable):

* a **16x16 frame repeated 32 times** — a 16x512 RGBA PNG, the layout an animated block texture
  uses; the frame count is the loop length;
* one **three-armed spiral** in the purple register the block's identity calls for: for a pixel at
  ``(x, y)`` the polar angle and radius around the frame's centre feed
  ``phase = 3*theta + 5*r - 2*pi*frame/32``, and the ribbon is the band where ``phase`` falls in the
  first 0.26 of a turn — so one arm turns once over the 32 frames and the loop is seamless;
* the ribbon's leading edge is the brightest part (``HIGHLIGHT``), its trailing edge fades through
  ``BRIGHT`` and ``MID`` into the membrane's ``DEEP``; a radial falloff darkens the rim and a soft
  glow lifts the centre, so the disc reads as a lit membrane rather than a flat colour;
* **alpha is opaque** (240-255, a slight depth cue). This pin's models declare no render type
  (``docs/resource-packs.md``: only frame count, frame time and frame size of an animation are
  read), so the texture must look right whether the game draws it solid or cut out; a fully
  transparent background would render as a black frame on a solid layer.

``portal_barrier.png.mcmeta`` is written here too: 32 frames, ``frametime: 2`` (the 64-tick loop
the upstream animation kept), and an explicit frame list, so the animation is the generator's and
not an echo of the jar's four-line default. ``interpolate`` is **not** asked for: this pin steps
frames instead of blending them (``docs/resource-packs.md``, "Texture animation: interpolate: no"),
so the port does not claim a behaviour it does not get; the 32 frames make the step small.

    python3 tools/gen_portal_texture.py            # write both files
    python3 tools/gen_portal_texture.py --check    # exit 1 if either differs from what it would write

The output is deterministic: fixed palette, fixed frames, integer arithmetic, and a fixed zlib
level, so the same commit always produces the same bytes (``--check`` proves it).
"""

from __future__ import annotations

import argparse
import json
import math
import struct
import zlib
from pathlib import Path

PORT = Path(__file__).resolve().parent.parent

TEXTURE = "assets/twilightforest/textures/block/portal_barrier.png"
ANIMATION = f"{TEXTURE}.mcmeta"

FRAME = 16
FRAMES = 32
FRAMETIME = 2

# The palette: one named colour per role, in the purple register the block's identity calls for.
# Nothing here is sampled from upstream; every channel is chosen for this generator.
DEEP = (28, 12, 56)  # the membrane between the arms
MID = (70, 32, 116)  # the arm's trailing edge
BRIGHT = (134, 76, 198)  # the arm's body
HIGHLIGHT = (206, 160, 250)  # the arm's leading edge
ALPHA = 255
ALPHA_RIM = 240  # the rim is a touch more transparent: depth, and visible if a layer blends

# The spiral's shape (see the module docstring): arms, twist, ribbon width, radial falloff.
ARMS = 3.0
TWIST = 5.0
RIBBON = 0.26  # in turns
RIM_START = 0.70  # radius where the membrane starts to darken, 1.0 = half the frame
RIM_END = 1.22  # radius where it is at its darkest
GLOW_RADIUS = 0.50  # the centre's soft glow fades out at this radius
GLOW = 0.35  # how far the glow lifts the membrane towards BRIGHT


def mix(low: tuple[int, int, int], high: tuple[int, int, int], amount: float) -> tuple[int, int, int]:
    """Linear blend of two palette colours, ``amount`` 0 = low, 1 = high, rounded to a byte."""
    return (
        round(low[0] + (high[0] - low[0]) * amount),
        round(low[1] + (high[1] - low[1]) * amount),
        round(low[2] + (high[2] - low[2]) * amount),
    )


def colour(radius: float, phase: float, rim: float) -> tuple[int, int, int]:
    """One pixel of the membrane: the arm's brightness from its position in the ribbon."""
    band = phase % 1.0
    if band < RIBBON:
        edge = 1.0 - band / RIBBON  # 1 at the leading edge, 0 at the trailing one
        body = mix(MID, BRIGHT, min(1.0, edge * 1.4))
        lit = mix(body, HIGHLIGHT, max(0.0, edge - 0.62) / 0.38)
    else:
        # Between the arms: the membrane, darkest against the rim and just behind an arm.
        lit = mix(DEEP, MID, 0.18 * max(0.0, 1.0 - (band - RIBBON) / (1.0 - RIBBON)))
    glow = max(0.0, 1.0 - radius / GLOW_RADIUS) * GLOW
    shade = 1.0 - 0.55 * rim
    return mix(DEEP, mix(lit, BRIGHT, glow), shade)


def frame_pixels(index: int) -> list[list[tuple[int, int, int, int]]]:
    """One frame, row by row: a spiral turned by ``index``/``FRAMES`` of a full revolution."""
    turn = 2.0 * math.pi * index / FRAMES
    centre = (FRAME - 1) / 2.0
    rows: list[list[tuple[int, int, int, int]]] = []
    for y in range(FRAME):
        row: list[tuple[int, int, int, int]] = []
        for x in range(FRAME):
            dx, dy = x - centre, y - centre
            radius = math.hypot(dx, dy) / centre
            angle = math.atan2(dy, dx)
            phase = (ARMS * angle + TWIST * radius - turn) / (2.0 * math.pi)
            # 0 inside RIM_START, 1 from RIM_END out: the membrane's radial falloff.
            rim = min(1.0, max(0.0, (radius - RIM_START) / (RIM_END - RIM_START)))
            r, g, b = colour(radius, phase, rim)
            alpha = round(ALPHA + (ALPHA_RIM - ALPHA) * rim)
            row.append((r, g, b, alpha))
        rows.append(row)
    return rows


def png(frames: list[list[list[tuple[int, int, int, int]]]]) -> bytes:
    """A 16x(16*FRAMES) RGBA PNG, encoded by hand: no image library, no timestamp."""
    height = FRAME * len(frames)
    raw = bytearray()
    for rows in frames:
        for row in rows:
            raw.append(0)  # filter: none
            for pixel in row:
                raw.extend(pixel)

    def chunk(kind: bytes, body: bytes) -> bytes:
        return b"".join(
            (
                struct.pack(">I", len(body)),
                kind,
                body,
                struct.pack(">I", zlib.crc32(kind + body) & 0xFFFFFFFF),
            )
        )

    header = struct.pack(">IIBBBBB", FRAME, height, 8, 6, 0, 0, 0)  # 8-bit RGBA
    return b"".join(
        (
            b"\x89PNG\r\n\x1a\n",
            chunk(b"IHDR", header),
            chunk(b"IDAT", zlib.compress(bytes(raw), 9)),
            chunk(b"IEND", b""),
        )
    )


def animation() -> bytes:
    """The animation file: our 32 frames, in our order, with no interpolate claim."""
    return (
        json.dumps(
            {"animation": {"frametime": FRAMETIME, "frames": list(range(FRAMES))}},
            indent=2,
        )
        + "\n"
    ).encode()


def build() -> dict[str, bytes]:
    """Both files, keyed by their path inside the package."""
    return {TEXTURE: png([frame_pixels(index) for index in range(FRAMES)]), ANIMATION: animation()}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--out", type=Path, default=PORT)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    files = build()
    if args.check:
        differences = [
            name
            for name, body in files.items()
            if not (args.out / name).is_file() or (args.out / name).read_bytes() != body
        ]
        print(f"{len(files)} files drawn; {len(differences)} differ")
        for name in differences:
            print(f"  differs: {name}")
        return 1 if differences else 0
    for name, body in files.items():
        target = args.out / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(body)
        print(f"wrote {name} ({len(body)} B)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
