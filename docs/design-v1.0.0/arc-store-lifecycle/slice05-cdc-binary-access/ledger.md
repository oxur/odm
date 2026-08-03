# Ledger — SL slice 05: CDC binary access (cargo-zigbuild)

> Per LEDGER-DISCIPLINE v2.0 §A. Evidence strength ladder:
> `asserted < attested < reproduced < reconciled`.
> Closer (CC/operator) ≠ verifier (CDC). Five-iteration cap.

| ID | Criterion | Verify | Significance | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|----------|-------|
| F-1 | `cargo zigbuild --release --target x86_64-unknown-linux-musl` produces a binary | `file bin/odm-linux-x86_64-musl` → "ELF 64-bit LSB executable, x86-64, statically linked" (or `ldd` → "not a dynamic executable") | serious | open | | The core build step. |
| F-2 | The musl binary runs in CDC's cloud container: `./odm --help` exits 0; `./odm check` exits 0 on the self-hosted store; `./odm orient` produces output | CDC stages binary via device bridge → `chmod +x` → runs each command | serious | open | | CDC reproduces — not the operator's attestation. |
| F-3 | A build script / Makefile target / justfile recipe exists that cross-compiles in one command | Run the target from a clean state → binary appears at the documented path | correctness | open | | Reproducibility. |
| F-4 | The binary location and CDC staging workflow are documented in `CLAUDE.md` or the bootstrap | A fresh session can find and use the binary without this slice's context | correctness | open | | Discoverability for future sessions. |
| F-5 | No odm source changes required (or, if musl compat needs a tweak, the change is minimal and tested) | `cargo test` still green on the native target after any changes | correctness | open | | Guard against unintended breakage. |
