# CC Prompt — Slice 06: `store set-remote`

> **Branch:** `release/1.0.x` · **Ledger:** `ledger.md` (F-1…F-8)
> **Slice doc:** `slice-doc.md` · **CLAUDE.md** governs style and safety.

## Overview

Add `odm store set-remote` — a thin config command that tells the store which
git remote to sync to — and teach `sync` to handle the **first-push** case
(remote configured, branch never pushed). Today `sync` hardcodes
`DEFAULT_REMOTE` ("origin") and lumps a missing upstream ref together with "no
remote at all" as `NoUpstream`. After this slice: `store init → set-remote →
commit → sync` pushes the store to a remote on the first run, no raw git.

Six code-level changes, one new test file. Read each section; the line numbers
are from the current `release/1.0.x` HEAD. Where a section says "add," it means
new code; where it says "modify," there is existing code to change.

---

## 1. Add `remote` to `StoreLocation` (home.rs)

**File:** `crates/odm-store/src/home.rs`
**Struct:** `StoreLocation` (line 49)

Add a new field after `branch_name` (line 63):

```rust
/// The git remote the store syncs to. Written by `set-remote` or
/// auto-detected by `init` when exactly one remote exists. Absent in
/// stores created before this slice; `sync` falls back to
/// `DEFAULT_REMOTE` ("origin") when `None`.
#[serde(default)]
pub remote: Option<String>,
```

`#[serde(default)]` makes existing `odm.toml` files (which lack the field)
parse as `None` — backward compatibility with zero migration.

**Do not** add `remote` to the `Default` impl (line 74) — `None` is what
`Option::default()` already gives.

---

## 2. Add `list_remotes` and `remote_url` to worktree.rs

**File:** `crates/odm-store/src/worktree.rs`

Add two new public helpers. Place them after the existing `push()` (line 245),
before `merge_ff_only()` (line 265). The module doc comment (line 1) should
be updated to mention the new helpers in the scope-widening paragraph.

### 2a. `list_remotes`

```rust
/// Lists the repository's configured remotes (`git remote`).
///
/// Returns an empty `Vec` when there are none — not an error, since a
/// local-only repo is a valid state.
///
/// # Errors
///
/// [`StoreError::Git`] if git cannot be run.
pub fn list_remotes(repo_root: &Path) -> Result<Vec<String>> {
    let out = capture(repo_root, &["remote".into()])?;
    Ok(out
        .map(|t| t.lines().map(str::to_string).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default())
}
```

### 2b. `remote_url`

```rust
/// The URL configured for `remote` (`git remote get-url`), or `None` when
/// the remote does not exist.
///
/// Used by `set-remote` to validate that a named remote is real before
/// writing it to the config.
///
/// # Errors
///
/// [`StoreError::Git`] if git cannot be run.
pub fn remote_url(repo_root: &Path, remote: &str) -> Result<Option<String>> {
    let out = capture(repo_root, &["remote".into(), "get-url".into(), remote.into()])?;
    Ok(out.map(|t| t.trim().to_string()).filter(|u| !u.is_empty()))
}
```

Both use the existing `capture()` helper (line 460), which returns `None` for
non-zero exit — exactly right here.

---

## 3. Add `StoreCommand::SetRemote` variant (lib.rs)

**File:** `crates/odm-cli/src/lib.rs`
**Enum:** `StoreCommand` (line 217)

Add a new variant after `Sync` (lines 311–318), before the enum's closing `}`
(line 319):

```rust
/// Configure which git remote the store syncs to.
///
/// **Explicit:** `odm store set-remote <name>` — validates the named
/// remote exists (`git remote get-url`), writes `remote = "<name>"` to
/// `[store]` in `odm.toml`, and reports. Re-running with a different
/// name overwrites.
///
/// **Auto-detect:** `odm store set-remote` (no argument) — if exactly
/// one remote is configured, uses it; zero or multiple remotes error
/// with a clear message listing the available names. No interactive
/// prompt.
SetRemote {
    /// The remote to use. Omit to auto-detect (exactly one must exist).
    #[arg(value_name = "REMOTE")]
    remote: Option<String>,
    /// Emit JSON describing the result.
    #[arg(long)]
    json: bool,
},
```

Add the dispatch arm after the `Sync` dispatch (line 961):

```rust
StoreCommand::SetRemote { remote, json } => {
    store_cmd::set_remote(root, remote.as_deref(), json, out, err)?;
}
```

---

## 4. Add `set_remote()` handler (store_cmd.rs)

**File:** `crates/odm-cli/src/store_cmd.rs`

Add a new function. Suggested placement: before `sync_cmd()` (line 762).

### Logic

1. Resolve the store home: `StoreHome::resolve(root)`. Bail if no `[store]`
   section (same guard as `sync_cmd`, line 770).
2. Choose the remote name:
   - If `remote` arg is `Some(name)`:
     - Validate: `worktree::remote_url(&repo_root, name)?`. If `None`, bail
       with exit 2: `"remote {name:?} does not exist — available remotes:
       {list}"` (use `worktree::list_remotes`).
     - `auto_detected = false`.
   - If `remote` arg is `None`:
     - `let remotes = worktree::list_remotes(&repo_root)?;`
     - `remotes.len() == 0` → bail exit 2: `"no remotes configured for this
       repository"`.
     - `remotes.len() > 1` → bail exit 2: `"multiple remotes configured —
       specify one: {}"` (join the names).
     - `remotes.len() == 1` → use `remotes[0]`, `auto_detected = true`.
3. Get the URL for reporting: `worktree::remote_url(&repo_root, &chosen)?`
   — should be `Some` since we validated.
4. Update `odm.toml`: read the file, use `toml::Value` to parse, set
   `table["store"]["remote"] = chosen.into()`, serialize back with
   `toml::to_string_pretty`, write. (See §4a below for the TOML update
   approach.)
5. Report:
   - `--json` → `SetRemoteJson { remote, url, auto_detected, branch,
     store_root }`.
   - Plain → `term::success`: `"store set-remote: using {remote:?} ({url})"`
     — add `"(auto-detected)"` when `auto_detected`.

### 4a. TOML update approach (D-5)

`write_locator()` (init.rs line 431) appends a new `[store]` section — it runs
only at init time when the section doesn't exist yet. `set-remote` must update
an *existing* section. The recommended approach:

```rust
fn update_remote_in_locator(repo_root: &Path, remote: &str) -> anyhow::Result<()> {
    let path = repo_root.join(odm_store::home::LOCATOR_FILE);
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;
    let mut doc: toml::Value = text.parse()
        .with_context(|| format!("parsing {}", path.display()))?;
    doc.as_table_mut()
        .and_then(|t| t.get_mut("store"))
        .and_then(|s| s.as_table_mut())
        .ok_or_else(|| anyhow::anyhow!(
            "no [store] section in {} — run `odm store init` first",
            path.display()
        ))?
        .insert("remote".to_string(), toml::Value::String(remote.to_string()));
    std::fs::write(&path, toml::to_string_pretty(&doc)?)
        .with_context(|| format!("writing {}", path.display()))
}
```

**Trade-off noted (D-5):** `toml::to_string_pretty` does not preserve comments
or manual formatting. The `odm.toml` written by `write_locator` is
machine-generated with a single comment line; this approach is acceptable. If
comment preservation becomes important later, switch to the `toml_edit` crate —
but do not add that dependency in this slice.

### 4b. JSON shape

```rust
#[derive(Serialize)]
struct SetRemoteJson {
    /// The remote name now configured.
    remote: String,
    /// Its URL.
    url: String,
    /// Whether it was auto-detected (no argument given).
    auto_detected: bool,
    /// The store branch.
    branch: String,
    /// The store root path.
    store_root: String,
}
```

### 4c. Error shape under `--json`

Follow the house pattern: errors under `--json` still exit non-zero. Use
`anyhow::bail!` (which returns exit 2 via the existing dispatch error path).
The JSON flag doesn't change the error reporting mechanism — the caller sees
a non-zero exit and can read stderr.

---

## 5. Modify `sync_cmd()` to read the configured remote and handle first-push

**File:** `crates/odm-cli/src/store_cmd.rs`
**Function:** `sync_cmd()` (line 762)

Two changes:

### 5a. Read the remote from config

Replace the hardcoded `init::DEFAULT_REMOTE` references (lines 784, 786, and
the `LocalAhead` push at line 824) with a `remote` variable read from the
store location:

```rust
// After resolving `location` (line 774):
let remote = location.remote.as_deref().unwrap_or(init::DEFAULT_REMOTE);
```

Then use `remote` wherever `init::DEFAULT_REMOTE` appeared:
- Line 784: `let _ = worktree::fetch(&repo_root, remote);`
- Line 786: `let upstream_ref = format!("{remote}/{branch}");`
- Line 824 (inside `LocalAhead` arm): `worktree::push(&repo_root, remote, &branch)?;`

### 5b. First-push path

Currently, when `local` is `Some` but `upstream` is `None` (branch exists
locally, never pushed), the code falls through to the `_ =>` arm (line 801):

```rust
_ => (init::sync_action(None), 0, 0),
```

This yields `NoUpstream`, which reports "nothing to sync." But if the fetch
*succeeded* (the remote is reachable), the branch just hasn't been pushed yet.
The fix: after the fetch, check whether the remote is valid. If it is, and
`local` is `Some` but `upstream` is `None`, treat this as a first-push.

**Approach:** Track whether the fetch succeeded:

```rust
let fetch_ok = worktree::fetch(&repo_root, remote).is_ok();
```

Then split the fallthrough:

```rust
(Some(_local), None) if fetch_ok => {
    // Remote is reachable but doesn't carry the branch yet → first-push.
    // Reuse the LocalAhead action with the local commit count.
    let count = worktree::count_commits(&repo_root, &format!("{branch}"))?;
    (SyncAction::LocalAhead(count.max(1)), count, 0)
}
_ => (init::sync_action(None), 0, 0),
```

In the execution block, the existing `LocalAhead` arm already pushes — no
change needed there. The JSON action will be `"pushed"`, matching the
slice-doc's D-4.

**Dry-run:** The `!dry_run` guard (line 803) already wraps the push, so
`--dry-run` naturally skips the push and reports "would push."

**Plain-text:** The existing `LocalAhead(n)` match arms in the text output
(lines 861–866) already cover both dry-run and real variants.

---

## 6. Modify `store init` to auto-set remote (init.rs)

**File:** `crates/odm-store/src/init.rs`
**Function:** `bootstrap()` (line 291)

After `write_locator()` (line 325), add:

```rust
// Auto-set remote when exactly one exists (D-3).
let remotes = worktree::list_remotes(&plan.repo_root)?;
if remotes.len() == 1 {
    // Update the locator we just wrote: add `remote = "<name>"`.
    // Use the same TOML-update path `set-remote` uses — but since we
    // just appended the section, `toml::Value` can parse it.
    let path = plan.repo_root.join(LOCATOR_FILE);
    let text = std::fs::read_to_string(&path)
        .map_err(|e| StoreError::io(&path, e))?;
    let mut doc: toml::Value = text.parse()
        .map_err(|_| StoreError::Git("could not parse odm.toml".into()))?;
    if let Some(store) = doc.as_table_mut()
        .and_then(|t| t.get_mut("store"))
        .and_then(|s| s.as_table_mut())
    {
        store.insert("remote".into(), toml::Value::String(remotes[0].clone()));
        let serialized = toml::to_string_pretty(&doc)
            .map_err(|e| StoreError::Git(format!("serializing odm.toml: {e}")))?;
        std::fs::write(&path, serialized)
            .map_err(|e| StoreError::io(&path, e))?;
    }
}
```

**Alternative (simpler, if you prefer):** modify `write_locator()` to accept an
`Option<&str>` for the remote and append the field when present. Then pass
the auto-detected remote (or `None`) from `bootstrap()`. This avoids the
parse-update-serialize round-trip but couples `write_locator`'s signature to
this feature. Either approach is acceptable; flag the choice.

Also: the `init.rs` `sync()` function (line 543) uses `DEFAULT_REMOTE` at lines
546 and 562. These should also read from `plan.location.remote` when present,
falling back to `DEFAULT_REMOTE`. But note that `init::sync()` is only called
during `store init` (when the store already exists locally and needs a
fast-forward), and the `location` at that point is freshly parsed from
`odm.toml` — so if a `remote` was set, it'll be on the location. Change:

```rust
let remote = plan.location.remote.as_deref().unwrap_or(DEFAULT_REMOTE);
let upstream_ref = format!("{remote}/{branch}");
// ...
let _ = worktree::fetch(&plan.repo_root, remote);
```

Same pattern as §5a.

---

## 7. Tests: `crates/odm-cli/tests/store_set_remote.rs`

New file. Follow the pattern in `store_sync.rs` (the `run()` helper, the
`git_init()` + `bootstrapped_store()` scaffolding, the bare-remote fixture).

### Required fixtures (mapped to ledger rows)

| Fixture | Ledger | What it tests |
|---------|--------|---------------|
| `set_remote_explicit` | F-1 | `set-remote origin` → odm.toml gains `remote = "origin"` |
| `set_remote_explicit_nonexistent` | F-1 | `set-remote nope` → error exit 2, clear message |
| `set_remote_explicit_overwrite` | F-1 | `set-remote origin` then `set-remote other` → second value wins |
| `set_remote_auto_one` | F-2 | repo with one remote, `set-remote` (no arg) → auto-detects |
| `set_remote_auto_zero` | F-2 | repo with no remotes → error exit 2 |
| `set_remote_auto_multi` | F-2 | repo with two remotes → error listing both |
| `sync_first_push` | F-3 | `init + set-remote + commit + sync --json` → action `"pushed"`, remote has the commit |
| `sync_first_push_dry_run` | F-3 | same but `--dry-run` → reports "would push", remote unchanged |
| `sync_reads_configured_remote` | F-4 | set-remote to a non-"origin" name → sync uses it |
| `sync_falls_back_to_default` | F-4 | store with no `remote` field → sync falls back to "origin" |
| `init_auto_sets_remote` | F-5 | `init` in single-remote repo → odm.toml has `remote` |
| `init_no_remote_no_field` | F-5 | `init` in a no-remote repo → no `remote` field |
| `init_multi_remote_no_field` | F-5 | `init` in a two-remote repo → no `remote` field |
| `set_remote_json` | F-6 | `set-remote --json` → parses, fields correct |
| `set_remote_json_auto` | F-6 | `set-remote --json` (auto) → `auto_detected: true` |

### Bare-remote scaffold

Reuse the pattern from `store_sync.rs`:

```rust
fn add_remote(repo: &Path, name: &str) -> TempDir {
    let bare = TempDir::new().unwrap();
    Command::new("git")
        .args(["init", "--bare", "-q"])
        .current_dir(bare.path())
        .status().unwrap();
    Command::new("git")
        .args(["remote", "add", name, &bare.path().display().to_string()])
        .current_dir(repo)
        .status().unwrap();
    bare
}
```

For fixtures needing the commit on the remote, push after `store commit`:

```rust
// After `store commit`:
Command::new("git")
    .args(["push", "origin", "odm"])
    .current_dir(repo)
    .status().unwrap();
```

### Verifying odm.toml content

After `set-remote`, read `odm.toml` and parse:

```rust
let text = std::fs::read_to_string(repo.join("odm.toml")).unwrap();
let doc: toml::Value = text.parse().unwrap();
let remote = doc["store"]["remote"].as_str().unwrap();
assert_eq!(remote, "origin");
```

---

## 8. Clippy and safety (F-8)

- `clippy -D warnings` must pass with the new code.
- No `unsafe` anywhere in the new code.
- Run `cargo test -p odm-cli --test store_set_remote` to confirm all fixtures
  pass.
- Run `cargo test -p odm-cli` to confirm no regressions.

---

## Implementation order

1. `home.rs` — add the field (§1). Smallest change, no dependencies.
2. `worktree.rs` — add `list_remotes` and `remote_url` (§2).
3. `init.rs` — modify `bootstrap()` for auto-set and `sync()` for reading the
   configured remote (§6).
4. `lib.rs` — add the `SetRemote` variant and dispatch (§3).
5. `store_cmd.rs` — add `set_remote()` handler (§4) and modify `sync_cmd()`
   (§5).
6. `store_set_remote.rs` — write the tests (§7).
7. `cargo clippy -D warnings && cargo test -p odm-cli` (§8).

---

## Consistency note: `status` and the configured remote

`status()` (store_cmd.rs line 645) also hardcodes `init::DEFAULT_REMOTE` for
its ahead/behind comparison. F-7 says "status/commit unmodified," and the
intent is that this slice doesn't change the command surface of status. But if
`sync` reads a configured remote while `status` keeps hardcoding "origin,"
the two will disagree when `set-remote` points at a non-origin remote — status
would show ahead/behind against origin, while sync acts against the configured
one.

**Recommended:** apply the same one-line change to `status()`:

```rust
let remote = location.as_ref()
    .and_then(|l| l.remote.as_deref())
    .unwrap_or(init::DEFAULT_REMOTE);
let upstream_ref = format!("{remote}/{branch}");
```

This is behavioral consistency, not a structural change to the command — F-7's
intent (no new flags, no new subcommand) is preserved. Flag the choice.

## What not to touch

- **`status`** — stays read-only; no new flags or subcommands (s02 D-1 stands).
  The one-line remote-reading change above is a consistency fix, not a
  modification in the F-7 sense.
- **`commit`** — unchanged.
- **Node schema / ODD-0022** — no model changes; ODD-0022 is cited but not
  modified.
- **`store_sync.rs` (existing tests)** — should pass unchanged (the
  `DEFAULT_REMOTE` fallback preserves existing behavior).

---

## Ledger evidence targets

Every fixture above is class-(a): CDC reproduces by direct read of the test
output. The integration test (`init → set-remote → commit → sync → remote has
the commit`) is the real test (F-3). The ledger rows reach `reproduced` when
the fixtures pass green.
