"""Build text-only inputs; algorithm input never contains references or gold."""

import argparse
import ast
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--fixtures", action="store_true")
    args = parser.parse_args()
    source = args.bundle / "itn/WeTextProcessing/itn/chinese"
    rows = []
    if args.fixtures:
        rows += [dict(id=f"positive-{i}", text=pair[0]) for i, pair in enumerate(
            json.loads((source / "plate/fixtures.json").read_text()))]
        tree = ast.parse((source / "test/plate_parser_test.py").read_text())
        for node in ast.walk(tree):
            if isinstance(node, ast.FunctionDef) and node.name == "test_no_guessing":
                for loop in ast.walk(node):
                    if isinstance(loop, ast.For) and isinstance(loop.iter, ast.List):
                        rows += [dict(id=f"negative-{i}", text=ast.literal_eval(item))
                                 for i, item in enumerate(loop.iter.elts)]
        for i, text in enumerate(["", " ", "苏B一\"二\\三四五", "苏Ｂ五ｊ，也就是字母ｊ，七八九",
                                  "苏B一二三四五，金额三百元，最后一位是六",
                                  "苏B一二三四五，金额三百元，车牌最后一位是六",
                                  "苏B一二三四五和苏C六七八九零", "e\u0301苏B一二三四五", "ﬃ苏B一二三四五"]):
            rows.append(dict(id=f"edge-{i}", text=text))
        assert len(rows) >= 80
    else:
        with (args.bundle / "evaluation/predictions.jsonl").open() as stream:
            for line in stream:
                record = json.loads(line)
                for field in ["raw_text", "reference"]:
                    rows.append(dict(id=f"{record['id']}:{field}", text=record[field]))
        assert len(rows) == 34136
    assert len({row["id"] for row in rows}) == len(rows)
    with args.output.open("x") as stream:
        for row in rows:
            stream.write(json.dumps(row, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
