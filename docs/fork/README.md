# Owner fork: Parret consumption snapshots

This directory owns fork-specific version and maintenance documentation.
The upstream project is [SpenserCai/wetext-rs](https://github.com/SpenserCai/wetext-rs),
a Rust implementation of [pengzhendong/wetext](https://github.com/pengzhendong/wetext).
The [owner fork](https://github.com/MonadKai/wetext-rs) owns explicit Chinese
compiled ITN, plate rules, validation, original UTF-8 spans and full-text rendering.
Parret owns its adapter, execution bounds, final lifecycle and public ASR contract.
See [maintenance and inherited profile details](MAINTAINING.md), the
[v1.5 profile](../../profiles/number_plate_v1_5/README.md) and
[plate.4 default profile](../../profiles/number_plate_v1_6/README.md).

## Remote source freezes

These annotated tags freeze the **actual consumer-pinned commits**. Tag objects
have their own identities; always verify the peeled commit as well. Development
branches and squash-merge commits are separate records.

| Remote annotated tag | Complete source SHA | Supported consumer profiles | Known Parret consumers |
|---|---|---|---|
| [parret/itn-source-6de7f3b](https://github.com/MonadKai/wetext-rs/tree/parret/itn-source-6de7f3b) | `6de7f3b7d91899c591f844c61e558d5eb84080a4` | Chinese base and all v1.5 plate profiles below | [PR #2531](https://github.com/MonadKai/parret/pull/2531), clean vendor at `bc92ad890e3c03dbbb36cb798c82a058a3fd1f17`; historical LPR source/image/rollback; plate.4 audio comparison main at `b2b1db996e0f33f51d8c3edec9198a1051bda6f0` |
| [parret/itn-source-2b2edc2](https://github.com/MonadKai/wetext-rs/tree/parret/itn-source-2b2edc2) | `2b2edc2f57dc0ea385a4b03512a2c2919876f1d3` | Retains base/v1.5; adds default plate.4 full text | [PR #2753](https://github.com/MonadKai/parret/pull/2753), native audio program source `2a559ae39aefc05b9751dfcd7e54ada6e7c324b9`, reviewed delivery source `03dbab92a5ee12fb434b057f009828977779f585` |

The prerequisite `6de7f3b` was merged by [fork PR #2](https://github.com/MonadKai/wetext-rs/pull/2).
[Fork PR #3](https://github.com/MonadKai/wetext-rs/pull/3) merged plate.4 as
`cd4a3cc83957bd302f77364e740557752c76fe24`; its tree equals `2b2edc2`, but the
Parret vendor source and source tag remain **2b2edc2**. Documentation commits
and later merge identities must not be substituted for an evaluated program.
The development branch `feat/plate4-default-itn-20261109` remains a development
reference, not the durable consumption identity.

## Profile, reference and asset mapping

One source snapshot can support several independently selected profiles.
The consumer names below are Parret Rust profile identities, not Python wheel
or Rust package release versions. Source tags do not publish a crate or image.

| Consumer profile | Rule identity / reference | Rendering / assets |
|---|---|---|
| `wetext.zh.itn@1.2.0+plate.1.base.cached.1` | Base Chinese grammar; delivered `1.2.0+plate.1` export, plate semantics disabled | Explicit Chinese, cached full-request graph; tagger + verbalizer |
| `number_plate@1.5.0-structured.1` | `plate_semantic_v1_5`, frozen `1.2.0+plate.1` oracle | Structured accepted plan, unrelated text preserved; lexicon only |
| `number_plate@1.5.0-full.1` | Same v1.5 rules and reference | Legacy per-fragment `full_text` rendering; lexicon + both FSTs; known text differences retained |
| `number_plate@1.5.0-full.2` | Same v1.5 rules and reference | `full_text_request_graph_v1`; lexicon + both FSTs |
| `number_plate@1.6.0-full.1` (2b2edc2 only) | `plate_semantic_v1_6`; `1.2.0+plate.4`, release `plate4-badcase-guarded-20261109`, upstream source `caf4a6a6fdf81bcba2c7c9c60047f9382ce8ea50` | Complete request rendering; **no** `plate_context` argument, including no empty object; same three assets |

Frozen runtime asset SHA256 values, verified in the consumer manifests:

| File | SHA256 |
|---|---|
| `zh_itn_tagger.fst` | `a0935a3eeebbdf9cf0a3f33e81b5f383af598a7eca23ed134594cd8dd8dd00bd` |
| `zh_itn_verbalizer.fst` | `e5f3fd84ad76f0a430295c178431a3b603295d93d3c883f51e9f5189377cfdee` |
| `lexicon.json` | `e8eeee6c7ddb651e6708a5dc49ceb688addfc2a612f50c015ad390e2142c15c9` |

Base graph parameters: `reload_then_optimize`, `remove_interjections=true`,
`enable_standalone_number=true`, `enable_0_to_9=false`, `enable_million=false`,
`enable_plate_semantics=false`. The separate wheel oracle enables plate semantics.
Plate profiles use Unicode `15.0.0`, policy SHA256
`462082acde24c0271ddcb6ec2379c41c7f1902f340461b1df24d3b4d41ee2003`, and an
explicit-kind nesting limit of 64. Exact oracle options, wheel file hashes,
Python dependencies and cache behavior live in each profile's oracle manifest.
The plate.4 wheel SHA256 is
`e9b46980d7f557a63257f749090418044175197b9afde6cc8998d7b6e0614513`.
Identical lexicon bytes do not imply identical parsing rules.

The caller still owns worker/queue/input/output/deadline limits and original-text
fallback. Public LPR output remains one `text`: successful normalized text,
or original ASR text on fallback; there is no simultaneous raw-final channel.
Each original normal final is processed independently. No business context is
inferred. Cross-final completion/correction/re-report belongs to the client.
The same-final correction followed by “圈出来” retains the fixed wheel's
conservative rejection; this is not a cross-final restriction.

## Historical consumption and withdrawn candidates

The inventory covers all distinct owner SHAs found in available Parret vendor
`SOURCE.json` history, adapter dependency declarations, tracked qualification
receipts, source/deployment locks and the retained LPR rollback records.
The adapter originally used a mutable sibling path, then a generated path
dependency; `Cargo.lock` records the package version but no Git source SHA for
that path. Such a lock alone cannot establish an earlier consumed commit.

| Historical source | Evidence and classification | Frozen-reference disposition |
|---|---|---|
| `4c396e8125f0b6426d115d9100115028b656159a` | Actually executed legacy full.1 local service/replays; Parret `5cd24f99edeaa10bc15ed82259d17f53e783dcc1` vendor; 2026-10-19 functional/full-audio receipts | Withdrawn lineage; existing local annotated `plate-v1.5.0-full.1` and restricted bundle, **not remotely republished** |
| `47cc2536ee9e31c39fdcfb69bf0f0f900b02c8a3` | Actually executed full.2 historical service: Parret `b36e93282860de3b398b0be0bc954b30a78e0153`, binary `cb0966d4219180421df88bd7b21b449ce169d967fbd7feae15248fb034fb8dfd`; vendor introduced at `3ca50367f5c9c738ca542489c2789ed178957eb8` | Same restricted archival exception; no new remote tag |
| `af03ed037e1b3c65940f77c21abfe08dacbaa4a2` | Intermediate local integration/vendor candidate at Parret `29a295f78f5174bcfa813a9ee641316a4e0b50b8`; do not infer an independently qualified served binary from this declaration | Same restricted archival exception; superseded by clean 6de7f3b, without relabeling its SHA |

The old chain contained source-derived fixtures and was withdrawn in fork PR #2.
The owner explicitly accepted restricted archival of these three original SHAs
instead of remote tags on 2026-11-10. A tag pointing to an original commit also
retains its ancestors: removing fixtures in a later commit does not make the
old tag safe to publish. The sanitized 6de7f3b is a new source identity, not an
alias for any withdrawn commit. Never copy restricted originals into this index,
Git, PRs or tag messages.

Other fork development/export/documentation commits (for example `7bd8312`,
`4f72c0e`, `feb74ee`) are not independently established consumer snapshots by
the surveyed declarations; no delivery tag is invented for them. The public
source-preparation candidate `25017a4` and merge `cd4a3cc` are integration
identities, separately recorded, not the actual Parret source pin.

Consumer evidence: [historical basic delivery](https://github.com/MonadKai/parret/blob/03dbab92a5ee12fb434b057f009828977779f585/docs/experiments/2026-10-21-number-plate-basic-delivery.md),
[historical full.2 qualification](https://github.com/MonadKai/parret/blob/03dbab92a5ee12fb434b057f009828977779f585/docs/experiments/2026-10-21-number-plate-final-full2-results.md),
[plate.4 qualification](https://github.com/MonadKai/parret/blob/03dbab92a5ee12fb434b057f009828977779f585/docs/experiments/2026-11-10-lpr-plate4-complete-qualification.md),
and [plate.4 deployment handoff](https://github.com/MonadKai/parret/blob/03dbab92a5ee12fb434b057f009828977779f585/crates/parret-asr-stream-server/deploy/lpr/PLATE4-HANDOFF.md).
All are revision-scoped evidence, not production acceptance. New API image
build/publication, pre deployment and onsite validation remain separate pending
operations; neither source tags nor a source merge supply an image digest.
