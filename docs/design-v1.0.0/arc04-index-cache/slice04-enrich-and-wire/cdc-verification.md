# CDC Verification — Arc 04 / Slice 04 (seam a): Enrich record + wire `list`

> Independent verification of CC's closed ledger (impl `2dafaa1`; closed `2df674a`),
> per LEDGER-DISCIPLINE v2.0 (slice scale, §A). slice04 was delivered as **seam (a)** —
> enrich + maps + index-backed `list` — with seams **(b)** index→graph adapter + graph
> readers and **(c)** composed views **deferred to a renumbered continuation**. CDC
> reproduces structural rows here; cargo rows route to CI / local 1.85+.

## Environment constraint (disclosed)

No 1.85+ toolchain in the sandbox; cargo executions route to CI / CC's local 1.95.0 run
(held **attested pending CI**). CDC reproduces structural rows by reading the branch
`arc04-slice04-enrich-and-wire` at `2dafaa1`.

## Row dispositions

**Row count:** 10 opened, 10 addressed (7 `done`, 3 `deferred`). No silent drops. ✔
The deferrals (I-6/I-7/I-8) carry a reason (slice too large — the named split seam
fired) and a re-entry condition (the renumbered continuation slice) → **validly
deferred** per §A.

**Reproduced by CDC (structural, at `2dafaa1`) — seam (a):**

- **I-1** — `GateState { gate, evidence }`; `IndexRecord.gates: Vec<GateState>`
  (`record.rs`); `number: u32` + `component: Option<String>` added. ✔
- **I-2** — `FORMAT_VERSION: u16 = 2` (`snapshot.rs:40`); a forged v1 file →
  `RebuildNeeded(VersionMismatch)` → cold rebuild (slice01 self-heal; no migration
  code). ✔ **Discharges the slice02 FORMAT_VERSION freeze watch-item.**
- **I-3** — `build_one` populates per-gate evidence; `meta_hash` covers gate evidence
  (an evidence raise flips it) and excludes `updated`/stat **and** the new
  `number`/`component` (display/filter, not graph-semantic — the right exclusion). ✔
- **I-4** — `IndexMaps` (`maps.rs`): `by_type` / `by_tag` / `by_gate` + forward edge
  adjacency, built on load; no disk, no FTS. ✔
- **I-5 / I-9 (`list`)** — the `list` human table is index-backed via reconcile-then-read
  and matches the `load_all` baseline (filters narrow; order preserved); a node added
  after a first `list` shows on the next (freshness). ✔
- **I-10** — odm-cli → odm-index dep added; no `unsafe` (grep empty). ✔

**Attested by CC (local rustc 1.95.0), pending CI:** `cargo test` full workspace → 263
passed; clippy `-D warnings` → exit 0; `fmt --check` clean; line coverage **odm-index
98.53% / odm-cli 93.82%**. → **PENDING CI** (`attested → reproduced`).

## Bubble-up check (PM Part IV / LEDGER v2.0 §A)

- **Delivered its piece?** ✔ — seam (a) (enrich + maps + `list`), a coherent sub-unit;
  the split was recognised *before* half-landing (b)/(c) under budget — the right call.
- **Silent-drop diff honest?** ✔ — the deferred seams are disclosed with re-entry, not
  buried; CC refused to self-name a `04b` and requested the operator renumber.
- **Findings dispositioned + arc-plan updated?** ✔ — arc-plan **v1.7** records the
  seam-(a) bubble-up + the three findings. (A-4 stays `open` — correct; the slice isn't
  fully delivered until the continuation lands.)

## Rulings on CC's flagged decisions

1. **Enrichment added `number` + `component` (beyond evidence-only).** **Accepted** —
   `list`'s table needs `number`; its filters need `component`. Disclosed; and excluding
   them from `meta_hash` is correct (display/filter fields don't change graph meaning, so
   they must not invalidate downstream).
2. **`list --json` stays `load_all` (the §3.5 boundary).** **Ratified — accept the
   boundary.** The index is the filter/sort accelerator, **not** a full-node store; the
   `--json` full-node dump (`origin`/`reserved`/`retired`/…) is the whole `Document`,
   which the index deliberately does not mirror (same reason bodies stay out, §3.5).
   Growing the record to carry every field just for `--json` parity would defeat the
   point. *Optional future refinement (not required):* `list --json` could use the index
   to **filter** (which ids match) then targeted-load only those `Document`s — a speedup
   without growing the record. Recorded; not a slice04 obligation.
3. **Seam-(b) adapter sketch** (synthesize `Frontmatter`s from records → feed the
   existing `NodeGraph`/`Satisfaction`). **Noted** — a sound starting point; the record
   now carries the evidence it needs. *Design choice for the continuation slice:*
   synthesize a partial `Frontmatter` vs. give the graph/satisfaction builders an
   index-native input — settle it in that slice's doc, against which is the smaller,
   clearer seam. Not prescribed here.

## Open: the renumber (operator's call — see the question to Duncan)

CC correctly deferred naming the continuation. The two schemes (clean NN renumber with a
back-reference bridge note, vs. stable-ID + a `slice04-continuation` name) are being put
to the operator; whichever is chosen, the continuation slice (seams b+c) is the next
unit to draw up, and `A-4` closes only when it lands and the whole reproduces.

## Verdict

**Arc 04 / Slice 04 seam (a) CDC-verified on structure; deferrals valid; all flags
ruled (the `--json` §3.5 boundary ratified); cargo rows pending CI.** The record is
enriched (FORMAT_VERSION 2, freeze watch-item discharged), the in-memory maps exist, and
`list` reads the index. The graph/view wiring (the crux: the index→graph adapter) is the
continuation. **A-4 stays `open`** until that lands.

CDC: planning thread, 2026-06-29. Iterations used: 1.
