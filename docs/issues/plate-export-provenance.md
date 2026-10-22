# PLATE-EXPORT-1: verify the source used to export frozen graphs

Status: local exporter compatibility/provenance fix; no remote issue or release.
The parent task's Parret plan remains the only implementation progress entry.

## Failure and responsibility

At `4f72c0e5867ffce61c4f02ae7c1f0eaa4d4baed5`, the exporter verified delivery
bundle files against its wheel and manifest, then imported whichever `itn`
package Python resolved. A same-version shadow or modified package could produce
different graphs while the receipt still attributed them to frozen sources.
The oracle's existing imported-source hash check was missing from the exporter.
This is an added tool defect; it is not a rule change, a core FST executor bug,
or evidence that previously frozen assets are wrong.

## Minimal reproduction and fix

`python -m unittest discover -s tools -p test_export_plate_fsts.py` uses only
synthetic modules/graphs. Matching source bytes export; changed ITN or TN source
must fail before a normalizer is constructed or an output directory is created.
On the old exporter the two rejection tests fail; after reusing the oracle's
source validator they pass. The oracle uses the same validator before importing
its semantic functions. ITN and TN are separately resolved Python packages:
validate each package's own root, including a separately shadowed TN tree.
The additional split-root test fails the initial shared-root guard and passes
the corrected validator. No graph, rule, profile parameter or frozen asset changes.

Restricted red/green logs and original tools are preserved under
`parret-evaluation-evidence/itn-20261019/fallback-integration-p55/`. Actual export
and fixture gates are recorded separately; synthetic provenance tests alone do
not establish FST equivalence. The fix commit references this issue.
