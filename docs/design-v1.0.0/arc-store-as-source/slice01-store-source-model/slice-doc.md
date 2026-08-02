# Slice 01 — ODD-0026: the store-as-source model

<!-- Name/title carries no document-role metadata, per ODD-0013 §2.1. -->

**Arc:** Store-as-source-of-truth & native authoring · **Kind:** design/decision
(the diff is the accepted ODD, not code) · **Opened:** 2026-08-02

## Goal

Decide and record the **store-as-source-of-truth model** as **ODD-0026**, so the
implementing slices (02 self-sourced nodes, 03 native authoring, 04 cutover) build
against a settled model instead of re-deriving it. The single sentence the ODD
must make true: *a planning node's body is authored and owned in the store, and odm
no longer depends on an external `./docs` source to hold or verify it.*

## Scope — in

- Resolve the four open design forks (A, B, D, E below) with a recorded decision +
  rationale for each. Fork C is already settled (the ODDs are migrated planning
  artifacts → store-as-source); record it, don't re-decide it.
- Author ODD-0026 and take it to **Accepted**, reconciled against ODD-0025
  (fidelity), ODD-0013 (identity/naming), ODD-0017 (projection-out).
- Derive and record the acceptance criteria that slices 02–04 will meet (so the
  arc ledger's SS-rows have concrete tests).

## Scope — out

- **No code.** Any change to `migrate`/`reconcile`/`check`/`node` is slice 02–03.
- **No deletion.** The `./docs` planning subtree stays until slice 04.
- **No authoring-command design detail** beyond the contract sketch fork E needs
  (the command surface is slice 03's to specify and build).

## The forks, with CDC recommendations (ratify or adjust)

**A — `source.paths` for planning nodes.**
Today every planning node carries `source.paths: [docs/…]` + `class: arc-plan|…`
and a body-hash gate against that file. Options: **(A1)** drop external source
entirely for authored nodes — the store file is the source; mark them (e.g.
`origin: authored`, or `source: { class: authored }`); or **(A2)** repoint
`source.paths` self-referentially at the store node's own file.
**Recommend A1.** ODD-0025 *already* models "a node with no external 1:1 source"
(the project synthesis node and retired nodes). A1 generalizes that one category;
A2 keeps a vacuous hash and a path that means nothing. Decide the exact marker in
the ODD.

**B — the body-hash fidelity gate.**
The gate (ODD-0025) proves a migrated body matches its external source. For a
self-sourced node there is no external source, so the check is vacuous. Options:
**(B1)** the gate applies only to nodes *with* an external source (migrated /
end-user / legacy) and is N/A for authored nodes; **(B2)** keep a self-checksum for
tamper detection.
**Recommend B1.** The gate's job is faithful *migration*; there is nothing to
migrate. The store lives on its own git branch — git history is the tamper record.
Schema/edge/decomposition/order validation still applies to authored nodes.

**C — the design corpus (ODDs 0011–0025). SETTLED — not a fork.**
The ODDs are design/planning artifacts already migrated into the store (repeatedly).
They are store-as-source like every other planning node; `docs/design/` deletes in
the cutover (slice 04) along with the plan tree. Recorded here so slice 04's
deletion scope is explicit — no decision required. (Earlier framing of this as an
open choice was a CDC error: "already in the store" settles it.)

**D — end-user `./docs`.**
After cutover `./docs` holds conventional end-user documentation. Is it tracked by
odm? Options: **(D1)** outside the odm planning store entirely (versioned with code
on `release/1.0.x`); **(D2)** a separate non-planning corpus odm is aware of.
**Recommend D1.** Freeing `./docs` from odm is the point. odm tracks *planning*.
(A future "docs coverage" feature, if ever wanted, is a separate arc.)

**E — the authoring/update contract (drives slice 03).**
How you create/update a planning node once the store is the source.
**Recommend:** `odm node new <type> <name> --from-file <md>` (you still write
markdown; odm ingests it as the node body and owns placement/frontmatter/ULID) —
plus `odm node edit <ref>` opening the store node in `$EDITOR`. "Hand-edit the
store `.md` + `odm check`" stays the always-works fallback. The human ergonomics
barely change (write markdown); the source of truth moves into the store. The ODD
records the contract; slice 03 builds it.

## Verification approach

This is a design slice, so "verify" means the decision is *recorded and coherent*,
not that code runs. Each fork has a written decision + rationale in ODD-0026; the
ODD is reconciled row-by-row against ODD-0025/0013/0017 with any tension named and
resolved; and slices 02–04's acceptance criteria are derivable from it (the arc
ledger's SS-2…SS-7 map onto ODD-0026 sections).

## Exit criteria

ODD-0026 is Accepted; forks A–E each have a recorded decision; the downstream
slices' acceptance criteria are written; and the arc-plan's SS-1 row can close
`attested` (Accepted ODD) with the rest of the SS-rows now concretely testable.
