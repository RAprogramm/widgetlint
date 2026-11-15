#!/usr/bin/env python3
"""Parity gate: compare the widgetlint zlib-9 counter with `zlib.compress(data, 9)`.

The server measures a widget as CPython's `zlib.compress(html, 9)`. This script
builds a deterministic corpus (>= 500 files), compresses every file with the
reference zlib and with the `widgetlint size --format json` binary, and fails
on the first byte-count mismatch.

Some distros (Arch and friends) ship zlib-ng as the system zlib, whose deflate
output differs from the reference. The script prints the zlib implementation
and refuses to run on zlib-ng. On such machines run it with a real-zlib
interpreter, for example:

    uv run --python 3.12 tools/zlib_parity.py [path/to/widgetlint]
"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import zlib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_BINARY = REPO_ROOT / "target" / "debug" / "widgetlint"

HTML_UNIT = b"<div class='card'><p>Hello</p></div>"
TEXT_UNIT = "привет мир виджет".encode()


def require_real_zlib() -> None:
    """Abort unless the interpreter links the reference zlib."""
    runtime = zlib.ZLIB_RUNTIME_VERSION
    if "zlib-ng" in runtime:
        sys.exit(
            f"refusing to run: zlib runtime is '{runtime}' (zlib-ng differs from the "
            "reference); use a real-zlib interpreter, e.g. "
            "`uv run --python 3.12 tools/zlib_parity.py`"
        )
    print(f"reference zlib: version {zlib.ZLIB_VERSION}, runtime {runtime}")


def lcg_bytes(length: int, seed: int) -> bytes:
    """Deterministic pseudo-random bytes (linear congruential generator)."""
    state = seed & 0xFFFFFFFF
    out = bytearray(length)
    for i in range(length):
        state = (state * 1103515245 + 12345) & 0xFFFFFFFF
        out[i] = (state >> 16) & 0xFF
    return bytes(out)


def build_corpus() -> dict[str, bytes]:
    """Return the deterministic corpus: boundaries, repeats, random blocks."""
    files: dict[str, bytes] = {
        "empty": b"",
        "one_byte": b"a",
        "zeros_4095": bytes(4095),
        "zeros_4096": bytes(4096),
        "zeros_4097": bytes(4097),
        "zeros_32768": bytes(32768),
        "html_4095": (HTML_UNIT * 200)[:4095],
        "html_4096": (HTML_UNIT * 200)[:4096],
        "html_4097": (HTML_UNIT * 200)[:4097],
        "html_32768": (HTML_UNIT * 2000)[:32768],
        "utf8_50": TEXT_UNIT * 50,
        "utf8_32768": (TEXT_UNIT * 5000)[:32768],
        "ab_repeat_1000": b"ab" * 1000,
        "alpha_tail": bytes(range(256)) * 128,
    }
    for i in range(250):
        files[f"random_{i}"] = lcg_bytes(1 + i * 131, seed=0x9E3779B9 + i)
    for i in range(120):
        files[f"html_like_{i}"] = (HTML_UNIT * (1 + i))[:32768]
    for i in range(120):
        mixed = lcg_bytes(64, seed=7919 * i) + HTML_UNIT * (1 + i % 50)
        files[f"mixed_{i}"] = mixed[:32768]
    return files


def widgetlint_zlib9(binary: Path, path: Path) -> int:
    """Return the zlib-9 size reported by the binary for one file."""
    proc = subprocess.run(
        [str(binary), "size", str(path), "--format", "json"],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode not in (0, 1):
        sys.exit(
            f"widgetlint failed on {path.name} (exit {proc.returncode}): {proc.stderr}"
        )
    report = json.loads(proc.stdout)
    return int(report["size"]["zlib9"])


def main() -> int:
    require_real_zlib()
    binary = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_BINARY
    if not binary.is_file():
        sys.exit(f"binary not found: {binary} (run `cargo build` first)")
    corpus = build_corpus()
    if len(corpus) < 500:
        sys.exit(f"corpus too small: {len(corpus)} files, need >= 500")
    failures = 0
    with tempfile.TemporaryDirectory() as tmp:
        tmpdir = Path(tmp)
        for index, (name, data) in enumerate(corpus.items(), start=1):
            path = tmpdir / name
            path.write_bytes(data)
            expected = len(zlib.compress(data, 9))
            actual = widgetlint_zlib9(binary, path)
            if expected != actual:
                failures += 1
                print(
                    f"MISMATCH {name}: raw={len(data)} python={expected} rust={actual}"
                )
            if index % 100 == 0:
                print(f"checked {index}/{len(corpus)}")
    print(f"corpus: {len(corpus)} files, mismatches: {failures}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
