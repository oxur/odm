# Slice 03 (Arc 01) — Frontmatter schema + round-trip (plan-of-record)

> Refs: ODD-0013 §2.3 (the normative schema), §3 (edges). `depends_on:` slice02
> (the identity types). This slice picks the YAML library (deferred from slice01).

## Goal

Define the on-disk node format and make it **round-trip-stable**: parse a
`--- YAML --- ` frontmatter block + markdown body into the typed model, and emit it
back in a canonical field order such that **`parse ∘ emit = identity`**. Forward-
compatible: keys this slice doesn't yet model are *preserved*, not dropped.

## The YAML library decision (deferred from slice01)

`serde_yaml` is archived/unmaintained (cf. ODD-0014's caution on archived crates).
Decision criteria: maintained, pure-Rust, serde-integrated, drop-in for the
`serde_yaml` API. **Use `serde_yaml_ng`** (a maintained drop-in fork) unless CC's
maintenance check (most recent release within ~12 months) finds it stale — then
fall back to `serde_norway`. **Either way, isolate it behind a single
`frontmatter` module** so no YAML-crate type appears in our public API and the lib
is swappable later (the same insurance we applied to `ulid`'s error type in
slice02). Add the chosen crate to `[workspace.dependencies]`.

## Scope

**In:** a `frontmatter` module (parse `---`-delimited YAML + body; emit canonical);
the schema covering id, number, type, name, created, updated, `origin`, `reserved`,
`tags`, `component`, and the **edges block** (`part_of`, `depends_on` [optional
`satisfied_at`], `blocked_by`, `consumes`, `verifies`, `supersedes` `{node, kind:
obsoletes|updates}`, `affects`, `tears`); **unknown-key preservation** across
round-trip; canonical field-order emission (per §2.3); typed parse errors with
position where feasible.

**Out:** the *status* vector + *desired_facts* (added to the schema by their owning
slices — arc02 slice03 and Arc A5 — and preserved as unknown keys until then); edge
*semantics*/graph (Arc 02); persistence/store layout (slice04); link-integrity
(slice06).

## Verification

`cargo test -p odm-core` (or wherever `frontmatter` lives) green; **proptest
`parse ∘ emit = identity`** incl. unknown-key preservation; chosen YAML lib isolated
(grep); clippy `-D warnings`; coverage ≥ 90%. Rows in `ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI/local 1.85+). Then slice04
(store layer) persists these to `nodes/YYYY/MM/<ULID>.md`.
