"""Freeze the oracle's case/category policy; never depend on host Rust Unicode."""

import json
import unicodedata
from pathlib import Path


def main():
    assert unicodedata.unidata_version == "15.0.0", "use the pinned oracle Python"
    upper = {}
    ranges = []
    start = 0
    previous = None
    for point in range(0x110001):
        flags = 0
        if point < 0x110000:
            char = chr(point)
            kind = unicodedata.category(char)[0]
            flags = (1 if kind == "P" else 0) | (2 if kind in "CZ" or char.isspace() else 0) | (4 if kind == "C" else 0)
            if char.upper() != char:
                upper[char] = char.upper()
        if flags != previous:
            if previous:
                ranges.append([start, point - 1, previous])
            start, previous = point, flags
    output = Path(__file__).resolve().parent.parent / "profiles/number_plate_v1_5/unicode-15.json"
    output.write_text(json.dumps(dict(version=unicodedata.unidata_version, upper=upper, ranges=ranges), ensure_ascii=True, separators=(",", ":")) + "\n")


if __name__ == "__main__":
    main()
