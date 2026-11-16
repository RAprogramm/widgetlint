# SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
# SPDX-License-Identifier: MIT

#!/usr/bin/env python3
"""Rewrite the workspace member versions in Cargo.lock to the release version.

The release pipeline bumps the workspace version in Cargo.toml and must keep
Cargo.lock consistent for `cargo build --locked`. A package in Cargo.lock is
a workspace member exactly when it carries no `source` key, so members are
detected from the lockfile itself and no name list is hard-coded here.

Usage:

    python3 tools/bump_lock.py <version> [lockfile]

The lockfile defaults to `Cargo.lock` next to the repository root. The script
exits with an error when the file is missing or when no member was rewritten,
so a malformed lockfile can never pass the bump silently.

Example:

    python3 tools/bump_lock.py 0.1.1 Cargo.lock
"""

import sys
from pathlib import Path


def bump_lock(text: str, version: str) -> tuple[str, int]:
    """Return the lockfile text with every member set to version and the count.

    Args:
        text: full Cargo.lock contents.
        version: the semantic version to write into every member block.

    Returns:
        A tuple of the rewritten text and the number of rewritten members.

    Example:
        >>> text = '[[package]]\\nname = "wl-core"\\nversion = "0.1.0"\\n'
        >>> bumped, count = bump_lock(text, "0.2.0")
        >>> count
        1
        >>> 'version = "0.2.0"' in bumped
        True
    """
    result: list[str] = []
    block: list[str] = []
    rewritten = 0

    def emit(pending: list[str]) -> None:
        nonlocal rewritten
        if (
            pending
            and pending[0] == "[[package]]\n"
            and not any(line.startswith("source = ") for line in pending)
        ):
            for index, line in enumerate(pending):
                if line.startswith("version = "):
                    pending[index] = f'version = "{version}"\n'
                    rewritten += 1
                    break
        result.extend(pending)

    for line in text.splitlines(keepends=True):
        if line.startswith("[[package]]"):
            emit(block)
            block = [line]
        elif line.startswith("["):
            emit(block)
            block = []
            result.append(line)
        else:
            block.append(line)
    emit(block)
    return "".join(result), rewritten


def main(argv: list[str]) -> int:
    """Rewrite the lockfile named by argv; return a process exit code."""
    if len(argv) < 2 or len(argv) > 3:
        print("usage: bump_lock.py <version> [lockfile]", file=sys.stderr)
        return 2
    version = argv[1]
    lock = Path(argv[2] if len(argv) == 3 else "Cargo.lock")
    if not lock.is_file():
        print(f"lockfile not found: {lock}", file=sys.stderr)
        return 1
    text = lock.read_text(encoding="utf-8")
    bumped, count = bump_lock(text, version)
    if count == 0:
        print(f"no workspace members found in {lock}", file=sys.stderr)
        return 1
    if bumped != text:
        lock.write_text(bumped, encoding="utf-8")
    print(f"set {count} workspace members to {version}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
