# PLATE-MIGRATION-1: reproduce delivered complete request rendering

Status: local issue draft and compatibility implementation in progress. No remote
issue or release. This concerns the plate migration added in this fork, not an
established upstream wetext-rs bug or an error in number_plate_v1.0.0.
Parret's `docs/plans/itn-multiple-implementations-plan.md` is the sole cross-repo
progress entry; this document owns the defect/reproduction contract.

## Baseline and expected behavior

Delivered wheel `wetextprocessing 1.2.0+plate.1`, `plate_semantic_v1_5`, its actual
integration/provenance/source/options/dictionary/order are authoritative. Frozen
wheel SHA256 `8cf7b2633c91d227b702c7746571b62e81d9c26379dade4cd3b2cb38cc230d69`.
Old fork source `4c396e8125f0b6426d115d9100115028b656159a`, full.1 replay artifact
SHA256 `98abf05dc3984e36b618c52d8160c168a85061b39c045a3a60e82d1918873d78`.
The old source/tag/build stay immutable; new builds use a separate target and
new effective source/build/parameters before any Adapter adoption.

For the same original snapshot compare structured fields, final accepted span
plan and complete text separately and exactly. No gold input, business-rule
optimization, special text patches or relaxed differential is permitted.

## Observed and minimally reproduced

The full offline B original-wheel audit has two text differences across 34,136
rows: both rates of one original raw snapshot. Structured fields and complete
span plans match. The strict failure is retained; equal plate scores do not
qualify complete text. The old migration normalizes gaps separately and joins
outputs, whereas delivered `rule.specialize()` composes and optimizes the full
request before shortest-path selection and final verbalization.

For one differing original raw text, exporting the unchanged delivered complete
request graph and using the existing Rust FST/token executor reproduces the
original text. Rust optimization of the corresponding unoptimized whole-request
graph also reproduces it. This narrows attribution; it does not establish
universal executor correctness. Same minimum weights or multiple valid paths
alone do not establish any lower-layer bug or excuse text differences.

Private evidence under `parret-evaluation-evidence/itn-20261019/`:

- `full-audio-v1/offline-python-oracle-v2/`: strict diff, original commands,
  same-weight paths, render-order probe and asset identities.
- `full-audio-client-io-v2/scope-lock-v1/whole-request-probe/`: original text,
  expected/actual outputs, export/probe sources and executable/library/FST hashes.
- `full-text-request-v2/`: new implementation qualification and protected old
  dependency manifest. Raw inputs and outputs remain restricted, never Git.

Reproduce using the recorded commands with fresh output paths. Algorithm inputs
are original raw text only; compare expected oracle results after computation.

## Change and attribution boundary

Port the delivered whole-request construction, optimize/shortest-path order,
semantic/literal token verbalizers, escaping and retained-context processing.
Do not change semantics.py/span/lexicon behavior or streaming algorithms.
Keep legacy full.1 explicit and candidate request rendering distinct (full.2).

If a separate FST/shortest-path/token/escaping executor defect is independently
found, commit another local fork issue with minimal input, expected/actual,
source/build/dependency/features/assets, impact and tests before fixing it.
Associate that fix commit with its issue. Delivered behavior/limits are a third
class, not automatically defects. No remote issue is authorized.

## Required closure evidence

Strict field/plan/text equality on frozen plus new raw snapshots; coverage of
no span, protected fragments, single/multiple plates, correction/re-report,
retained context, Unicode/quotes/backslashes and explicit tied path choices.
Retain every failure. Bind the new commit, Cargo.lock/features, artifacts/assets
and effective parameters. Real Adapter/normal-final/fallback/BR and 8/16 kHz
service qualification remains separate from snapshot replay. Link the eventual
fix commit and results here only after actual qualification.
