# Slice 03 (Arc 02) — Gates, status & evidence recording (plan-of-record)

> Refs: ODD-0013 §5.1 (gate-sets) + §4.4 (evidence levels), §2.3 (status schema).
> ODD-0001 D2/D3. `depends_on:` arc01 (the node + status schema field).

## Goal

Make status a configurable, multi-gate, evidence-tagged vector — and the operation
that advances it. **Done when per-type gate-sets load from `odm.toml`, `set-gate`
records `{reached, by, evidence}`, the `Evidence` level is a total order, and an
out-of-set gate is rejected.** This is the *recording* half of evidence-leveled
satisfaction; slice04 is the *consuming* half.

## Scope

**In:** the `Evidence` enum `asserted < attested < reproduced < reconciled` with a
**total order** (the canonical definition lives here; slice04 consumes it);
per-node-type gate-set config in `odm.toml` (`[gates.<type>] sequence = [...]`);
the status vector ops (`set-gate <node> <gate> --by --evidence`); validation (gate
must belong to the type's set); the **terminal gate** accessor (used by default
satisfaction in slice04).

**Out:** satisfaction / min-propagation / threshold / surfacing (slice04); status
*serialization* (already in the arc01 frontmatter schema — this slice operates on it).

## Verification

`cargo test -p odm-core` green; clippy `-D warnings`; coverage ≥ 90%. Rows in
`ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI/local 1.85+). slice04
(derived order & satisfaction) consumes the `Evidence` type + terminal-gate.
