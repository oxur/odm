# Slice 13 (Migration Fidelity) — CDC verification: Live reconcile + vision mint

> **Arc:** Migration Fidelity · **Slice:** 13 · **Verifier:** CDC (independent) · **Date:** 2026-07-31 ·
> **Method:** LEDGER-DISCIPLINE v2.0 §A + PM Part IV. **Class-(b) live slice** — the committed `odm`
> store *is* the evidence; CDC reproduces store state by direct git read + independent recomputation
> (reproduce-don't-attest). Runtime rows (`cargo`/`clippy`/`llvm-cov`/`check`/`orient`/`rollup` execution)
> are attested-by-CC → CI — the store's binaries are macOS, this verifier reads the store from a Linux
> bridge and cannot execute them; every *store-state* claim below is reproduced, every *execution* claim
> is routed to CI.
> **Under review:** `slice13-live-reconcile/ledger.md` (F-1…F-10 + the Post-close correction),
> `closing-report.md`; the `odm`-branch chain `2fc25f5` (known-good) → `26bea1d` (the s13 fire CC closed
> against) → `e1e94bf` (the `# Vision` heading/`wrong-type-field` fix) → **`e06fffe` (HEAD — the shipped
> store)**; `release/1.0.x` wiring + ODD-0020 v1.4. **I verified against the true HEAD `e06fffe`, not the
> `26bea1d` CC's ledger closed against** — see §1.

## Verdict

**PASS — CDC-verified.** The shipped `odm` corpus is faithful and the modeled vision is live. Reproduced by
direct read: the store is at `e06fffe` (387 nodes); every source-bearing node is byte-faithful to its
current source **except two by-design living-plan-tail nodes** (the active `arc-plan.md` arc node and
slice10's `ledger.md` — both drifted *because their sources were edited after the reconcile fired*, exactly
the §2.9 case, invisible to `check` by design); the vision is a byte-identical 1:1 `project-plan` node
(`#1001`) superseded by an attested editorial-merge synthesis (`#1000`); and the doc-coverage gap CC
disclosed at close as *open* was in fact **closed** before ship by the operator-directed `migrate --all`
run baked into `e06fffe`. The ODD-0020 §2 vs ODD-0025 §2.3 conflict the operator surfaced is resolved by a
sound, narrowly-scoped carve-out (ODD-0020 v1.4).

This was the arc's messiest close, and the record is honest about it: CC self-caught a schema-drop bug
pre-commit, disclosed a `check`-error undercount (operator-surfaced), and fixed a latent `wrong-type-field`
regression same-day. **I add a fourth disclosure of my own** (§6) — my first drift scan was wrong and
raised a false alarm; the number below is the corrected one. The one thing PASS does **not** license is
declaring MF-9/P-12 done: the two living-plan-tail drifts must be closed by the **arc-close
final-reconcile-and-freeze**, once the plan docs stop moving. That is a routed condition, not a defect.

## 1. Which store I verified — reproduced (a scope correction)

CC's `ledger.md` Closure and F-1…F-8 evidence are written against `odm@26bea1d` (369→ the s13 fire, "373
nodes"). **That is not the shipped HEAD.** Two operator-directed commits landed after CC closed:

| Commit | What it did | Reproduced |
|--------|-------------|------------|
| `26bea1d` | s13 fire CC closed against: reconciles + vision mint | (superseded) |
| `e1e94bf` | Refresh `#1000`'s body with its `# Vision` heading; the `wrong-type-field` fix | body heading present |
| **`e06fffe`** | **`migrate --all`: mint the artifact/note family gap + reconcile living-plan drift** | **HEAD; 387 nodes** |

`git --git-dir=…/worktrees/odm diff --name-status 2fc25f5..e06fffe` = **18 A + 8 M** (26 files). The 18
creates fully account: **14** artifact-family nodes (the `--all` gap mint), **1** project node `#1001` (the
1:1 vision node), **3** slice plan nodes (s11/s12/s13's own). **No re-mint**, no id churn. Node count
369 → 387. This is the store I reproduced every claim below against. The arc-plan **header** already
reflects `e06fffe` (v2.24); only the s13 **verdict** was pending — this document supplies it.

## 2. Corpus fidelity — reproduced (383 faithful / 2 living-tail / 1 synthesis / 1 sourceless)

Independent recomputation of the §2.1 body gate against **current** sources (I re-ran my own scan; per
§2.1 a frontmatter-bearing source's node body = the source body *after the fence*, whole-file for the
frontmatter-less plan corpus — see §6 for why this detail bit me):

```
faithful=383  drifted=2  moved=0  synthesis=1  sourceless=1  total=387
DRIFTED: #58837400  docs/…/arc-migration-fidelity/arc-plan.md      (node 900 vs source 926 lines; source newer)
         #509907700 docs/…/slice10-coverage-live-run/ledger.md     (node  33 vs source  88 lines; source newer)
SYNTH:   #1000   SOURCELESS: #1605 (the retired A6·slice05 UAT tombstone)
```

**Both drifts are the living-plan tail, not corruption.** In each the *source is newer than the node* —
the doc kept being edited after the `e06fffe` reconcile snapshotted it. `arc-plan.md` especially is
self-referential: maintaining this very arc-plan (flipping slice statuses, this v2.25 entry) re-drifts its
own arc node `#58837400` the instant it's saved. slice10's `ledger.md` grew its *What Worked*/*Closure*
prose after reconcile. This is precisely what ODD-0025 §2.9 predicts and licenses: inter-reconcile drift on
a still-living plan doc is **by-design invisible to `check`** (the gate is migration-time-only), and the
arc-close ends with a **final reconcile-and-freeze** once the docs stop moving. `moved=0` confirms
ODD-0017/0018's `04-accepted/`-relative `source.paths` all resolve. `#1605` is sourceless-by-design (a
retired tombstone) and is coverage-satisfied structurally (§4).

## 3. The vision mint — reproduced (byte-1:1 + attested synthesis)

- **`#1001` (the 1:1 `project-plan` node)** — `id 01KYSX4RGCZT9H1N83TJFX8XFB`, `class: project-plan`, **no
  `synthesis` field**. Its body is **byte-identical** to `project-plan.md`: normalized SHA-256
  `ae2dcfe895a9…` on both sides, node-body == source (True). Hard-gate-clean, as MF-7's 1:1 requirement
  demands.
- **`#1000` (the re-cast project node)** — `type: project`, `schema: project/v1.1`,
  `source.class: vision`, **`source.synthesis: editorial-merge`**, `source.attestation` present ("distills
  project-plan.md's Definition-of-done section verbatim"), `edges.supersedes → 01KYSX4RGCZT9H1N83TJFX8XFB`
  (kind `updates`) — i.e. exactly `#1001`. Body opens with the `# Vision` heading (the `e1e94bf` fix).
  Lineage is single-target and clean.

The supersede-based synthesis model (s11) and the promoted `apply_project_vision` path (s12) landed live
exactly as specified: a faithful 1:1 node preserved, the synthesis carrying attestation + lineage rather
than a hash gate. MF-7 is done on the live store.

## 4. Doc-coverage — reproduced (the disclosed gap is CLOSED, not open)

CC's F-7 closed *disclosing a deviation*: `check` at exit 1 with 14 `uncovered-doc` errors for
s11/12/13's own artifact-family docs, "left for the arc-close (or a follow-up `--artifacts` run)." **The
operator did not wait for the arc-close** — the `migrate --all` run in `e06fffe` minted those covering
nodes. Reproduced set-difference over `scan_root="docs"` at `e06fffe`:

```
total .md under docs        = 388
covered by a node source.paths = 385
not primary-covered         = 3  → docs/design/index.md, docs/design/templates/design-doc-template.md
                                   (conventionally excluded), + the retired A6·slice05 UAT slice-doc
                                   (structurally fallback-covered by slice number via #1605)
mig-fidelity slice1x artifact docs = 20, of which uncovered = 0
```

So the 14-error gap is gone: **0 of the 20 arc-fidelity slice1x docs are uncovered**, 385/386 scanned
docs primary-covered, the one residual being the retired tombstone's slice-doc (fallback-covered — benign).
The exact `check` exit code is attested→CI, but the reproduced set-difference shows **no genuine uncovered
design/plan doc remains**. This is a materially *better* state than the ledger's Closure describes, because
the ledger closed one commit too early.

## 5. The ODD-0020 §2 / ODD-0025 §2.3 conflict — carve-out ratified

The operator caught (post-CC-close) that `#1000`'s `supersedes` edge is, under ODD-0020 §2's work/document
field split, an invalid field on a `project`-type node — a latent conflict with ODD-0025 §2.3's
synthesis-node model that nothing before s13 had exercised through `check`. Reproduced the resolution:
ODD-0020 **v1.4** amends `check_field_validity` to exempt a work node that **carries `source.synthesis`**,
keyed on the `source.synthesis` field being present, **not** on `node_type == Project`. I concur this is
the right cut: a work node re-cast as a synthesis is structurally a document-lineage record wearing a work
node's type, so the exemption belongs to the *synthesis* property, and keying it there (not on the project
type) keeps it general for any future work-type synthesis — amend-not-work-around, honored. No live data
changed to resolve it.

## 6. My own verification error — disclosed

Self-knowledge before self-assertion: **my first drift scan was wrong.** It compared each node body to the
**whole** source file *including the source's own frontmatter fence*, which made every frontmatter-bearing
design/research doc look drifted — a false alarm of "≈21 nodes drifted, corpus not faithful." Per ODD-0025
§2.1 a frontmatter doc's node body is the source body **after** the fence; the corrected scan strips source
frontmatter for frontmatter docs (whole-file only for the frontmatter-less plan corpus). Corrected, the
true count is the **2** living-tail drifts in §2. I caught it by sanity-checking a flagged "drift" by eye
and seeing the only difference was the YAML header. The number in this verdict is the corrected one; I flag
the error so the operator knows where my scans can go wrong (source-shape assumptions) and can check there.

## 7. The rocky close — CC's disclosures, independently confirmed sound

Three CC/operator disclosures, all reproduced as honestly-handled, none a hidden failure:

1. **F-6 schema-drop, self-caught pre-commit.** `build_synthesis` never stamped `schema`, so the first
   fire silently dropped `#1000`'s `schema` until a later pass re-added it — a collateral mutation. CC's
   own idempotence re-run caught it; store `git reset --hard 2fc25f5`, fixed, re-fired clean. The revert
   protocol worked as designed. (Reproduced: `#1000`/`#1001` both carry `schema: project/v1.1` at HEAD.)
2. **F-7 undercount, operator-surfaced.** CC's closing evidence recorded "14" `check` errors when the true
   count was 15 — the missing one *was* an s13 regression (`wrong-type-field` on `#1000`), not pre-existing.
   Disclosed in the ledger's *Post-close correction*, fixed same-day (→ ODD-0020 v1.4, §5). CC's
   empirical "pre-existing vs regression" method (a scratch worktree at the pre-fire SHA) was right in
   spirit; it simply undercounted by one, and the correction says so plainly.
3. **F-10 file-aggregate coverage.** `migrate.rs`'s file-level coverage is under 90%, dominated by
   pre-existing untouched `replan`/`coverage` functions; the *new* wiring is fixture-exercised. Disclosed,
   scoped-out correctly (thin-wiring-only), routed. Attested→CI.

## 8. Ledger — CDC disposition

F-1 (pre-flight/revert anchor) reproduced — known-good `2fc25f5`, before-manifest recorded. F-2 (dry-run
adjudication) attested→CI; its *outcome* is reproduced in the committed delta (§1: only re-snapshots +
moved-path rewrites + the one vision create, no id/schema/edge churn). F-3/F-4 (reconcile + moved-source
re-discovery) **reproduced** — 0 genuine drift remains modulo the living tail, `moved=0`. F-5 (vision)
**reproduced** byte-for-byte (§3). F-6 (no collateral/idempotent) — the schema-drop was the collateral,
caught and cleaned (§7.1); execution attested→CI. F-7 (coverage) **reproduced closed** at `e06fffe` (§4) —
better than the disclosed-open state it closed against. F-8 (one revertible commit) — reproduced as the
`2fc25f5`-based chain; note the shipped store is 3 commits deep, not the single commit F-8's letter
describes, because of the two operator-directed follow-ups (disclosed, §1). F-9 (no model drift) — the 1:1
rule + migration-time-only gate are intact; §2.9 and the ODD-0020 v1.4 carve-out are the cited model
touches. F-10 attested→CI (§7.3). **10 rows; no silent drops** — every "Out" item (arc-close/MF-9/P-12,
L-8a) confirmed untouched.

## 9. Findings

- **CDC-F1 (routed, not a defect) — the living-plan tail must be frozen at arc-close.** Two nodes
  (`#58837400` arc-plan, `#509907700` slice10 ledger) are drifted because their sources are still being
  edited. This is §2.9-sanctioned and `check`-invisible, but it means the shipped corpus is **not yet**
  byte-faithful end-to-end. **The arc-close MUST run a final reconcile-and-freeze once the plan docs stop
  moving, and MF-9/P-12 must not be claimed before it.** (Editing this arc-plan for the s13 flip *adds* to
  this tail — expected.)
- **CDC-F2 (LOW, doc hygiene) — the ledger Closure lags HEAD by two commits.** `ledger.md`/`closing-report.md`
  close against `26bea1d`; the shipped store is `e06fffe`. The arc-plan header is already current; the slice
  docs are not. Not worth a re-close, but the arc-close `closing-report.md` should narrate the true shipped
  chain (`26bea1d`→`e1e94bf`→`e06fffe`) so the record isn't read as ending at `26bea1d`.
- **Observation — the operator's `--all` composition is a keeper.** Folding self-host + design/research
  reconcile + `--artifacts` + `--notes` + `--vision` into one idempotent pass (prompted by "why does
  nothing compose the five derivations?") is exactly the general migration capability MF's Capability
  statement asks for. Worth carrying into L-8a.

## 10. Bubble-up check (PM Part IV)

- **Did s13 deliver?** Yes — the live corpus is faithful (modulo the routed living tail), the vision is
  minted 1:1 + attested-synthesis, the coverage gate is green, and the ODD-0020 conflict is resolved. MF-7
  done; MF-9 fidelity is true on the live store *pending the freeze*.
- **Silent-drop diff:** none. The one thing the slice docs under-tell is the two post-close commits
  (CDC-F2) — disclosed here, not dropped.
- **Arc-plan change:** flip s13 → **CDC-verified PASS** (v2.25). Correct the status line's stale
  forward-looking clause (the doc-coverage gap is *closed*, so the arc-close's remaining fidelity work is
  the **living-tail freeze**, not "a fresh `--artifacts` run"). **The arc-close is next** — MF-9
  composition + the P-12 self-host acceptance demonstration + the final reconcile-and-freeze — after which
  Migration Fidelity closes and the self-host DoD is demonstrable.

## Closure

s13 **CDC-verified PASS** on 2026-07-31, reproduced against the shipped store `odm@e06fffe` (387 nodes) —
not the `26bea1d` the ledger closed against (§1). Corpus faithful (383/2/1/1; the 2 drifts the by-design
living-plan tail); vision byte-1:1 + attested synthesis; doc-coverage gap closed by the operator `--all`
run; ODD-0020 v1.4 carve-out sound. One CDC finding routed (the living-tail freeze — a hard precondition
for MF-9/P-12), one LOW doc-hygiene note, one own-error disclosed (§6). **The arc-close reproduces P-12 —
the end of Migration Fidelity.**

_Verified by: CDC (independent), 2026-07-31 — against `odm@e06fffe` (chain `2fc25f5`→`26bea1d`→`e1e94bf`→
`e06fffe`) and `release/1.0.x` (s13 wiring + ODD-0020 v1.4). Store state reproduced by direct git read;
execution rows attested-by-CC → CI._
