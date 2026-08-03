# Closing Report — Slice 05 (Store Lifecycle): CDC binary access (cargo-zigbuild)

> Verified by: CDC (Cowork cloud session, F-1/F-2, `cdc-verification.md`) + CC (this session, F-3/F-4/F-5).
> All five ledger rows `done`. Closed 2026-08-03 on `release/1.0.x`.

## The walk

Per-row disposition is in `ledger.md`'s Evidence column; this is the narrative. This slice split cleanly
across the two roles the cc-prompt named: **the operator/CC build and document; CDC reproduces the actual
cloud-container run.**

**F-1 (the binary is real).** CDC's independent build — `cargo zigbuild --release --target
x86_64-unknown-linux-musl` — produced a binary the cloud container's own `file`/`ldd` confirmed static:
"ELF 64-bit LSB executable, x86-64, ..., statically linked, stripped", "not a dynamic executable". Not
CC-attested — CDC reproduced this from the operator's raw build output before any Makefile target existed.

**F-2 (the acceptance anchor — runs in CDC's actual container).** CDC staged the binary via the Cowork
device bridge, assembled a minimal store layout (`odm.toml`, `config.toml`, one node), and exercised the
full command surface named in the cc-prompt plus more: `--help`, `--version`, `check` (+`--json`),
`orient` (+`--json`, `drift.tracked:true` proving live store connection), `validate`, `node show`,
`store --help` — all exit 0. This is the row that mattered most (per the cc-prompt: "CDC must run the
binary in the actual cloud container, not just assert it works") and it's the one row CC could not have
closed — no local access to a genuine x86_64 Linux cloud container exists in this session either.

**F-3 (reproducible build).** CDC's verification report flagged the real gap correctly: no cross-compile
Makefile target existed, so the binary CDC tested was a manual, undocumented `cargo zigbuild` invocation.
Added `make build-linux` — guards on `cargo-zigbuild` and the `x86_64-unknown-linux-musl` rustup target
being present (both already installed on this machine), runs the zigbuild, copies the result to
`bin/odm-linux-x86_64-musl`, and — unlike a naive `clean`-dependent target — does **not** wipe or touch the
native `bin/odm` macOS binary, since `build-linux` doesn't depend on `clean` the way `build`/`build-release`
do. Ran clean from this session's state: `file bin/odm-linux-x86_64-musl` → static ELF x86-64, 7.0M.
Matches CDC's sketch in `cdc-verification.md` closely (CDC's sketch called `strip` explicitly; `--release`
+ zigbuild already stripped the binary, confirmed by `file`'s own "stripped" report, so the extra step
was dropped as redundant).

**F-4 (documented).** Added a "CDC binary access" section to `CLAUDE.md` (after the existing Build &
Development Commands block): build command, artifact location (flagged as gitignored, not committed —
consistent with the slice-doc's explicit scope exclusion), a staging note, and source-build-as-fallback.
One thing added beyond CDC's suggested wording: an explicit **x86_64-only architecture caveat**. This
machine is Apple Silicon (arm64); `device_bash` — the *other* Linux-VM bridge this project already uses
(distinct from CDC's own Cowork cloud container, per `CDC-SESSION-BOOTSTRAP.md`'s bridge-mechanics
section) — runs in a VM on this same Mac and its architecture is not verified here. A future session
should not assume every "Linux bridge" in this project is x86_64 just because CDC's *cloud container*
happens to be; the doc now says to check `uname -m` before assuming the binary will run rather than
silently repeating that assumption.

**F-5 (no regression).** Zero Rust source changes — only `Makefile` (new target) and `CLAUDE.md`
(new section). Ran `make test` (full workspace, all crates + doctests) and `make lint` (clippy `-D
warnings` + rustfmt --check) after the change: both green. This closes the "operator's next `cargo test`
confirms no regression" deferral CDC's report left open.

## Scope discipline

Diff: `Makefile` (`build-linux` target + a help-text line), `CLAUDE.md` (new documentation section). No
Rust source changed. No binary committed to git (`bin/` is gitignored — confirmed via `git check-ignore`)
— consistent with the slice-doc's explicit "Out of scope: binary committed to git" line.

## Iterations

One pass for the CC-side rows (F-3/F-4/F-5); F-1/F-2 were already closed by CDC's independent cloud-session
verification before this session started. No rework needed — CDC's `cdc-verification.md` recommendation
for the Makefile target and CLAUDE.md wording matched almost exactly what was implemented, modulo the
redundant `strip` call and the added architecture caveat.

## Bubble-up → `../arc-plan.md`

- **Slice 05 done**, delivering ledger row **SL-7**: CDC can now run `odm` without building from source —
  build it via `make build-linux`, stage `bin/odm-linux-x86_64-musl`, run directly. This unblocks CDC's
  participation in verifying **s02** (`store status`) and **s03** (`store sync`), the arc's next slices.
- **What implementing it revealed that the arc-plan didn't need to anticipate further:** the slice's
  actual bottleneck wasn't the cross-compilation itself (tooling was already installed, the build already
  worked) — it was that the *reproducibility and discoverability* artifacts (a checked-in build target,
  a documented location) hadn't caught up to the fact that CDC had already used a hand-built binary once.
  Worth naming for future arc-plan readers: when an operator-driven slice's core mechanism already works
  informally, the slice's real deliverable is usually turning that informal, undocumented step into
  something a fresh session can reproduce without tribal knowledge — exactly what F-3/F-4 targeted.
- **Silent-drop check:** all 5 ledger rows closed done, none deferred or no-op. No rows dropped.
- **s02/s03 remain the arc's immediate priority** (per the arc-plan's 2026-08-03 resume note) — this
  slice was explicitly sequenced ahead of them to unblock CDC's participation in verifying those two.
