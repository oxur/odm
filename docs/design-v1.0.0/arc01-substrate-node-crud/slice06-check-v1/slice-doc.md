# Slice 06 (Arc 01) — `check` v1 + link-integrity (plan-of-record)

> Refs: ODD-0013 §7, ODD-0011 R6 (link-integrity), ODD-0001 A3 (renumber rot).
> `depends_on:` slice03 (schema/edges) + slice04 (load) + slice05 (CLI). This is
> `check` v1; Arc 02 slice06 extends it to `check` v2 (graph-level).

## Goal

The first mechanical gate: validate the corpus's *structure* — frontmatter
completeness, link-integrity (no dangling refs), and supersession-chain integrity —
with CI-grade exit codes and errors that name the fix. **Done when `odm check`
flags each of those defects on a broken corpus and passes a clean one.**

## Scope

**In:** `check` command (over the full-scan-loaded corpus): required-field
completeness per node type; **link-integrity** — every `part_of`/`supersedes`/edge
reference resolves to a real node id (no dangling refs); **supersession-chain
integrity** (chains are acyclic and terminate; no node supersedes itself); exit
codes (`0` clean, `1` violations, `2` usage); **errors-as-affordances** (each
finding names the exact command to fix it); `--json` report.

**Out:** graph-level checks — cycles-without-tears, out-of-order/staleness,
recomposition, below-threshold satisfaction — all land in `check` v2 (Arc 02
slice06), which *extends* this. Reconcile/drift is Arc A5.

## Verification

`cargo test -p odm-cli -p odm-core` + `assert_cmd` green; a deliberately-broken
corpus (missing field, dangling `part_of`, self-supersede) is flagged; a clean
corpus passes with exit `0`; clippy `-D warnings`; coverage ≥ 90%. Rows in
`ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI/local 1.85+). **Arc 01
complete** — the substrate (identity, schema, store, CRUD, structural check) is in
place; Arc 02 builds the graph on top.
