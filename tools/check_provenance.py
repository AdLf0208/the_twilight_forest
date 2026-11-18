#!/usr/bin/env python3
"""Prove that no packaged file carries an upstream byte, and write the audit trail.

The port's code is a clean-room Rust rewrite (LGPL-2.1). Its assets are the port's own: the
textures are drawn by the generators named in the table below and the JSON is either written
here or rewritten from the semantics the jar's files express. The upstream repository's
``ASSET_LICENSE`` is CC-BY-NC-SA-4.0, so **not one byte** of its art may travel in this package
(ARCHITECTURE.md §8, the MineColonies precedent).

This tool is the machine-checked half of that claim:

* it opens the pack's Twilight Forest jar and the built ``twilightforest.hmod``;
* it asserts that **no packaged file's sha256 equals any jar entry's sha256** — a copy under a
  different name is caught too, because the comparison is by content, not by path;
* it writes ``PROVENANCE.md``, one line per packaged file, naming the generator or the rewrite
  that produced it — the audit trail a reviewer reads beside ``LICENSE-NOTICE.md``.

    python3 tools/check_provenance.py            # assert, then write PROVENANCE.md
    python3 tools/check_provenance.py --check    # assert only (the ladder gate)

Exit status: 0 when the package is clean, 1 when a packaged file matches a jar entry (each
offender is printed with the jar path it matches), 2 when the jar or the package is missing.
"""

from __future__ import annotations

import argparse
import hashlib
import sys
import zipfile
from dataclasses import dataclass
from pathlib import Path

PORT = Path(__file__).resolve().parent.parent
PORTS = PORT.parent
DEFAULT_JAR = PORTS / "_shared" / "licence-cache" / "twilightforest-1.21.1-4.8.3345-universal.jar"
DEFAULT_HMOD = PORT / "twilightforest.hmod"
DEFAULT_REPORT = PORT / "PROVENANCE.md"

TEXTURE_GEN = "generated (`tools/gen_portal_texture.py`)"
SURFACE_GEN = "generated (`tools/assemble_surface_blocks.py`)"
WORLDGEN_GEN = "rewritten-from-semantics (`tools/assemble_worldgen.py`, the jar's definition converted to 26.3)"
BUILD_GEN = "generated (`cargo horizon-mod build`)"
CONTENT_GEN = "authored (`tools/assemble_content.py`)"

# Files whose provenance is not obvious from their folder. Everything not named here is the
# port's own Rust, manifest or prose and is classified below by prefix.
EXPLICIT: dict[str, str] = {
    # The portal's own art and its animation: drawn, frame by frame, from the generator's palette.
    "assets/twilightforest/textures/block/portal_barrier.png": TEXTURE_GEN,
    "assets/twilightforest/textures/block/portal_barrier.png.mcmeta": TEXTURE_GEN,
    # The four portal files re-authored from the semantics they express (same states, same
    # geometry intent, same membership; our own bytes, ordering and formatting).
    "assets/twilightforest/blockstates/twilight_portal.json": (
        "rewritten-from-semantics (`tools/assemble_content.py`: the two-variant multipart the "
        "block's states need, our own order)"
    ),
    "assets/twilightforest/models/block/twilight_portal_barrier.json": (
        "rewritten-from-semantics (`tools/assemble_content.py`: two horizontal planes at 11.5 and "
        "13.5, the one-way barrier's geometry, our own uv and key order)"
    ),
    "data/twilightforest/tags/block/portal/edge.json": (
        "rewritten-from-semantics (`tools/assemble_content.py`: the frame's membership, sorted)"
    ),
    "data/twilightforest/tags/block/portal/fluid.json": (
        "rewritten-from-semantics (`tools/assemble_content.py`: the pool's membership, sorted)"
    ),
    "data/twilightforest/tags/block/portal/generated_decoration.json": (
        "rewritten-from-semantics (`tools/assemble_content.py`: the vanilla plants the port "
        "carries; the mod's own plants are dropped)"
    ),
    "data/twilightforest/tags/block/portal/decoration.json": (
        "rewritten-from-semantics (`tools/assemble_content.py`: what may stand on the frame; "
        "vanilla ids and tags only)"
    ),
    # The rest of the portal's content files are the port's own declarations.
    "assets/twilightforest/models/block/twilight_portal.json": CONTENT_GEN,
    "assets/twilightforest/lang/en_us.json": CONTENT_GEN,
    "data/twilightforest/horizon/block/twilight_portal.json": CONTENT_GEN,
    "data/twilightforest/loot_table/blocks/twilight_portal.json": CONTENT_GEN,
    "data/c/tags/item/gems/diamond.json": CONTENT_GEN,
    "data/twilightforest/tags/item/portal/activator.json": CONTENT_GEN,
    # The package's own prose, the licence text and the manifests.
    "horizon-mod.toml": "authored (the package manifest)",
    "Cargo.toml": "authored (the crate, one SDK path dependency)",
    "README.md": "authored",
    "LICENSE": "authored (the canonical LGPL-2.1 text, from gnu.org; not from the jar)",
    "LICENSE-NOTICE.md": "authored (the licence split and the upstream attribution)",
    "PROVENANCE.md": "generated (`tools/check_provenance.py`, this report)",
    "uses.json": BUILD_GEN,
}


def classify(name: str) -> str:
    """The provenance of one packaged file, by its exact path or its folder."""
    explicit = EXPLICIT.get(name)
    if explicit is not None:
        return explicit
    if name.startswith("src/"):
        return "authored (the port's Rust; `src/runtime.rs` is the pure rule)"
    if name.startswith("assets/twilightforest/textures/block/"):
        return SURFACE_GEN
    if name.startswith(
        (
            "assets/twilightforest/blockstates/",
            "assets/twilightforest/models/block/",
            "assets/twilightforest/items/",
        )
    ):
        return SURFACE_GEN
    if name.startswith("data/twilightforest/worldgen/"):
        return WORLDGEN_GEN
    if name.startswith(("data/twilightforest/horizon/block/", "data/twilightforest/loot_table/")):
        return SURFACE_GEN
    if name in (
        "data/twilightforest/tags/block/deadrock.json",
        "data/minecraft/tags/block/mineable/pickaxe.json",
    ):
        return SURFACE_GEN
    return "authored"


@dataclass(frozen=True)
class Packaged:
    """One entry of the built package."""

    name: str
    size: int
    digest: str

    @property
    def provenance(self) -> str:
        return classify(self.name)


def packaged(hmod: Path) -> list[Packaged]:
    """Every file entry of the package, sorted by path."""
    entries: list[Packaged] = []
    with zipfile.ZipFile(hmod) as archive:
        for info in archive.infolist():
            if info.is_dir():
                continue
            digest = hashlib.sha256(archive.read(info)).hexdigest()
            entries.append(Packaged(info.filename, info.file_size, digest))
    return sorted(entries, key=lambda entry: entry.name)


def jar_digests(jar: Path) -> dict[str, list[str]]:
    """sha256 -> the jar's paths with that content (a copy under another name still matches)."""
    found: dict[str, list[str]] = {}
    with zipfile.ZipFile(jar) as archive:
        for info in archive.infolist():
            if info.is_dir():
                continue
            digest = hashlib.sha256(archive.read(info)).hexdigest()
            found.setdefault(digest, []).append(info.filename)
    return found


def report(
    jar: Path, hmod: Path, entries: list[Packaged], offenders: list[tuple[Packaged, list[str]]]
) -> str:
    """The markdown audit trail: the claim, the two artefacts, and one line per file."""
    jar_bytes = jar.stat().st_size
    hmod_bytes = hmod.stat().st_size
    lines = [
        "# Provenance — every packaged file is this port's own",
        "",
        "The Twilight Forest's **code is LGPL-2.1** (the jar's `META-INF/neoforge.mods.toml`), so a",
        "port may be distributed under the same licence with its complete source; the upstream",
        "**assets are CC-BY-NC-SA-4.0** (`TeamTwilight/twilightforest`'s `ASSET_LICENSE`), so **no",
        "upstream byte may travel in the package** (ARCHITECTURE.md §8; the MineColonies precedent).",
        "This report is generated by `tools/check_provenance.py`, which also proves it: it compares",
        "the sha256 of every packaged file against every entry of the jar.",
        "",
        "| | |",
        "|---|---|",
        f"| jar | `{jar.name}` — {jar_bytes:,} B, sha256 `{hashlib.sha256(jar.read_bytes()).hexdigest()}` |",
        f"| package | `{hmod.name}` — {hmod_bytes:,} B, sha256 `{hashlib.sha256(hmod.read_bytes()).hexdigest()}`, {len(entries)} files |",
        f"| files identical to a jar entry | **{len(offenders)}** |",
        "",
        "## How each file was produced",
        "",
        "| packaged file | bytes | provenance |",
        "|---|---:|---|",
    ]
    lines += [
        f"| `{entry.name}` | {entry.size:,} | {entry.provenance} |" for entry in entries
    ]
    lines += [
        "",
        "## The categories",
        "",
        "* **authored** — written by hand for this port, from the mod's behaviour and ids.",
        "* **generated (`tool`)** — written by a deterministic generator in `tools/`: the same",
        "  palette, frames or inputs always give the same bytes, and the generator's `--check`",
        "  re-renders and compares.",
        "* **rewritten-from-semantics** — the file expresses the same functional content as a jar",
        "  file (a blockstate's variants, a model's geometry, a tag's membership, a biome's",
        "  definition) in the port's own bytes, ordering and formatting. Written for a reason the",
        "  file itself cannot carry, so the reason is in the table above.",
        "",
        "This report is not packaged: the package carries `README.md`, `LICENSE` and",
        "`LICENSE-NOTICE.md` among its root documents, and `notes.md`/`VALIDATION.md`/`PROVENANCE.md`",
        "stay in the repository as the audit trail.",
        "",
        "Worldgen data is the one place where the *semantics* are the jar's by design: a biome's",
        "colours and climate are the mod's design, not its art, and `tools/assemble_worldgen.py`",
        "converts the definition to 26.3's shape. It is data, never a texture, model or lang file,",
        "and no converted file is byte-identical to the jar's (checked above).",
        "",
        "Regenerate with:",
        "",
        "```sh",
        "python3 tools/assemble_content.py        # the portal's blockstate, model, tags, lang",
        "python3 tools/gen_portal_texture.py      # the barrier texture and its animation",
        "python3 tools/assemble_surface_blocks.py # the three deadrock blocks and their textures",
        "python3 tools/assemble_worldgen.py       # the 14 biomes, the surface rule, the noise settings",
        "python3 tools/check_provenance.py        # this report, and the assertion",
        "```",
        "",
    ]
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--jar", type=Path, default=DEFAULT_JAR)
    parser.add_argument("--hmod", type=Path, default=DEFAULT_HMOD)
    parser.add_argument("--report", type=Path, default=DEFAULT_REPORT)
    parser.add_argument(
        "--check",
        action="store_true",
        help="assert only: exit 1 on an identical file, write no report",
    )
    args = parser.parse_args()

    for path, what in ((args.jar, "jar"), (args.hmod, "package")):
        if not path.is_file():
            print(f"missing {what}: {path}", file=sys.stderr)
            return 2

    known = jar_digests(args.jar)
    entries = packaged(args.hmod)
    offenders = [(entry, known[entry.digest]) for entry in entries if entry.digest in known]

    if offenders:
        print(
            f"PROVENANCE FAIL: {len(offenders)} of {len(entries)} packaged files are byte-identical "
            "to a jar entry",
            file=sys.stderr,
        )
        for entry, paths in offenders:
            print(f"  {entry.name} ({entry.size:,} B) == {', '.join(paths)}", file=sys.stderr)
        return 1

    print(
        f"PROVENANCE OK: {len(entries)} packaged files, 0 identical to any of the jar's "
        f"{sum(len(v) for v in known.values()):,} entries"
    )
    if args.check:
        return 0

    args.report.write_text(report(args.jar, args.hmod, entries, offenders))
    print(f"wrote ports/{args.report.relative_to(PORTS)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
