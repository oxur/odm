# UAT feedback coverage audit — F / L / G rows

> **Question answered:** is every piece of UAT feedback captured, and where does each live? Prompted
> by the retirement of node #1605 ("UAT — CLI feedback"). **Date:** 2026-07-26 (CDC).
> **Headline: nothing is lost.** All three feedback corpora are present and tracked
> (`uat-punch-list.md` → F-rows; `uat-report-llm-pass-batch2.md` → L-rows;
> `workflow-gap-coverage-review.md` → G-rows). Node #1605 held **no** feedback — it was a tombstone
> pointing at `uat-report-llm-pass-batch2.md` L-2.

## F-rows — Duncan's punch list, triaged in the RH arc-plan (21 rows)

| Row | Chunk | Status |
|-----|-------|--------|
| F-1 (colours/theming) | C-1 | **done** (RH-1) |
| F-2 (`odd`→`design`), F-3 (add `research`) | C-2 | **done** |
| F-4…F-9 (`list`: number col, date, status col, tree, de-number, width) | C-3 | **done** |
| F-15 (retired nodes shown in `list`) | C-3 | **done** |
| F-10…F-13 (`new` warn, `context`→`project`, `path`→`chain`, `rollup` md/json) | **C-4** | open |
| F-14 (`self-host`→`migrate`) | **C-5** | open |
| F-18 (names embed metadata) | **C-5** (was C-7) | open |
| F-19 (normalized status) | **C-8** | open |
| F-20 (work-node dates) | **C-5** | open |
| F-16 (unconditional table ANSI) | upstream `oxur-term` (unassigned) | open — decide |
| F-17 (body-snapshot drift) | migrate/self-host (unassigned) | open — decide |
| F-21 (`--json` omits dates) | **LLM-command-surface arc** | open |

**10 done, 11 routed-and-open** — every one has a home.

## L-rows — pass-2 LLM UAT (9 rows). The arc-llm-command-surface arc was shaped *from* this report.

| Row | Home | Status |
|-----|------|--------|
| L-1 (status is write-only) | LLM arc **slice 01** (= **G-4**) | planned (the blocking gap) |
| L-2 (nothing reconciles plan vs nodes; work with no node) | **F-15** (retired-node half, **done**) + G-1/G-3 (check rules) | partly done |
| L-4 (`next` unordered/untyped), L-5 (`why` blocked-half only) | LLM arc **slice 02** | planned |
| L-7 (`ROLLUP` missing flat form) | LLM arc **slice 03** | planned |
| L-9 (`number` looks positional) | ODD-0013 **§2.1** (number = metadata, no ordering claim) | doc-addressed |
| **L-3 (orient degrades silently on the project)** | **routed 2026-07-26 → C-5 (data) + C-6 (check rule)** — see §Routing | ✔ dispositioned |
| **L-6 (distribution: reachability not feasibility)** | **routed 2026-07-26 → release-eng follow-up (not ship-blocking)** — see §Routing | ✔ dispositioned |
| **L-8 (authority drifted out of state dirs)** | **routed 2026-07-26 → pre-release housekeeping (now) + design-corpus migrate (follow-on)** — see §Routing | ✔ dispositioned |

## G-rows — workflow gap review (12 rows). All tracked in the review with routing.

| Row | Route | Status |
|-----|-------|--------|
| G-1 (ID-scheme decision) | its own **ODD** — ⚠ gates minting new nodes | pending decision |
| G-2 (tear-rationale in `check`) | RH **C-6** (reserved) | queued |
| G-3 (`decomposed`/orphan check rule) | a `check` rule | queued (small) |
| G-4 (status read-back) | = L-1, LLM arc slice 01 (pull-forward candidate) | queued |
| G-5 (foreign-project migration + lykn `--dry-run` = UAT pass 3) | **the long pole** | queued (medium-large) |
| G-6 (PM-skill straddler) | A6 slice05 | queued (A6) |
| G-7 (auto-commit for shared repos) | **routed 2026-07-26 → arc-store-home (largely resolved-by-design)** — see §Routing | ✔ dispositioned |
| G-8 (evidence-artifact field) | additive field; A7 slice03-adjacent | queued (post-MVP) |
| G-9 (concurrent-session races) | document as a known limit | queued (doc) |
| G-10 (register/`finding` as first-class type) | decide at lykn migration (`note` vs new type) | queued |
| G-11 (cross-repo / saga), G-12 (three-repo lykn) | post-1.0, record only | deferred |

## Actionable finding — the un-routed tail (original flag, 2026-07-26)

Nothing is lost, but **four items were captured-but-not-yet-chunked** and wanted an explicit decision
(chunk it, or accept as known/deferred), rather than sitting in a report indefinitely: **L-3**, **L-6**,
**L-8**, **G-7**. All four are now dispositioned below.

---

## Routing — the un-routed tail (decisions, 2026-07-26, CDC)

> **Standing:** CDC recommendations; **operator ratifies** before they become plan-of-record (each
> creates a ledger row / chunk / checklist item named below). Routing rule followed: a fix that lives
> only in a findings doc is *not routed* until it has a named home with a verify step.

### L-3 — `orient` degrades silently on the project that defines its DoD *(serious)* — **split, two homes**

The finding has a **data** half and a **check** half; they route separately.

- **L-3a (data): the self-host cutover carried the plan's structure, not its substance** — project
  #1000 has no vision/current-focus body. → **RH C-5** (the re-self-host that *is* the SH-6
  dogfood cutover). C-5's re-derivation must **carry the vision + current-focus** into the
  project/arc nodes, not just the skeleton. **Prerequisite:** the source (`project-plan.md`) must
  actually *have* a `# Vision` / focus section to derive from — author it before the cutover, or the
  re-self-host has nothing to carry. *Action:* add an L-3a acceptance row to the **C-5** cc-prompt/ledger
  ("vision + current focus present on the project node after re-self-host; `orient` renders them cold").
- **L-3b (check): a project with no vision body is a finding** — because the DoD depends on it (a
  DoD asserted in prose and unchecked by the tool is L-2's shape). → **RH C-6**, riding the *same
  `check`-hardening pass* as G-2 (tear-rationale surfacing) and **G-3** (decomposed/orphan rule), so
  `check` churns **once**. Warn-level; `--strict` promotes. *Action:* fold L-3b + G-3 into **C-6**'s
  scope (expand C-6 from "G-2 only" to a **check-hardening** chunk: G-2 + G-3 + L-3b). Recommend the
  bundling explicitly to avoid three separate passes over `check`.

### L-6 — Distribution: reachability, not feasibility *(medium)* — **route, NOT ship-blocking; reclassified**

**Correcting the first-pass routing.** The original flag said this "folds into the LLM arc's export
slice." On re-read it does **not**: `export` is ODD-0017's *federate-don't-convert* — projecting odm
**data/views** out to other formats. L-6 is about shipping the **binary** (cargo-dist, a musl static
Linux artifact, GitHub Releases) to consumers without a Rust toolchain. That is **release/packaging
engineering**, not an odm feature — different work, different owner. → a **release-engineering
follow-up** tied to **ODD-0017's adoption thesis** (legibility to teams who haven't adopted odm — and
who don't have cargo). **Not v1.0.0-blocking:** the DoD is LLM situational awareness, `cargo install`
already works, and the finding is severity-medium. *Action:* record as a distribution task — cargo-dist
CI matrix + `x86_64-unknown-linux-musl` static artifact alongside `cargo install`; the operator's call
whether it's a small pre-1.0 RH distribution chunk (if binaries are wanted at ship) or a post-1.0
fast-follow. Default recommendation: **post-1.0 fast-follow**.

### L-8 — Authority drifted out of the state directories *(medium, ironic)* — **split, do-now + follow-on**

The `odd`/design corpus kept the *truth-encoded-in-directory-position* disease that ODD-0013 §9
retired for **nodes** (0013 v1.9 sits in `01-draft/` while its amendments 0019/0020 are in
`04-accepted/`; 0017/0018 load-bearing but `01-draft`).

- **L-8b (immediate): reconcile the four documents before release** — move/accept ODD-0013, 0017,
  0018 into the state dirs their real authority implies. → a **pre-release housekeeping checklist
  item** (not an arc; small, do-now). *Action:* add to the RH release-gate checklist. **This one
  should happen before v1.0.0 ships**, since these are the normative architecture docs and the drift
  is exactly the failure odm exists to prevent.
- **L-8a (systemic): design-doc authority = the gate vector, directory = display** — realized by
  **migrating the design/ODD corpus into `design`-type nodes** (the C-2 `design` type now exists), so
  their authority is the node's gate vector and `NN-state/` becomes a rendered view. This is
  **`migrate` against `docs/design/`** — distinct from `self-host` (which imports the *plan* tree),
  and adjacent to **G-5**'s importer generalization + the arc-store-home cutover. → **follow-on migrate
  of the design corpus, post the store-home cutover**; record now, not ship-blocking. Realizes
  ODD-0013 §9 for design docs, not just work nodes.

### G-7 — Auto-commit for shared repos *(small; shortlisted)* — **route to arc-store-home; largely resolved-by-design**

`commit_all` commits the **whole worktree** (`git.rs` builds a tree from `repo.work_dir()`); the
finding's danger is that in a multi-session, operator-only-commits repo (lykn) this sweeps unrelated
work. **The store-home architecture substantially dissolves this:** once the store lives on its own
orphan `odm` branch in the dedicated `.worktrees/odm` **worktree**, odm's git ops target *that*
worktree — which contains **only** the store — so an odm commit can no longer sweep the code branch's
working tree. The isolation that ODD-0022 buys for *sync* also buys it for *commit*. → **arc-store-home**,
as an **explicit decision + verification**, not a loose "overlaps":

1. *Confirm the mechanism* — odm's `gix` commit/read ops **target the store worktree/orphan branch**,
   not the invocation repo. This must be true for the arc to work at all; make it an **arc-store-home
   ledger row** (proposed **SH-8**, or fold into the SH-5/SH-6 compose): "odm commits land on the
   `odm` branch in the store worktree; the code branch's working tree is never touched."
2. *Decide the auto-commit surface* — recommend **no implicit commit of the code worktree ever**;
   odm's own writes commit to the store branch; team-sync stays plain `git push`/`pull` (ODD-0022 §6).
   An explicit `--commit` / curated-index option is YAGNI until asked for.
3. *Verify today's behaviour* — the finding flags `[unverified — must be settled before foreign-repo
   use]`. Check whether anything **auto-commits today** (`odm.toml`'s `auto_stage_git`, per ODD-0022
   §3) and what it stages. **Add this verification to the arc-store-home slice that wires git ops to
   the worktree** (slice 02/03), since that is where the targeting is established.

### Routing summary

| Item | Disposition | Home | Ship-blocking? |
|------|-------------|------|----------------|
| L-3a (vision not carried) | route | **RH C-5** re-self-host (SH-6 cutover) + author `# Vision` source | rides the cutover |
| L-3b (check: no-vision finding) | route | **RH C-6** check-hardening (with G-2, G-3) | no (fast-follow within RH) |
| L-6 (prebuilt binaries) | route, reclassified | **release-eng** follow-up (ODD-0017 adoption) | **no** (post-1.0 default) |
| L-8b (reconcile 4 ODDs) | **do now** | **RH release-gate checklist** | **yes — before ship** |
| L-8a (design-corpus → nodes) | route, follow-on | **`migrate` on `docs/design/`**, post-cutover | no |
| G-7 (auto-commit sweep) | route + resolved-by-design | **arc-store-home** (SH-8 + slice 02/03 verify) | settle before foreign-repo use |

**Net new plan actions for operator ratification:** (1) expand **C-6** to a check-hardening chunk
(G-2 + G-3 + L-3b); (2) add an **L-3a** acceptance row to **C-5**; (3) add a **pre-release checklist**
row for **L-8b**; (4) record **L-6** + **L-8a** as post-1.0 follow-ups; (5) add **SH-8** (worktree-scoped
commits) to arc-store-home and a `auto_stage_git` verification to slice 02/03.

Everything else in the tables above is done, routed to a named chunk/arc, or a recorded post-1.0 deferral.
