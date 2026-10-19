"""Export the frozen delivery base grammar; graph construction stays in this fork."""

import argparse
import hashlib
import json
import tempfile
from pathlib import Path
from plate_oracle import OPTIONS, freeze, load_cached_normalizer


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    oracle = freeze(args.bundle)
    if oracle != json.loads(args.manifest.read_text()):
        raise ValueError("export oracle identity differs")
    options = {**OPTIONS, "enable_plate_semantics": False}
    args.out.mkdir(parents=True, exist_ok=False)
    with tempfile.TemporaryDirectory() as cache:
        normalizer = load_cached_normalizer(cache, options)
        for role, fst in [("tagger", normalizer.tagger), ("verbalizer", normalizer.verbalizer)]:
            fst.write(str(args.out / f"zh_itn_{role}.fst"))
    files = {path.name: dict(size=path.stat().st_size, sha256=hashlib.sha256(path.read_bytes()).hexdigest())
             for path in sorted(args.out.glob("*.fst"))}
    if len(files) != 2:
        raise AssertionError("incomplete FST export")
    receipt = dict(cache_mode="reload_then_optimize", schema="wetext.plate-base-export.v1", oracle=oracle, options=options, files=files)
    (args.out / "export-receipt.json").write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(files, indent=2))


if __name__ == "__main__":
    main()
