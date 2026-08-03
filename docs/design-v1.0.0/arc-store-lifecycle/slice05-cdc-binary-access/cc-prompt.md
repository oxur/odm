# CC Prompt — SL slice 05: CDC binary access (cargo-zigbuild)

> **Note:** This slice is **operator-driven** — the operator installs the
> cross-compilation tooling and runs the build. CDC's assignment is
> **verification**: confirm the resulting binary works in the cloud container
> and document the workflow.

## Context

The Store Lifecycle arc needs CDC to verify `store status` (s02) and
`store sync` (s03), but CDC cannot reliably build odm from source (~50% of
new instances refuse to run the Rust toolchain). The operator is adding a
**cargo-zigbuild** cross-compilation step that produces a static
`x86_64-unknown-linux-musl` binary — a fully self-contained Linux executable
that runs in CDC's cloud container without any Rust toolchain.

## CDC assignment

### 1. Receive and test the binary

When the operator has built the musl binary and placed it in the worktree:

1. Stage it from the device: use the device bridge to copy it into the cloud
   container.
2. Verify it's a static binary:
   ```
   file ./odm       # expect "ELF 64-bit LSB executable, x86-64, statically linked"
   ldd ./odm        # expect "not a dynamic executable" or "statically linked"
   ```
3. Run the smoke tests:
   ```
   ./odm --help          # exits 0, shows usage
   ./odm --version       # shows the current version
   ./odm check           # exits 0 on the self-hosted store (if store is available)
   ./odm orient          # produces orientation output (if store is available)
   ```
4. If the store worktree is available (staged from device), also test:
   ```
   ./odm list            # lists nodes
   ./odm node show <ID>  # shows a known node
   ```

### 2. Document the workflow

Update the relevant `CLAUDE.md` (the 1.0.x worktree's) with:
- Where the musl binary lives in the worktree
- How to stage and run it in a Cowork session
- That building from source is a fallback, not the primary path

### 3. Report

Write `closing-report.md` and walk each ledger row (F-1 through F-5) with
evidence. The key reproduction is **F-2** — CDC must run the binary in the
actual cloud container, not just assert it works.

## Acceptance

All five ledger rows at `done`; the binary runs in the container; the workflow
is documented for future sessions.
