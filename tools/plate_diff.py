"""Compare complete ID sets and exact result/plan fields; never normalize outputs."""

import argparse
import json


def load(path):
    rows = {}
    with open(path) as stream:
        for line in stream:
            row = json.loads(line)
            if row["id"] in rows:
                raise ValueError(f"duplicate id in {path}: {row['id']}")
            rows[row["id"]] = row
    if not rows:
        raise ValueError(f"empty result set: {path}")
    return rows


def compare(expected, actual, full_text):
    if expected.keys() != actual.keys():
        raise ValueError("missing or extra result IDs")
    fields = ["plate", "plates", "version"] + (["text"] if full_text else [])
    differences = []
    for key, left in expected.items():
        right = actual[key]
        changed = [field for field in fields if left["result"][field] != right["result"][field]]
        if left["spans"] != right["spans"]:
            changed.append("spans")
        if changed:
            differences.append(dict(id=key, fields=changed))
    return dict(rows=len(expected), differences=len(differences), examples=differences[:30])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("expected")
    parser.add_argument("actual")
    parser.add_argument("--full-text", action="store_true")
    args = parser.parse_args()
    result = compare(load(args.expected), load(args.actual), args.full_text)
    print(json.dumps(result, ensure_ascii=False, indent=2))
    raise SystemExit(bool(result["differences"]))


if __name__ == "__main__":
    main()
