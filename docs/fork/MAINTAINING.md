# Maintaining the owner fork

See the [consumption snapshot index](README.md) for remote tags, full source
SHAs, profiles, parameters, asset hashes and actual consumer revisions.
Keep new fork-specific guidance here instead of adding it to the upstream README.
The profile overview below was moved from that README without changing runtime
code, manifests, fixtures or LICENSE/NOTICE attribution.

## Toolchain and profile contract

This owner fork is qualified with Rust 1.95.0 and its committed lockfile.
Earlier compiler versions are not part of the fork's current support contract.
The root Rust badge remains consistent with `Cargo.toml`'s actual minimum version.

`PlateProfile::structured_plate4` selects the default, context-free
`plate_semantic_v1_6` behavior from WeTextProcessing `1.2.0+plate.4`.
Combine it with `with_request_base` for full text. See
[the plate.4 profile](../../profiles/number_plate_v1_6/README.md).
The original `structured` constructor retains v1.5 rules.

The owner fork adds `plate::PlateProfile` and `CompiledChineseItn` independently
from the legacy automatic-language normalizer. Chinese must be selected
explicitly by callers. Structured-only preserves unrelated text;
`with_request_base` selects complete full.2 delivery rendering. `with_base`
retains legacy full.1 fragment rendering with its recorded text differences.
Fields always come from one accepted span plan. Original span coordinates are
UTF-8 byte offsets. A null unique plate is not an abstention reason.

The frozen wheel, options and Unicode policy live under
`profiles/number_plate_v1_5/`; `tools/plate_oracle.py` runs only original texts,
`tools/plate_diff.py --full-text` compares every result and plan field, and
`tools/export_plate_fsts.py` exports the same-option base grammar. Export and
oracle deliberately prime a fresh cache then reload/optimize as delivery does
on a cache hit: initial-build and cached graphs can choose different
same-weight numeric segmentations. The profile identity records this policy.
Both commands verify the imported package's source hashes before constructing
normalizers; checking distribution metadata or bundle files alone is insufficient.
Do not replace it with an arbitrary legacy FST bundle. Rust runtime builds
never construct or download FSTs.

Explicit real-asset regression (94 v1.5 snapshots plus the plate.4 full-text test):

```bash
WETEXT_TAGGER=/path/to/cached-export/zh_itn_tagger.fst \
WETEXT_VERBALIZER=/path/to/cached-export/zh_itn_verbalizer.fst \
cargo test --offline --locked --test plate_full -- --ignored
```

Explicit symbol-kind nesting is bounded at 64 and returns `InvalidInput` above
that boundary; it does not turn resource exhaustion into a no-plate result.
Unicode 15 normalization/uppercase/classification follows the frozen Python
oracle rather than the host's newer character database. Check the consuming
Parret implementation plan for revision-specific corpus and ASR qualification.

## Rule change and consumer workflow

1. Work on a new branch at a verified clean owner Base. Change rules, format
   validation, spans and full-text rendering here. Preserve old constructors and
   profile behavior, or explicitly retire a profile with a separately approved
   consumer migration; never silently reuse an old version for new rules.
2. Pin the real wheel, its source hashes, import origins, dependencies, Unicode
   and cache path. Use `tools/plate_oracle.py --wheel ... --freeze` for plate.4
   or the existing `--bundle` route for the old delivery. Do not pass
   `plate_context` for the default profile. Keep context/streaming behavior out
   of a single-final default upgrade.
3. Run fmt, offline locked library/profile tests, all-target Clippy with denied
   warnings, Python tool tests and the explicit real-FST tests above. Compare
   plate(s), original UTF-8 plans and complete text on matching originals;
   independently check positive, rejection, multiple-plate and escape cases.
   Preserve failures, fallback and accepted conservative upstream behavior.
4. Commit/review the exact source, check source-derived fixtures and **all
   ancestors** for publication suitability, then publish a new annotated source
   tag. The pattern is `parret/itn-source-<short-sha>`; use the full SHA as target
   and record profile/reference/consumer scope in its annotation. Inspect remote
   names before creation. Verify the tag object and its peeled commit after push;
   never move, replace or force-push an existing frozen ref.
5. Generate the Parret vendor into a new destination through its existing tool:

   ```bash
   python3 scripts/check_vendor_wetext.py --fork /absolute/owner-fork \
     --revision FULL_OWNER_SHA --destination /absolute/new-vendor-snapshot
   python3 scripts/check_vendor_wetext.py
   ```

   These commands run from a matching Parret checkout: export first, review and
   install the generated snapshot, then run the second command. Never hand-edit
   generated vendor rules. `SOURCE.json` binds the original source archive and
   file hashes; the original lock is retained as `Cargo.lock.upstream`, while
   actual builds use Parret's workspace lock and dependency resolution.
6. Record Parret source, actual executable/image, startup profile, semantic
   identity, execution parameters and model/tokenizer/FST/lexicon hashes
   separately. Source-tree equality supports bounded evidence reuse but does
   not rename a binary or qualify a new deployment. Repeat only checks whose
   inputs or applicability changed. An expensive audio rerun needs a named
   unresolved runtime uncertainty.

For tag verification, compare both `refs/tags/TAG` and `refs/tags/TAG^{}` from
`git ls-remote --tags origin`; locally require `git cat-file -t TAG` to be
`tag` and `git rev-parse TAG^{}` to equal the complete pinned source SHA.
An annotated source tag is not a signature, package release or image receipt.

## Upstream synchronization and rollback

Keep the upstream remote/source distinct from this owner fork. Review upstream
updates on a separate branch and assess conflicts against the fork's compiled
ITN, plate parser/planner/rendering, Unicode policy, tests, oracle/export tools,
profile manifests and dependency/toolchain requirements. Preserve source/license
provenance in LICENSE/NOTICE; those are not optional fork prose. Keep this
documentation directory and per-profile documentation out of the original
README's fork-specific narrative. Do not rewrite historical commits or tags to
make an upstream sync look compatible.

Retain old source bundles, annotated tags, programs/images, configuration and
hashed assets as one recoverable unit. Roll back using the old declared source,
profile, parameters and program; an old image with a new declaration is not a
rollback. Preserve exact program-bound historical evidence and sensitive originals
in restricted storage. The withdrawn three-commit lineage documented in the
index is a restricted archival exception, not a candidate for remote tag push.
New source publication, artifact publication, deployment switch and onsite
acceptance require separate records and authorization.
