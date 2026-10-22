# number_plate v1.5 qualification profile

Status: baseline `4c396e8` passes fields/plans and complete text on the frozen
34,136 oracle snapshots. Two new offline raw-text full-string differences are
strict failures of the added plate migration renderer; nine older saved-text
alternatives have their separate historical scope. See the local
[migration issue](../../docs/issues/plate-request-rendering-compatibility.md).
The initial candidate qualification below retains its original cohort identity.
Later Parret P53 adds 35,409 raw normal-final snapshots from its final full.2
34,136-case audio run: zero field/final-plan/text oracle differences. See the
sole Parret plan and restricted `plate-final-full2-p53/` receipts under the
evidence root below. This does not qualify all service fault paths, real BR
protection, independent effects or universal executor correctness; no release
or remote publication is implied.

The oracle is the delivered `wetextprocessing==1.2.0+plate.1` wheel, using
`pynini==2.1.7`, `importlib_resources==6.5.2`, Python 3.12.7 / Unicode 15.0.0.
The machine manifest is authoritative. Schema v2 binds all 332 wheel runtime
Python/data files and the actual imported module origins. The prior eight-file
manifest is preserved as `oracle-manifest.v1.json` for historical provenance;
new oracle/export commands require the v2 manifest. Existing FST assets and
raw-text outputs are not reinterpreted or overwritten. Inputs contain IDs and original text
only; expected plates, reference labels and dataset categories never enter
the algorithm. The result schema includes the delivered four fields plus
independently observable accepted/literal spans in original UTF-8 byte ranges.
`semantics.parse()` status does not replace the span planner.

From the fork root, create an isolated Python environment, install
`tools/oracle-requirements.txt` and the delivery wheel, then run:

```bash
python tools/plate_oracle.py --bundle "$PLATE_BUNDLE" --freeze
python tools/plate_oracle.py --bundle "$PLATE_BUNDLE" \
  --manifest profiles/number_plate_v1_5/oracle-manifest.json \
  --input profiles/number_plate_v1_5/oracle-inputs.jsonl \
  --cache-dir "$PLATE_ORACLE_CACHE" > /tmp/plate-oracle.jsonl
python tools/plate_diff.py profiles/number_plate_v1_5/oracle-outputs.jsonl \
  /tmp/plate-oracle.jsonl --full-text
python -m unittest discover -s tools -p test_plate_diff.py
python -m unittest discover -s tools -p test_export_plate_fsts.py
```

`--structured-only` does not execute full FST normalization and cannot qualify
full-text equivalence. Preparing the full historical raw/reference text corpus:

```bash
python tools/prepare_plate_cases.py --bundle "$PLATE_BUNDLE" --output /tmp/plate-corpus-inputs.jsonl
```

Large corpus inputs/results and exported FSTs stay outside Git. Local development
artifacts live in `/home/kai/MyProjects/VoicePlatform/parret-evaluation-evidence/itn-20261019`.
The 83 checked-in cases cover portable positives, explicit negatives, Unicode,
escaping, preservation and correction. Shared normalized text is never fed back
as original input. This fork owns both the rules and their complete renderer;
Parret's Adapter must not independently reorder them.

Explicit symbol-kind prefixes are limited to 64 nested declarations. Deeper
input is an execution error before recursive parsing, rather than an empty
plate or a process stack overflow. The limit is part of effective parameters.

Rust qualification runner (the algorithm receives only `text`):

```bash
cargo build --offline --locked --example plate_replay
target/debug/examples/plate_replay < /tmp/plate-corpus-inputs.jsonl > /tmp/plate-rust.jsonl
python tools/plate_diff.py /tmp/plate-oracle-corpus.jsonl /tmp/plate-rust.jsonl
```

`PlateProfile::structured` does not claim generic full-text normalization.
`with_base` binds the exact exported graphs; it preserves one plan while
normalizing only unrelated fragments and retained context. `CompiledChineseItn`
fails on missing graph paths or invalid byte/token output instead of returning
internal FST tokens. Registry consumers can load verified byte buffers without
reopening asset paths. Profiles own their effective Unicode/rendering parameters.

`with_request_base` is the separate full.2 candidate: construct/optimize the
complete delivered request before choosing tagger paths, reorder tokens, then
execute the complete semantic/literal verbalizer union. Retained context uses
the delivered semantic dispatch; fields still come from the original outer
plan. Its effective rendering parameter is `full_text_request_graph_v1`.
`plate_replay --full-request tagger.fst verbalizer.fst` selects it explicitly;
`--full` and `with_base` retain legacy full.1 behavior. Do not overwrite old
executables or use the active evaluation's Cargo target when building it.
Semantics, spans, lexicon and the delivered result version stay unchanged.
All original/new snapshots require separate strict field/plan/text comparison
before any Parret binding or real-service qualification.
