"""Pinned delivery oracle. Inputs are original text only; never gold labels."""

import argparse
import dataclasses
import hashlib
import importlib.metadata
import json
import platform
import zipfile
from pathlib import Path

OPTIONS = dict(remove_interjections=True, enable_standalone_number=True,
               enable_0_to_9=False, enable_million=False, enable_plate_semantics=True)
CORE = ["itn/chinese/inverse_normalizer.py", "itn/chinese/plate/semantics.py",
        "itn/chinese/plate/spans.py", "itn/chinese/plate/lexicon.json",
        "itn/chinese/plate/rule.py", "itn/chinese/plate/fixtures.json",
        "tn/processor.py", "tn/token_parser.py"]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def freeze(bundle):
    wheel = bundle / "itn/wetextprocessing-1.2.0+plate.1-py3-none-any.whl"
    source = bundle / "itn/WeTextProcessing"
    with zipfile.ZipFile(wheel) as archive:
        hashes = {name: digest(archive.read(name)) for name in CORE}
        for name, expected in hashes.items():
            if digest((source / name).read_bytes()) != expected:
                raise ValueError(f"source differs from wheel: {name}")
    return dict(schema="wetext.plate-oracle.v1", wheel_sha256=digest(wheel.read_bytes()),
                package="1.2.0+plate.1", semantic_version="plate_semantic_v1_5",
                options=OPTIONS, base_cache_mode="reload_then_optimize", source_sha256=hashes,
                dependencies={name: importlib.metadata.version(name) for name in
                              ["pynini", "importlib_resources", "WeTextProcessing"]},
                python=platform.python_version(), unicode=__import__("unicodedata").unidata_version)


def load_cached_normalizer(cache_dir, options):
    """Freeze delivery cache-hit behavior; cold-build tie ordering can differ."""
    from itn.chinese.inverse_normalizer import InverseNormalizer
    InverseNormalizer(cache_dir=cache_dir, **options)
    return InverseNormalizer(cache_dir=cache_dir, **options)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", type=Path, required=True)
    parser.add_argument("--freeze", action="store_true")
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--input", type=Path)
    parser.add_argument("--cache-dir", type=Path)
    parser.add_argument("--structured-only", action="store_true")
    args = parser.parse_args()
    current = freeze(args.bundle)
    if args.freeze:
        print(json.dumps(current, ensure_ascii=False, indent=2))
        return
    if not args.manifest or not args.input:
        parser.error("--manifest and --input required for oracle execution")
    if json.loads(args.manifest.read_text()) != current:
        raise ValueError("oracle identity differs from frozen manifest")
    from itn.chinese.plate.spans import plate_spans
    from itn.chinese.plate.semantics import VERSION
    # Require the installed wheel's actual modules to match, not a shadow copy.
    import itn
    package_root = Path(itn.__file__).resolve().parent.parent
    for name, expected in current["source_sha256"].items():
        if digest((package_root / name).read_bytes()) != expected:
            raise ValueError(f"loaded oracle module differs: {name}")
    normalizer = None
    if not args.structured_only:
        if args.cache_dir is None:
            parser.error("--cache-dir required for full text")
        normalizer = load_cached_normalizer(args.cache_dir, OPTIONS)
    ids = set()
    with args.input.open() as stream:
        for number, line in enumerate(stream, 1):
            record = json.loads(line)
            if set(record) != {"id", "text"} or not isinstance(record["text"], str):
                raise ValueError(f"line {number}: only id and original text accepted")
            if record["id"] in ids:
                raise ValueError(f"duplicate id: {record['id']}")
            ids.add(record["id"])
            raw = record["text"]
            plan = plate_spans(raw)
            spans = []
            for span in plan:
                value = dataclasses.asdict(span)
                value["start"] = len(raw[:span.start].encode())
                value["end"] = len(raw[:span.end].encode())
                spans.append(value)
            plates = list(dict.fromkeys(span.output for span in plan if span.accepted))
            result = dict(plate=plates[0] if len(plates) == 1 else None,
                          plates=plates, version=VERSION)
            if normalizer:
                full = normalizer.normalize_with_plates(raw)
                if any(full[key] != result[key] for key in result):
                    raise AssertionError("oracle result and span plan diverged")
                result = full
            print(json.dumps(dict(id=record["id"], result=result, spans=spans), ensure_ascii=False))


if __name__ == "__main__":
    main()
