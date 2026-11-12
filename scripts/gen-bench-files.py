#!/usr/bin/env python3
"""Generate benchmark inputs by concatenating Rust sources until a target size is reached.

Usage: python scripts/gen-bench-files.py [source-dir]
"""

import glob
import os
import sys

TARGETS = [100 * 1024, 1024 * 1024, 10 * 1024 * 1024]
NAMES = ["bench-100kb.rs", "bench-1mb.rs", "bench-10mb.rs"]


def main() -> int:
    src_dir = sys.argv[1] if len(sys.argv) > 1 else "bat-src/src"
    files = sorted(glob.glob(os.path.join(src_dir, "*.rs")))
    if not files:
        print(f"no .rs files found in {src_dir!r}", file=sys.stderr)
        return 1

    chunks = []
    for path in files:
        with open(path, encoding="utf-8", errors="ignore") as handle:
            chunks.append(handle.read())

    joined = "".join(chunks)
    largest = TARGETS[-1]
    if len(joined) < largest:
        repeats = largest // max(len(joined), 1) + 1
        joined = joined * repeats

    for target, name in zip(TARGETS, NAMES):
        with open(name, "w", encoding="utf-8") as handle:
            handle.write(joined[:target])
        print(f"{name}: {os.path.getsize(name)} bytes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())