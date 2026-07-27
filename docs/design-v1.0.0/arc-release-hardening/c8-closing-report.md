# C-8 closing report — normalized status for `odm node list`

> **Arc:** Release Hardening (UAT) · **Chunk:** C-8 · **Covers:** `F-19` · **Kind:** surface, display-only
> **Depends on:** C-3 (STATUS column) and C-4 (the reorg — `odm node list`)
> **Assignment:** `cc-prompt-c8-normalized-status.md` · **Implemented by:** CC · **Date:** 2026-07-27
> **Branch:** `rh-c8-normalized-status` (off the C-4 tip on `release/1.0.x`)
> **Evidence class:** attested-by-CC (local 1.85+); cargo rows reproduce on CI.

## What shipped

```
DATE       │TYPE     │STATUS  │NAME
 2026-06-26│ project │ active │ odm v1.0.0 — Project Plan (arc roadmap)
 2026-06-20│ arc     │ done   │ ├─ Substrate & node CRUD
 2026-06-20│ slice   │ done   │ │  ├─ Workspace scaffolding
 2026-06-26│ arc     │ active │ └─ Migrate, self-host & PM-skill
 2026-07-25│ slice   │ retired│    └─ UAT — CLI feedback
 2026-06-20│ design  │ planned│ odm — Architecture & Design (v-major rebuild)
```

The column shows each node's **position in its own gate ladder** rather than its furthest-reached
gate. A done arc (`verified`) and a done slice (`tested`) now read alike; an arc at `complete` reads
`active`, which is the specific misreading the chunk exists to fix.

**A view concept only.** Nothing stored, no gate added, no frontmatter touched.

## Kickoff decisions (operator, 2026-07-27)

| # | Decision |
|---|----------|
| 1 | **Labels: `planned` / `active` / `done`** (over `not-started/in-progress/done` and `todo/doing/done`). |
| 2 | **Normalized only** in the column — no `--status-format=gate`. |
| 3 | **`--status` accepts both** the derived vocabulary and raw gates. |
| 4 | **`blocked` overlay deferred** (needs the graph, not just gate position). |
| 5 | **`--json` carries the derived state**, alongside the raw ladder. |

## The deviation that matters: the brief's premise was false

Decision 2 rested on a stated basis — *"`show` and `--json` already carry the full gate vector for
anyone who needs the raw ladder, so `list` can simply show the normalized state, full stop."*

**They did not.** Checked against a real gated node before writing anything:

```
$ grep -A 12 '^status:' …/01KWXMBBTKNA3A0QC3SWPHBNAX.md
status:
  in-progress: {reached: 2026-07-07, …}
  planned:     {reached: 2026-07-07, …}

$ odm node show 1600            → no gates line at all
$ odm node show 1600 --json     → keys: id, number, type, name, origin, reserved,
                                  tags, component, retired, part_of, supersedes
```

Neither surfaced gates in any form. `orient` shows the vector for the *focus* arc only. So `list`
was the sole place any of the ladder appeared, and normalizing it alone would have made the rung
information **unreachable from the CLI entirely** — the exact opposite of the trade the decision
assumed.

The brief's own acceptance text says `show`/`--json` "still expose the raw gate vector", so making
that true is closer to its intent than shipping a decision whose justification is false. `show` now
prints the state and the whole ladder:

```
$ odm node show 1600
  status:    active
  gates:
    [x] planned — 2026-07-07 (asserted)
    [x] in-progress — 2026-07-07 (asserted)
    [ ] complete
    [ ] verified
```

and `--json` carries `status` plus a `gates` array in ladder order. Only with that in place is the
column's trade an actual trade rather than a loss.

## Two cases the brief did not name

**A type with no ladder reports nothing.** An `adr` has no gate-set, so it renders `—`, not
`planned`. It is not "planned but not started" — it has no lifecycle to be at the start of, and
rendering it as `planned` asserts something no one recorded. Pinned by
`test_derive_display_status_no_ladder_reports_nothing`, including an explicit `assert_ne!` against
`Planned`.

**`--json` emits an absent field, not the em-dash.** `—` is a *display* glyph; a machine consumer
should see the field missing and know it does not apply. Caught by writing the test against a
gate-less fixture, which is also why the JSON key-stability test now asserts **containment** rather
than an exact key set — spelling the contract as equality makes every additive field look like a
break.

## What the palette lost, and why that is right

The per-gate work palette is gone. Once the column shows a normalized state, no gate name can reach
the colour function, and a palette nothing can select is worse than none. Its **slots** survive in
`display_status_color` — early yellow, under way cyan, done green — so the tuning outlived the
mapping.

One distinction goes with it, and is worth naming rather than burying: `complete` was green and
`verified` bright green, keeping ODD-0013 §5.1's *done at its layer* and *verified live* apart.
`done` folds both. That is inherent to normalizing, the rung stays exact in `show`/`--json`, and it
is the column that trades detail for comparability — which is the whole of F-19. If the distinction
is wanted back on screen, `show`'s ladder is the place, and the slots are still there.

## Tests

The derivation takes the ladder as `&[String]` rather than a `GateSets`, so every branch is testable
with two string slices and two flags — no store, no config, no corpus. 11 unit tests over the real
ODD-0013 §5.1 ladders, not invented ones:

- terminal → `done`, mid → `active`, first-rung-or-nothing → `planned`, one case per node type;
- **`complete` is not the endpoint** (the chunk's reason for existing);
- **a done slice and a done arc are `assert_eq!` to each other** — comparability as an assertion,
  not a comment;
- position not count (skipped rungs still `done`; three reached rungs still `active`);
- overlays win over any ladder position, retired over superseded;
- a gate outside the type's own ladder says nothing about position in it;
- the three states have pairwise-distinct colours (the column leans on colour as much as text).

Three integration tests updated from C-3 (they pinned the pre-normalization behaviour) and three
added: cross-type comparability, and `--status` accepting both vocabularies without over-matching.

`gate_sets_are_read_from_the_stores_config_toml` used the raw `built` appearing in the column as its
proof the store's config was read. That proof is gone, so it now asserts on `show`'s ladder, which
names the gates outright — a stronger claim than the one it replaced.

## Verification

| Check | Result |
|-------|--------|
| `cargo test --all-features --workspace` | **58 binaries ok, 0 failed** |
| `cargo clippy --all-targets --workspace --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `unsafe` | none |
| **No model change** | `nodes/**` untouched on both branches; store branch clean |
| `odm validate` | ✓ 60 nodes, no problems — counts unchanged |
| Filter ↔ column agreement | `--status done` → 50 rows, **0** showing anything but `done` |
| Cross-type | arc `verified` and slice `tested` both render `done` |
| `--all` | #1605 renders `retired` |

## Silent-drop diff

None. Deferred as decided: the `blocked` overlay (needs the graph) and `--status-format=gate` (not
built — `show` covers the raw ladder now).

## Ledger

- **RH-8 / F-19** — dispositioned **done**; labels, `--status` vocabulary and the `blocked` deferral
  recorded above.
- **C-8** — flip to done in the arc-plan Chunks table.
- **Carried:** the `verified`/`complete` colour distinction now lives only in `show`'s ladder (plain
  text); colouring that ladder with the retained slots is a small follow-up if wanted.
