"""Source provenance tests; synthetic graphs never qualify actual FST behavior."""

import contextlib
import io
import json
import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest.mock import patch

import export_plate_fsts as export
from plate_oracle import CORE, digest


class ExportSourceTests(unittest.TestCase):
    def run_export(self, changed=None, split_tn=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            hashes = {}
            for name in CORE:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(name.encode())
                hashes[name] = digest(path.read_bytes())
            oracle = {"source_sha256": hashes}
            manifest = root / "manifest.json"
            manifest.write_text(json.dumps(oracle))
            if changed:
                (root / changed).write_bytes(b"shadow source")
            output = root / "export"
            graph = types.SimpleNamespace(write=lambda name: Path(name).write_bytes(b"synthetic"))
            normalizer = types.SimpleNamespace(tagger=graph, verbalizer=graph)
            package = types.SimpleNamespace(__file__=str(root / "itn/__init__.py"))
            tn_root = root / "shadow" if split_tn else root
            if split_tn:
                (tn_root / "tn").mkdir(parents=True)
                for name in ["tn/processor.py", "tn/token_parser.py"]:
                    (tn_root / name).write_bytes(b"separate TN shadow")
            tn = types.SimpleNamespace(__file__=str(tn_root / "tn/__init__.py"))
            argv = ["export", "--bundle", str(root), "--manifest", str(manifest), "--out", str(output)]
            with (patch.object(sys, "argv", argv), patch.dict(sys.modules, {"itn": package, "tn": tn}),
                  patch.object(export, "freeze", return_value=oracle),
                  patch.object(export, "load_cached_normalizer", return_value=normalizer) as load,
                  contextlib.redirect_stdout(io.StringIO())):
                if changed or split_tn:
                    with self.assertRaisesRegex(ValueError, "loaded oracle module differs"):
                        export.main()
                    load.assert_not_called()
                    self.assertFalse(output.exists())
                else:
                    export.main()
                    load.assert_called_once()
                    receipt = json.loads((output / "export-receipt.json").read_text())
                    self.assertEqual(receipt["oracle"], oracle)
                    self.assertEqual(len(receipt["files"]), 2)

    def test_matching_loaded_sources_export(self):
        self.run_export()

    def test_shadow_itn_source_rejected_before_graph_construction(self):
        self.run_export("itn/chinese/inverse_normalizer.py")

    def test_shadow_tn_source_rejected_before_graph_construction(self):
        self.run_export("tn/processor.py")

    def test_independently_resolved_tn_shadow_is_rejected(self):
        self.run_export(split_tn=True)


if __name__ == "__main__":
    unittest.main()
