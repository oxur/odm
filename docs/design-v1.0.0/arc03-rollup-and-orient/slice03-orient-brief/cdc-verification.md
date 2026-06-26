# CDC Verification — Arc 03 / Slice 03: orient / brief + bare-`odm`

> Independent verification of CC's closed ledger (impl `f3d9f21`; closed `3004d0b`),
> per LEDGER_DISCIPLINE. Evidence levels (0001-D3): **reproduced** = CDC re-ran in
> this session; **attested** = CC's evidence (incl. CC's local 1.95.0 run), not
> independently reproduced here.

## Environment constraint (disclosed)

Same as slices 01–02: no 1.85+ toolchain in the sandbox, so cargo executions route to
CI / CC's local 1.95.0 run (held as **attested pending CI**). CDC reproduces structural
rows by reading the branch `arc03-slice03-orient-brief` at `f3d9f21`.

## Row dispositions

**Row count:** 11 opened, 11 addressed. No silent drops. ✔

**Reproduced by CDC (structural, at `f3d9f21`):**

- **O-1** — `orient` (`odm-cli/src/orient.rs:39`) calls `Rollup::assemble` (l.62) and
  `commands::integrity_findings` (l.63); builds no graph itself. Section order vision →
  focus → ready/blocked → integrity → drift is the render sequence. ✔
- **O-2 / O-3** — `extract_vision(body, show_number)` (l.224): `# Vision` ATX heading
  (case-insensitive, bounded by the next same-or-higher heading) else the lead section,
  truncated to a line budget with a continuation marker; 6 unit cases (l.314–393).
  `orient` leads with the project name + excerpt, not the raw body. ✔
- **O-5** — soft-sat surfaces on the ready frontier: `⚠ soft dep <dep> at evidence=<level>`
  (l.142–147), travelling with `ReadyNode.soft` (slice02 ruling 2). ✔
- **O-6** — integrity via `commands::integrity_findings` (`commands.rs:1308`) → wraps
  `aggregate` (l.1313) with the explicit contract "integrity is never re-walked"
  (l.1303). Reuse, not reimplementation. ✔
- **O-8** — bare `odm`: `command: Option<Command>` (`lib.rs:40`) →
  `unwrap_or(Command::Orient)` (l.422); binary-level `bare_odm_orients` test in
  `oxur-odm/tests/cli.rs:112`. ✔
- **O-9** — never-bare-error fallbacks: `render_no_project` ("→ create one: `odm new
  project`", l.71–74) and the multiple-projects branch ("→ choose one: `odm use
  project <ref>`", l.87); `orient` returns `Ok` on these paths → exit 0. ✔
- **O-10** — `brief`: `#[command(visible_alias = "brief")]` on `Orient` (`lib.rs:258`)
  → same variant, identical output. ✔
- **O-11 (no `unsafe`)** — grep over odm-core/odm-cli src → no matches. ✔

**Attested by CC (local rustc 1.95.0), pending CI:** `cargo test --workspace` → 218
passed, 0 failed; clippy `--all-targets -- -D warnings` → exit 0; `fmt --check` clean;
line coverage **odm-core 98.69% / odm-cli 93.44%** (orient.rs 97.78%), both ≥ 90%. →
**PENDING CI.**

## Rulings on CC's flagged items

1. **`orient` is stdout-only (no stderr).** **Accepted.** `orient` is a pure query
   that exits 0; its no-project affordances are guidance *within* a successful view,
   not error diagnostics, so stdout is correct. (If a genuine error path ever appears
   — e.g. a corrupt store mid-read — that should route to stderr; none exists today.)
2. **Integrity surfaces *all* `check` Errors** (a superset of the spec's "orphan +
   cycle-without-tear"). **Accepted, and better.** Using the severity classification
   rather than an enumerated list is the correct generalization and stays DRY — "every
   Error is unmissable" is exactly the MVP-DoD intent (slice02 ruling 3).
3. **Six rare/defensive lines in `orient.rs` uncovered.** **Accepted** — named
   explicitly; odm-cli line coverage 93.44% (orient.rs 97.78%) clears the floor.

## Verdict

**Arc 03 / Slice 03 CDC-verified on structure; all flags accepted (two improve on the
spec); cargo rows pending CI.** `orient`/`brief` compose over the slice02 model with no
re-derivation, bare `odm` orients and never bare-errors, and both slice02 CDC rulings
(soft-sat ⚠ on the ready frontier; `check` integrity surfaced inline) are present and
verified — so a fresh session reaches full situational awareness from `odm orient`
alone (0015 §2). On CI green, the slice closes.

**MVP capability is feature-complete** — only **slice04 (`--json` + polish)** remains
to close the arc; it pins the orient/rollup `--json` and the `check --json` v2 envelope
as canonical schemas (the standing forward note).

CDC: planning thread, 2026-06-25. Iterations used: 1.
