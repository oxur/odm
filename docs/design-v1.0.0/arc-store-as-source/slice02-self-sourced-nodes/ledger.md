# Slice 02 ledger -- self-sourced planning nodes

Per `LEDGER-DISCIPLINE.md` sec. A. Code slice: rows reach `reproduced` where a test or the
live corpus demonstrates them; cargo/exec legs are `attested -> CI` (no macOS toolchain in the
CDC sandbox). Design-doc rows (amendments) top at `attested`/`reconciled`.

| ID | Criterion | Verify | Significance | Origin | Status |
|----|-----------|--------|--------------|--------|--------|
| F-1 | Schema adds `origin: authored` + the authored `source` shape (`class: authored`, no `paths`/`migrated_*`); it validates | fixture: an authored-shape node parses + validates; a malformed one is rejected | serious | ODD-0026 sec. 2.1 | open |
| F-2 | `check` passes an authored node (no `undeveloped-stub` / `missing-source` / body-hash error) | fixture: authored node, `check` exit 0 | serious | arc SS-2 | open |
| F-3 | Schema/edge/cycle/decomposition/derived-order validation **still applies** to authored nodes | fixture: authored node with a bad `part_of` / cycle / drift still fails `check` | serious | arc SS-2 | open |
| F-4 | `migrate --all` does not require or rewrite an external source for authored nodes; their bodies are left byte-intact (not churned) | fixture + live: authored node body unchanged across `migrate --all` | serious | arc SS-3 | open |
| F-5 | `reconcile` treats authored nodes as self-sourced -- no re-fidelity against a missing source, no spurious drift | fixture: authored node, `reconcile` reports 0 drift | serious | ODD-0026 sec. 2.2 | open |
| F-6 | Existing planning corpus converted to self-sourced per the sub-decision; `migrate --all` + `check` green with `./docs` still present | live: reset -> `migrate --all` -> `check` exit 0 on the real store | serious | arc SS-3 | open |
| F-7 | **No regression:** the body-hash gate STILL fails a genuinely-migrated node (external `source.paths`) whose body diverges from its source | fixture: migrated node + corrupted body -> `check` fails | correctness | arc SS-5 | open |
| F-8 | **ODD-0013 amendment** written: `origin: authored`; authored `source` shape; "authored is a provenance value" language; author-vs-odm field boundary | amendment doc present + accepted | serious | ODD-0026 sec. 3 | open |
| F-9 | **ODD-0025 amendment** written: authored nodes bypass the migration body-hash gate by construction; migrated fidelity unchanged | amendment doc present + accepted | serious | ODD-0026 sec. 3 | open |
| F-10 | The sub-decision (fate of converted nodes' provenance) is resolved + recorded in the ODD-0013 amendment before conversion runs | amendment states (i) or (ii) with rationale | serious | slice-doc | open |
| F-11 | `make format`/`lint`/`test` green; no new clippy warnings | CC attests; CI reproduces | correctness | house | open |

## What Worked

_(At slice close.)_

## Closure

_(At slice close: per-row walk + bubble-up to the arc in `closing-report.md`; CDC re-run in
`cdc-verification.md`. F-6 is the live acceptance anchor -- it rides the operator's next
`migrate --all` on the rebuilt binary if the CDC sandbox cannot run it.)_

Closed at commit <SHA> on <date>. Verified by: <name/session>.
Rows: 11. Done: _. Deferred: _. No-op: _.
