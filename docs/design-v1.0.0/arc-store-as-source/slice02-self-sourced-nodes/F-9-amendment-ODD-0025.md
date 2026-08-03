# Amendment stub — ODD-0025 (Migration Fidelity) for arc-store-as-source slice02

> **Draft the amendment before the cc-prompt closes** (slice-doc §Scope-in item 6). This stub
> specifies the exact changes to fold into `docs/design/04-accepted/0025-migration-fidelity-model.md`,
> plus the version-history entry to add. **Surfaced by:** ODD-0026 §2.2 (fork B) and §3 (the
> accepted design basis). **Status:** APPLIED 2026-08-03 -- folded into the accepted ODD (v1.4); originally proposed — implemented in code (the body-hash gate
> [`odm_migrate::fidelity::verify_body_hash`] is called only from the migration/reconcile paths,
> which now skip an `origin: authored` node by construction — see `selfhost::self_host`/`repair`/
> `reconcile`'s authored guards); this stub is the model-doc half.

## What changes

### §2.0 Three provenance-adjacent axes, kept distinct — add the authored *point*, not a fourth axis

Current (§2.0) closes: *"`provenance` (0013, unchanged) — derived git + supersede + gate lineage;
never stored."*

Add, as a new paragraph immediately after:

> **An authored node (arc-store-as-source, ODD-0026) is a new *point* in this space, not a fourth
> axis.** It is characterized by `origin: authored` (a new value on the existing `origin` axis) and a
> `source` record whose shape carries no external path (`class: authored`). The three axes —
> `origin`, `source`, `provenance` — are unchanged in kind; `authored` is simply the value `origin`
> and `source` take together for a node that was never pulled from an external file. See the
> ODD-0013 amendment (arc-store-as-source slice02, F-8) for the exact schema shape.

### §2.1 Migration is strictly 1:1 and verbatim, hard-gated — narrow the gate's scope explicitly

Current (§2.1) opens: *"A migrated node's body is its source body… Migration computes
`sha256(normalize(source_body))` and `sha256(normalize(node_body))` and hard-fails on mismatch. The
gate is a migration-time gate only…"*

Add, as a new paragraph after the normalization note:

> **Scope clarified (arc-store-as-source slice02, ODD-0026 §2.2): the gate applies only to a node
> with an external `source` to verify against.** An authored node (`origin: authored`) has no
> migration event and no external body to prove faithfulness against — there is nothing to hash and
> nothing to gate, **by construction**, not by a special-cased exemption. Concretely,
> [`verify_body_hash`] is only ever called from the migration/reconcile code paths
> (`self_host`/`repair`/`mapping::backfill_source`/`mapping::reconcile_source`), and every one of
> those paths now skips a node whose `origin` is `authored` before it would reach the gate — the
> same way they already skip a synthesis or retired node. **No-regression clause (load-bearing):**
> for a node that **does** carry a migration `source`, the gate is entirely unchanged — a drifted
> non-stub body still hard-fails exactly as before (arc-migration-fidelity s05 F-5; regression-
> fixtured at arc-store-as-source slice02, F-7). Fork B *narrows* the gate's applicability, it does
> not weaken it for the nodes it still applies to.

### §2.2 The `source` record — cross-reference the authored shape

Current (§2.2) opens: *"Every migrated node carries a `source` sub-map, computed at migration
time: …"*

Add, immediately after the example YAML block:

> **An authored node also carries a `source` sub-map** (ODD-0026 §2.1 — provenance is kept, never
> dropped for having nothing to migrate), but a structurally different one: `class: authored`, no
> `paths`, no `normalization`/`migrated_by`/`migrated_on` (there was no migration to record). An
> optional `migrated_from` marker preserves a converted node's former `paths` without re-verifying
> against them (arc-store-as-source slice02 sub-decision (i)). See the ODD-0013 amendment
> (arc-store-as-source slice02, F-8) for the full shape; this ODD's own concern — migration
> fidelity — simply does not apply to it.

## No changes needed

- **§2.3 Synthesis**, **§2.4 Frontmatter fidelity check**, **§2.5 `artifact` node type**, **§2.6
  Report self-coverage**, **§2.7 Optional containment**, **§2.8 Repair is update-in-place**: all
  concern migrated/synthesized content specifically and are unaffected — an authored node is never a
  synthesis, never repaired (it is never sourceless), and never subject to a document-node
  containment question this ODD didn't already answer.
- **§2.9 Reconcile re-establishes fidelity to a changed source**: the *policy* (force-resnapshot
  against a living plan node) is unchanged for migrated content. The one addition, covered in §2.1
  above, is that `reconcile` now skips an authored node before reaching that policy at all — it is
  self-sourced, so there is no external "changed source" to re-establish fidelity to.

## Version-history entry to add to ODD-0025

```
### vX.Y — 2026-08-03 — arc-store-as-source slice02: the gate is migration-scoped, not removed

Clarifies that the body-hash fidelity gate (§2.1) applies only to a node with an external `source`
to verify against — an authored node (ODD-0026, `origin: authored`) has no migration event and
nothing to hash, by construction. `origin`/`source`/`provenance` (§2.0) stay three axes; "authored"
is a new point in that space (an `origin` value + a source-less `source` shape), not a fourth axis.
No-regression clause: the gate is entirely unchanged for genuinely-migrated content — a drifted
non-stub body still hard-fails (regression-fixtured at arc-store-as-source slice02, F-7). Surfaced
by: ODD-0026 §2.2 (fork B) + §3 (design basis), implemented arc-store-as-source slice02.
```
