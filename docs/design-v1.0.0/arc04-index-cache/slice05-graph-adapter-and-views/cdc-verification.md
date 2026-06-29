# CDC Verification — Arc 04 / Slice 05 (partial): index→graph adapter + derived-order readers

> Independent verification of CC's closed ledger (impl `89a2223`; closed `2b287ee`),
> per LEDGER-DISCIPLINE v2.0 (slice scale, §A). slice05 was delivered **partial** — the
> adapter (the crux) + the derived-order readers — with `check`/`rollup`/`orient`
> deferred on a record-field gap. CDC reproduces structural rows here; cargo rows route
> to CI / local 1.85+.

## Environment constraint (disclosed)

No 1.85+ toolchain in the sandbox; cargo executions route to CI / CC's local 1.95.0 run
(held **attested pending CI**). CDC reproduces structural rows by reading the branch
`arc04-slice05-graph-adapter-and-views` at `89a2223`.

## Row dispositions

**Row count:** 7 opened, 7 addressed (4 `done`, 3 `deferred`). No silent drops. ✔

**Reproduced by CDC (structural, at `89a2223`):**

- **G-1 (the crux)** — `adapter.rs` `frontmatters_from_records(records, gates)` +
  `edges_from_record` reconstruct `Frontmatter`s from index records (the inverse of
  slice02's `map_edges` + a status rebuild from gates+evidence) and feed the **unchanged**
  `NodeGraph::build` / `Satisfaction::compute` — no frontmatter parse, no graph/
  satisfaction re-derivation, zero odm-core change. Fidelity asserted on ready frontier,
  per-node blocked (evidence-leveled), topo order, containment, over a corpus covering
  every edge kind. ✔
- **G-2** — `Derived::load` is index-backed (reconcile → adapter → graph); `next`/
  `blocked`/`path` ride it (`commands.rs` 1421/1484/1560). ✔
- **G-6** — reconcile-then-read (the `Derived::load` freshen). ✔
- **G-7 (no `unsafe`)** — grep empty. ✔

**Attested by CC (local rustc 1.95.0), pending CI:** full workspace → 266 passed; clippy
`-D warnings` → exit 0; `fmt` clean; line coverage **odm-index 98.19%** (adapter.rs
95.83%) **/ odm-cli 94.22%**. → **PENDING CI** (`attested → reproduced`).

**Deferred (valid):** G-3 (`check`), G-4 (`rollup`), G-5 (`orient`) — reason: they read
`fm.decomposed()` (recompose) and `fm.origin()` (rollup provenance), **not in the
record**; re-entry: enrich the record with those + wire the three. Confirmed the reads:
`rollup.rs:315/374` (`origin`), `recompose.rs:207/213` (`decomposed`). A clear,
re-enterable deferral.

## The finding — and my (CDC) planning miss

CC bubbled up the right finding: **wiring all consumers off the index is multiple
slices, because each consumer reads a different `Frontmatter` projection, so the record
grows per consumer** (slice04: `number`/`component` for `list`; this continuation:
`origin`/`decomposed` for `rollup`/`check`).

**I own the proximate miss.** The slice05 slice-doc/ledger specified the adapter's
reconstruction as *"id, number, type, name, edges from `EdgeRef`+qualifiers, status from
gates+evidence"* — and **overlooked `origin` and `decomposed`.** That under-specification
is mine, not CC's; CC caught it before half-wiring against absent fields (the right
call). The deeper miss is the arc-plan's original assumption (v1.6) that one enrichment
+ one slice would wire all consumers — the projection-per-consumer reality makes it more.

**Bounded now (verified):** I grepped every consumer's `fm.*` reads against the record.
The *complete* remaining gap is **exactly `origin` + `decomposed`** — no consumer reads
`reserved`/`retired`/`desired_facts`, and `orient`'s `.body()` is the targeted load (not
a record field). So the continuation enriches **once** (origin + decomposed) and wires
all three — there is no third surprise field. The index record is converging on "the
full frontmatter projection **minus the body**," which is consistent with 0014 §3.5
(structured metadata in; body out).

## Scoping the continuation (+ renumber)

Per Duncan's established scheme (clean sequential renumber + bridge note), the
continuation is **slice06** (enrich `origin`+`decomposed` → `FORMAT_VERSION 3`; the
`aggregate` refactor; wire `check`/`rollup`/`orient`); early-cutoff → **slice07**,
benchmark → **slice08**. The arc04 slice list is now **fully known (8 slices)** — this
is the last expected insert, so the renumber-bridges stop compounding after v1.9.

## Verdict

**Arc 04 / Slice 05 (partial) CDC-verified on structure; deferrals valid; cargo rows
pending CI.** The adapter — the arc's crux — is sound and faithful (graph == baseline
graph), and the derived-order readers are index-backed. The composed views + `check`
carry to slice06 on a bounded, verified enrichment (`origin`+`decomposed`). **A-5 stays
`open`** (partial); A-4 stays open until slice06 closes the consumer-wiring.

CDC: planning thread, 2026-06-29. Iterations used: 1.
