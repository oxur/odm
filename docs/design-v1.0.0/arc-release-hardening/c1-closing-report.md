# C-1 closing report — Adopt Oxur themed output in odm (via `oxur-term`)

> **Arc:** Release Hardening (UAT) · **Chunk:** C-1 · **Covers:** `F-1` · **Feeds:** `RH-1`
> **Assignment:** `cc-prompt-c1-adopt-oxur-term.md` · **Decision of record:**
> `adr-c1-oxur-table-re-extraction.md` (Route B)
> **Implemented by:** CC · **Date:** 2026-07-26 · **Branch:** `rh-c1-adopt-oxur-term`
> (off `release/1.0.x`) · **Evidence class:** attested-by-CC (local 1.85+); cargo rows
> reproduce on CI.

## What shipped

odm's CLI renders through Oxur's themed output. Tables go through
`oxur_term::table::OxurTable` with the default `TableStyleConfig` (warm orange); status
lines carry the house glyphs (`✓` / `→` / `Warning:` / `Error:`).

**Base branch (the arc-plan's open item, now settled):** branched directly off
`release/1.0.x`, which already contains the A6 slice04 self-host tip (`4ac36f6`) — the
"merge green-A6-first" option had already happened, so no merge was needed.

### 1. Deps hygiene

- `oxur-term = { git = "https://github.com/oxur/oxur", tag = "0.2.1" }` added to
  `[workspace.dependencies]`; `oxur-term.workspace = true` on `odm-cli`.
- **`oxur-cli` removed** from `[workspace.dependencies]` — superseded by `oxur-term`, and it
  does not compile with `--no-default-features` (ADR §0 / evidence #4).
- `colored` added to `odm-cli` (status-line glyphs; see §3).
- `tabled` **kept** on `odm-cli` though no `tabled` item is named in odm source: the `Tabled`
  derive re-exported by `oxur-term` expands to absolute `::tabled::` paths, so the crate must
  be in scope. Verified by removing it and reading the resulting `E0433`. Commented in the
  manifest so it is not "cleaned up" later.

**Route-B invariant holds.** `cargo tree -p odm-cli` gains `oxur-term` and its transitive
deps only:

```
oxur-term v0.2.1 (https://github.com/oxur/oxur?tag=0.2.1#e2e93f93)
├── anyhow ├── colored (└── lazy_static) ├── serde ├── tabled
```

New crates in odm's graph: **`oxur-term`, `colored`, `lazy_static`** (anyhow/serde/tabled
were already there). Absent, as required: `oxur-cli`, `oxur-lang`, `oxur-comp`, `oxur-repl`,
`tokio`, `reedline`, `crossterm`, `dirs`. (`clap` is present, but as odm's own direct
dependency — not via oxur.) `oxur-term`'s manifest also lists `toml`; it does not appear in
the resolved normal-dep tree, so the prompt's expected "4 transitive deps" is really 3 + a
`lazy_static` under `colored`.

### 2. Tables → the Oxur house shape (`odm-cli/src/table.rs`)

All three table sites now render through one new module, `table::Themed`:

| Site | Before | After |
|------|--------|-------|
| `commands.rs` — `list` | `tabled::Table::new(rows).with(Style::sharp())` | `Themed`, title `NODES` |
| `migrate.rs` — `migrate` report | `tabled::builder::Builder` + `Style::sharp()` | `Themed`, title `MIGRATE` / `MIGRATE (DRY RUN)` |
| `migrate.rs` — `self-host` report | `Builder` + `Style::sharp()` | `Themed`, title `SELF-HOST` / `SELF-HOST (DRY RUN)` |

The shape — **operator-specified during implementation, from the `oxur-odm` original**:

```
row 0      title bar        (bright orange #F97316)
row 1      column names     (mid orange #D45500, flush left)
rows 2..n  data             (alternating dark bands #451A03; every cell indented one space)
last row   summary          (`Total: …`, dark orange #803300)
```
Every row also closes with a trailing space on its last cell, so no text runs flush into the
right edge of its band.

**Why not `OxurTable`.** The first cut used `OxurTable::new(rows).render()` and it rendered
*wrong*: the default theme sets `title.enabled` **and** `footer.enabled`, so it unconditionally
paints row 0 as the title bar and the last row as the footer bar. Handed a table with no title
row, it painted the **column names as the title** and the **first data row as the header** —
visible in the colour dump as row 1 in `#F97316` and node 1100 in `#D45500`. Supplying title
and summary rows is what makes the theme render correctly; they are not decoration.

`OxurTable` cannot express the rest: `with_footer()` emits a *blank* footer (no summary text),
and its `theme` field is private, so padding cannot be set. So `Themed` drives the same theme
through the crate's public lower level — `Builder` + `TableStyleConfig::apply_to_table` — which
is the path `oxur-odm` itself used (`legacy/oxur-odm/src/commands/list.rs:215-247`), and the
one the cc-prompt sanctions for commands that need more than the wrapper. The title and
summary bars carry their text in cell 0 under a `tabled` `Span::column`, so a long summary
cannot widen the first column (legacy dodged this by splitting the text across cells).

*Upstream follow-up:* `OxurTable::with_theme(…)` plus a text-carrying footer would let
`Themed` collapse onto the wrapper.

**Column parity.** `list` keeps `NUMBER TYPE NAME ID`. The two `migrate` tables keep their
columns and order, capitalised for consistency with `list` (`action`→`ACTION`,
`name / path`→`NAME / PATH`, …). Row data is untouched — verified below.

No other command prints a table: `orient`, `rollup`, `show`, `next`, `blocked`, `path`,
`check` and `reconcile` are line renderers, untouched here.

### 3. Status lines → themed

`oxur_term::common::output::{success,error,info,warning}` is the house style, but its helpers
`println!`/`eprintln!` **straight to the process's streams**. odm's command surface threads
`out`/`err` as `&mut dyn Write` precisely so `dispatch` can be driven in-process by tests
(`lib.rs` — "tests wire them to buffers … no global current-directory mutation"). Calling
those helpers from a command would write past the injected writer: the output would vanish
from every in-process test and could not be ordered against the data stream.

**Resolution:** a new `odm-cli/src/term.rs` — four writer-taking helpers emitting the *same*
glyphs and colours over the same `colored` crate. `oxur_term::common::output::error` is still
called directly in `run()`, where the sink genuinely is the process's stderr. The module
header records the upstream follow-up that removes the duplication: **`oxur-term` growing
string-returning (`success_str(&str) -> String`) or writer-taking variants**, after which
`term.rs` collapses to four one-line forwards. The fork is four format strings, deliberately
bounded; it is not a theme fork — the colours are `colored`'s named colours, not a copy of the
table theme.

Converted, by intent:

- **`✓` success** — a completed mutation: `new`, `rename`, `retire`, `supersede`, `link`,
  `unlink`, `set-gate`, `tear`, `decomposed`, `use`; `rollup`'s "wrote …"; `migrate` /
  `self-host` real-run status; `check: ok`; `reconcile: no drift`.
- **`→` info** — a plan or a statement of fact that changed nothing: every `--dry-run`
  "would …" line, `new`'s idempotent "exists: …", `migrate`/`self-host` dry-run status.
- **`Warning:`** — `unlink`'s "no-op: … has no `<edge>` edge", `check`'s warnings-only verdict,
  `reconcile`'s couldn't-check-only verdict (neither fails without `--strict`).
- **`Error:`** — `check`'s verdict when `errors > 0`, `reconcile`'s when drift is confirmed,
  and `run`'s top-level error path.

Message text is unchanged in every case — the glyph is a prefix, so existing assertions
(`contains("6 created")`, `contains("check: ok")`) still hold.

## Verification

All local, on `release/1.0.x` + this branch, Rust 1.85+ (attested-by-CC):

| Check | Result |
|-------|--------|
| `cargo build --workspace` | clean |
| `cargo test --all-features --workspace` | **all green**, 0 failed (incl. 9 new unit tests: 4 `term`, 5 `table`) |
| `cargo clippy --all-targets --workspace -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `unsafe` | none added (`rust.unsafe_code = "deny"` unchanged) |

**Reflexive check (the loop closes):**

```
$ ./target/release/odm self-host docs/design-v1.0.0 --dry-run
→ self-host (dry-run): 0 created, 46 skipped — nothing written
$ ./target/release/odm self-host docs/design-v1.0.0
✓ self-host: 0 created, 46 skipped
$ ./target/release/odm check ; echo $?
✓ check: ok (59 node(s), no problems)
0
```

Note the dry run was taken **first, deliberately**: `CLAUDE.md` freezes node minting until the
G-1 ID-scheme decision, so the cutover was proven to be a 0-created no-op before it was run
for real. No node was minted by C-1.

**Content parity — row data unchanged vs pre-C-1.** The themed `list` table was ANSI-stripped,
split on the column separator, and diffed against `odm list --json`: **59 rows, 4 columns,
identical** to the JSON projection (`number|type|name|id`). The rendering changed; the answer
did not.

**Capture.** `c1-capture-odm-list.ansi` in this directory holds the real escape sequences —
`cat` it in a truecolor terminal to see the resolved `F-1` complaint. Structure (ANSI
stripped):

```
 NODES
NUMBER│TYPE│NAME                                                    │ID
 1100 │ arc│ Arc 01 — Substrate & node CRUD (plan-of-record)        │ 01KWXMBBTKKTRXE1DF6WYSG5JB
 1200 │ arc│ Arc 02 — Graph, gates & derived order (plan-of-record) │ 01KWXMBBTKTMGY8EJV5TAEY7S0
…
 Total: 6 node(s)
```

Colours actually emitted, per row (sampled from the capture with the escapes decoded):

| Row | Background | Role |
|-----|-----------|------|
| 1 | `#F97316` | title bar |
| 2 | `#D45500` | column names |
| 3…n | `#451A03`, text alternating `#FED7AA` / `#FDBA74` | data bands |
| last | `#803300` | summary bar |

That is `TableStyleConfig::default()` — the Oxur warm-orange theme, unmodified.

## Consequences worth the operator's attention

1. **Tables always emit ANSI; status lines do not.** `colored` suppresses colour when the
   process's stdout is not a terminal (and honours `NO_COLOR`), so the `✓`/`→` prefixes are
   plain text when piped — but the table theme rides on `tabled::settings::Color`, which
   writes escapes unconditionally. `odm list > file` therefore contains escape sequences.
   That matches `oxur-odm`'s long-standing behaviour and `--json` remains the machine path,
   so it was **left as-is rather than silently widened**. If piped output should degrade to
   plain (worth considering given the UAT L-findings on LLM consumption), that is a small,
   separate decision — a TTY/`NO_COLOR` guard around `apply_to_table`, best made once for
   both odm and oxur, and most naturally landed **upstream in `oxur-term`**.
2. **`oxur-term` is a git dep pinned to tag `0.2.1`** (ADR §7.2). `Cargo.lock` pins commit
   `e2e93f93`. A crates.io publish would let odm pin a version instead; not blocking.
3. **Two tracked upstream follow-ups**, neither an accident: `term.rs` (writer-taking status
   helpers — §3) and `table.rs` (`OxurTable::with_theme` + a text-carrying footer — §2). Both
   collapse to thin forwards once `oxur-term` grows the API; both are documented in their
   module headers so the next reader finds the reason, not just the code.
4. **Scope note.** The table's title bar and `Total:` summary line are *new content*, not a
   pure rendering swap — added on the operator's instruction mid-implementation, against the
   `oxur-odm` reference. The title/summary rows are also what make the theme paint correctly
   (§2), so this is not a C-3 pre-emption: C-3 still owns columns, tree structure, dates,
   status, and width elision.

## Ledger

- **`RH-1` (C-1 closed)** — ready to close: **attested** on this report;
  **reproduced** when CI runs the cargo rows green.
- **`F-1`** — dispositioned: **shipped** (Route B, via `oxur-term`).
- Bubble-up to `arc-plan.md`: C-1 route settled (B); base-branch open item settled
  (`release/1.0.x` already carried the A6 tip); the piping/ANSI question in §1 above is raised
  as a new finding for triage rather than silently absorbed.
