# Plate.4 default single-input profile

This profile implements `normalize_with_plates(raw_text)` from the delivered
WeTextProcessing `1.2.0+plate.4`, without a `plate_context` argument. The release
is `plate4-badcase-guarded-20261109`, source
`caf4a6a6fdf81bcba2c7c9c60047f9382ce8ea50`, semantic version
`plate_semantic_v1_6`. Wheel SHA256:
`e9b46980d7f557a63257f749090418044175197b9afde6cc8998d7b6e0614513`.

Use `PlateProfile::structured_plate4` and `with_request_base`. The old v1.5
constructor and rendering selections keep their behavior. One call processes
one original text; no segment, conversation or business state is retained.
No province, vehicle type, candidate list or circle convention is inferred.
An empty PlateContext is a different upstream entrypoint and is not used here.

The lexicon bytes are identical to the registered v1.5 lexicon. Base Chinese
grammar and rendering use the existing cached delivery graphs, subject to the
complete text differential. Context-dependent recovery, clarification outputs
and StreamingPlateITN are outside this profile. Unicode and explicit-kind
nesting resource protection are unchanged.

Freeze and execute the actual installed wheel in an isolated Python 3.12.7
environment (Unicode 15.0.0, matching the existing Rust policy) with
`tools/plate4-oracle-requirements.txt`. `tools/plate_oracle.py --wheel PATH
--freeze` binds its actual files and installed dependencies; use the resulting
manifest with `--input` and `--cache-dir`. The oracle verifies imported modules
against the wheel. Compare every original input's fields, UTF-8 spans and full
text using `tools/plate_diff.py --full-text`. Inputs contain only IDs and text.
Run `examples/plate_replay --plate4 --full-request TAGGER VERBALIZER` for Rust.

The independent rule tests cover positive formats, rejection protection,
same-input correction, absent business context and cross-call isolation.
Corpus and service evidence remain separately identified in the consuming
Parret upgrade; source support is not deployment or quality qualification.
