# Slice 05 — CDC binary access (cargo-zigbuild cross-compilation)

> **Arc:** Store Lifecycle · **Slice:** 05 · **Status:** open
>
> **Why now:** CDC (Cowork / cloud container sessions) cannot run `odm` without
> building from source. ~50% of new instances refuse to build Rust, and even
> when they do, the build takes minutes and pulls in the full toolchain. The
> Store Lifecycle arc's remaining slices (s02 `store status`, s03 `store sync`)
> need CDC to verify their output — which requires a working binary. Meanwhile
> the operator already builds locally on every change. Adding a cross-compiled
> musl target to the operator's build produces a static Linux binary CDC can
> pick up directly from the worktree — no Rust toolchain in the container, no
> duplicate build, no wasted context arguing about whether Rust is available.

## Goal

Add a **cargo-zigbuild** cross-compilation step to the operator's local build
that produces a **fully static `x86_64-unknown-linux-musl` binary**. The binary
is placed in a known location in the worktree so CDC can stage it via the
device bridge. The result: when the operator runs `make` (or the agreed build
command), a Linux binary appears alongside the macOS binary, and CDC's setup
shrinks to "stage the binary, `chmod +x`, run."

## Scope

### In

- Install `cargo-zigbuild` + the zig toolchain + `x86_64-unknown-linux-musl`
  Rust target on the operator's machine.
- Verify the build: `cargo zigbuild --release --target x86_64-unknown-linux-musl`
  produces a working static binary.
- Add a build target (Makefile, justfile, shell script, or cargo alias) that
  makes cross-compilation reproducible and part of the regular build flow.
- Place the binary at a documented location (e.g., `bin/odm-linux-x86_64-musl`
  or `target/x86_64-unknown-linux-musl/release/odm` — operator's call on
  whether to copy it somewhere convenient).
- CDC verification: stage the binary into the cloud container, confirm it runs
  (`--help`, `check`, `orient`, and any store commands that exist at the time).
- Document the workflow (in `CLAUDE.md` or equivalent) so future sessions know
  where the binary is and how to use it.

### Out

- **GH Actions CI release workflow** — that's the durable, multi-target
  distribution solution (project-plan §4a, its own future item). This slice is
  the operator's local workflow, not CI.
- **macOS / Windows / aarch64 targets** — those come with the GH Actions
  workflow. This slice produces only `x86_64-unknown-linux-musl` (the CDC
  container's architecture).
- **Binary committed to git** — the musl binary is a build artifact, not a
  source artifact. It lives in the worktree (gitignored) or a known local
  path, not checked into the repo. (GH releases will handle distribution.)
- **Changes to odm source code** — this is a build/tooling slice, not a
  feature slice. No Rust code changes expected (unless a build.rs or
  Cargo.toml tweak is needed for musl compatibility).

## Verification approach

- **F-1 (build):** `cargo zigbuild` produces a binary; `file` confirms it's a
  static ELF x86_64 executable; `ldd` reports "not a dynamic executable" (or
  "statically linked").
- **F-2 (runs in container):** CDC stages the binary into the Cowork cloud
  container and runs `./odm --help`, `./odm check`, `./odm orient` — all
  succeed.
- **F-3 (reproducible):** A build script/target exists; running it from a
  clean state produces the binary.
- **F-4 (documented):** `CLAUDE.md` (or the bootstrap) tells a fresh session
  where the binary is and how to stage it.

## Exit criteria

The operator's regular build produces a static Linux binary; CDC can stage and
run it; the workflow is documented.

## Notes

This is an **operator-driven slice** — the operator installs the tooling and
runs the build; CDC's role is verification (test the binary in the container).
The cc-prompt is written from that perspective.

This slice is sequenced as s05 but **runs ahead of s02/s03** — it enables CDC
to participate in verifying the store status and sync commands those slices
will implement. It's infrastructure that unblocks the rest of the arc.
