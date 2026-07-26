# C-5 closing report — the dogfood cutover + derivation overhaul

> **Arc:** Release Hardening (UAT) · **Chunk:** C-5 (the payoff) · **Also closes:** arc-store-home **SH-6**
> **Covers:** F-14 (fold), F-18 (names), F-20 (dates), L-3a (vision + focus), SH-6 (cutover)
> **Assignment:** `cc-prompt-c5-cutover.md` (completing `cc-prompt-c5-selfhost-derivation.md`)
> **Implemented by:** CC · **Date:** 2026-07-26 · **Branch:** `rh-c5-cutover` (off `release/1.0.x`)
> **Evidence class:** attested-by-CC (local 1.85+, git 2.39.5); cargo rows reproduce on CI.

## What shipped

odm now lives in the store home its own arc built. The corpus is 60 nodes on the orphan `odm`
branch at `.worktrees/odm`; the working branch no longer carries `nodes/`; `odm.toml` is a locator
and nothing else.

```
$ odm check
✓ check: ok (60 node(s), no problems)          # …in the home

$ odm orient
VISION  #1000 odm v1.0.0 — Project Plan (arc roadmap)
  `odm` is a markdown/git-native, dependency-ordered planning + documentation substrate …

CURRENT FOCUS
  arc #1600 Migrate, self-host & PM-skill — planned=asserted, in-progress=asserted, …
```

Both of those lines used to read "no vision text yet" and "(no current arc)".

| Finding | What landed |
|---------|-------------|
| **F-14** | `migrate` autodetects corpus shape (plan-set vs legacy state-tree); `--plan`/`--legacy` force it. One verb; `self-host` is a spelling of the same path. |
| **F-18** | `normalize_name` strips one positional prefix and the known role suffixes. 44 of 45 work-node names changed. |
| **F-20** | `created` = earliest git add-date of the node's own plan directory; `updated` = latest commit date. Dates now span **2026-06-20 → 2026-07-25** across **13 distinct days**, against 45× `2026-07-07` before. |
| **L-3a** | The project node carries a `# Vision` body from `project-plan.md` §1; `odm use arc 1600` sets the focus, and it is committed with the store. |
| **SH-6** | The relocation: corpus + config into the worktree home, `odm.toml` reduced to `[store]`. |

## G-1: nothing was minted

The chunk asserted it was G-1-safe, and it held. Verified mechanically rather than by inspection —
the pre-cutover ULID set from `git ls-tree` against the post-cutover set on disk:

```
pre: 60   post: 60   diff: (empty)
```

Every id preserved, none created, none lost; `check` reports the same 60 nodes with zero dangling
edges.

## Two things the brief expected that turned out to be wrong

**Nothing relocates, and nothing should.** Step 3 said the corrected `created` moves each file to
its real `nodes/<year>/<month>/`. It does not: `layout` derives the path from the **ULID's**
timestamp, not the frontmatter date — deliberately, so a node never moves on retitle or reparent.
Preserving ids therefore preserves the shard, and forcing a move would break `Store::load`, which
recomputes the path from the id. The two requirements conflict only apparently; id-preservation
wins. The `moved` counter is kept as a guard and reads **0**.

**Retired nodes are skipped.** Node #1605 is a tombstone recording a slice minted against a stale
listing, and its source directory is itself a tombstone. Re-deriving from it would have renamed the
record of the error to `"MOVED — UAT is its own arc now"` and re-dated it, quietly erasing what it
exists to say. A retired node is a record, not live work.

## What the dogfood found — three defects, all invisible before the move

This is the value of the chunk, and all three share the shape the arc-store-home report named:
**success and failure look identical**, or **the path could not be exercised where it was written**.

**1. `orient` was blind to a selection `use` had just written.** Slice 01 moved the context under
the store root "as the `.odm/` index already does", and updated `use` and `context` — the two
obvious consumers. `orient` is the third and kept reading the invocation root. Identical for every
repo and every test, because until this cutover the store *was* the invocation root. The moment the
two parted:

```
$ odm use arc 1600
✓ context: arc = Migrate, self-host & PM-skill (01KWXMBBTKNA3A0QC3SWPHBNAX)
$ odm orient
CURRENT FOCUS
  (no current arc — `odm use arc <ref>`)
```

Fixed by deriving the path from the `Store` handle rather than accepting a `&Path`, so the context
is located by the same thing that locates the nodes and no caller can pass the wrong root.

The regression tests run against a **redirected** store; in an un-redirected one they pass either
way, which is exactly why the defect survived this long. Confirmed failing against the pre-fix
source before being accepted. They assert the **CURRENT FOCUS block specifically** — my first
version asserted on the whole render and passed with an empty focus, because every node name also
appears under READY.

**2. A fresh store offered its derived state for commit.** `store init` scaffolded `config.toml`
and `nodes/` but no `.gitignore`, so the first `git add -A` in a store branch sweeps in the index —
a cache that rebuilds from the nodes and changes on nearly every command — onto a branch whose
whole purpose is to be shared.

**3. …and my first fix for (2) was itself wrong**, in the more interesting way. Ignoring all of
`.odm/` treats two different things as one. The index and drift snapshots are derived.
`context.json` is not: it is an operator stating which arc the project is on, which is precisely
what a newcomer needs — and the project's own success test is *"a fresh session reaches full
situational awareness from `odm orient` alone."* A blanket ignore would have meant every fresh
clone opening on "(no current arc)": the store carrying the plan but not the place in it. I caught
this only because deleting the caches for a cold verification also deleted the focus, and the sweep
that had just passed now failed.

The rule is written as ignore-the-directory plus one exception rather than a list of caches,
because a list fails open — `.odm/drift` already existed and was already missing from mine, and the
next cache would have been committed before anyone noticed.

## Deviations from the brief

| Brief | What happened | Why |
|-------|---------------|-----|
| Corrected `created` relocates files into their real month shard | Nothing relocates; `moved` = 0 | The shard is a function of the ULID, not the date. See above. |
| `grep -rE "^name:.*\((plan-of-record\|build plan)\)" <store>/nodes` → empty | **One hit remains: #15** `odm — Arc/Slice Breakdown (build plan)` | #15 is a *document* node, and that string is its document's actual H1 title, not an importer-appended role marker. Stripping it would rename a document to something other than its own name. **Zero work nodes carry a role suffix** — F-18's target is fully met. Flagged for CDC rather than silently stripped or silently failed. |
| Ordering: init → split config → relocate + re-stamp | init → relocate → split → re-stamp | Re-stamping last let the rewrite be verified *in its final home*; the split had to follow the move so resolution was reading the store's own `config.toml` when it did. Same steps, one rewrite. |
| Config split moves "`[gates.*]`, `[display]`, `docs_directory`, author" | Also moved `dev_directory`, `preserve_dustbin_structure`, `auto_stage_git` | Three legacy 0.3.x keys no v1.0 crate reads. The brief says don't drop any, so they moved with a comment recording that they are unread and whose arc owns the decision to re-adopt them. |
| — | `odm use arc 1600` chosen as the focus | Neither RH nor arc-store-home exists as a node — G-1 blocked minting them. #1600 is the only non-verified minted arc and the one this cutover continues. |

## The safety rule held

The brief required a dry run before the destructive step. It ran twice — once before the relocation
and once after — with identical results (45 nodes, 44 names, 44 dates, 0 relocated, 1 vision),
which is itself the evidence that the move did not disturb derivation.

The diff was then checked field-by-field across all 60 files rather than eyeballed:

```
frontmatter keys changed:  88 name   88 created   86 updated     (44/44/43 × 2 diff lines)
body changes:              1 file — the project node, insertion-only (1a2,27)
```

`id`, `number`, `type`, `schema`, gates and edges: untouched. That was the brief's red-flag list,
and it is clean.

## Verification

| Check | Result |
|-------|--------|
| `cargo build --workspace` | clean |
| `cargo test --all-features --workspace` | **58 binaries ok, 0 failed** (+1 file, +8 tests) |
| `cargo clippy --all-targets --workspace --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `unsafe` | none added |
| `check` in the home, cold | ✓ 60 nodes, no problems |
| ids pre vs post | 60 = 60, empty diff |
| `odm.toml` operational keys | 0 |
| work-node `created` range | 2026-06-20 → 2026-07-25, 13 distinct |
| #13 legacy dates | `2026-06-20` / `2026-06-26`, unchanged |

The cold sweep was re-run after deleting every cache, so it exercises committed state only.

## Silent-drop diff

None. Everything scoped in shipped. Scoped out and absent: C-4, C-6, C-8, F-21, F-16, F-17.

## Ledger

- **RH-5** (C-5 closed) — **attested** on this report; **reproduced** when CI runs the cargo rows.
- **SH-6** (arc-store-home dogfood cutover) — **attested**; this is the row the whole arc deferred
  here. With it, arc-store-home reaches its true close.
- **F-14 / F-18 / F-20 / L-3a** — dispositioned, done.
- **P-14** — the dogfood half completes.
- **Carried, unassigned:** #15's document-title parenthetical (above); the three unread legacy
  config keys now sitting in `config.toml`; `context.json` being shared rather than per-user is a
  deliberate default that a multi-operator repo may want to revisit.
