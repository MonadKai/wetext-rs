"""Deterministic gates for the differential evidence tool."""

import unittest
from plate_diff import compare


class DiffTests(unittest.TestCase):
    def test_fields_plan_and_missing_ids_are_checked(self):
        row = dict(id="a", result=dict(text="原文", plate=None, plates=[], version="v1"), spans=[])
        expected = {"a": row}
        self.assertEqual(compare(expected, expected, True)["differences"], 0)
        changed = {"a": {**row, "result": {**row["result"], "text": "另一文本"}}}
        self.assertEqual(compare(expected, changed, True)["differences"], 1)
        self.assertEqual(compare(expected, changed, False)["differences"], 0)
        with self.assertRaises(ValueError):
            compare(expected, {}, True)


if __name__ == "__main__":
    unittest.main()
