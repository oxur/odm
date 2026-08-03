//! The migration-fidelity primitives both importers share (ODD-0025 §2.1/§2.2,
//! arc-migration-fidelity slice03): body normalization, the hard body-hash
//! gate, and the `source` record both importers populate at build time.
//!
//! Neither importer transforms a body — `selfhost.rs` now imports the verbatim
//! `arc-plan.md`/`slice-doc.md`/`project-plan.md` content, and `mapping.rs`
//! already imported the ODD body verbatim. The gate is therefore, by
//! construction, an **invariant check**: it verifies the body about to be
//! persisted is exactly the body that was read, so a future change that
//! reintroduces a transformation (e.g. a synthesized heading) is caught
//! immediately rather than silently reproducing the 44-stub regression.

use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use odm_core::frontmatter::Source;
use sha2::{Digest as _, Sha256};

use crate::MigrateError;

/// The `source.normalization` value both importers record (ODD-0025 §2.1):
/// leading/trailing whitespace trimmed, then CRLF → LF line endings
/// normalized — nothing else. Any *internal* content change still fails the
/// hash gate; line endings are treated as a cross-platform checkout artifact,
/// not content.
pub const NORMALIZATION: &str = "trim+lf";

/// Normalizes a body per [`NORMALIZATION`]: CRLF → LF, then trim.
pub(crate) fn normalize_body(body: &str) -> String {
    body.replace("\r\n", "\n").trim().to_string()
}

/// The hex SHA-256 of `text`, for comparison and error messages (no hash is
/// ever stored on the node — ODD-0025 §2.1).
fn sha256_hex(text: &str) -> String {
    Sha256::digest(text.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

/// Verifies the hard body-hash gate (ODD-0025 §2.1):
/// `sha256(normalize(source)) == sha256(normalize(node))`. `context` labels
/// the node in the error (e.g. `"#1602 (slice)"`).
///
/// # Errors
///
/// [`MigrateError::BodyHashMismatch`] if the normalized bodies hash differently.
pub fn verify_body_hash(
    source_body: &str,
    node_body: &str,
    context: impl Into<String>,
) -> Result<(), MigrateError> {
    let source_hash = sha256_hex(&normalize_body(source_body));
    let node_hash = sha256_hex(&normalize_body(node_body));
    if source_hash != node_hash {
        return Err(MigrateError::BodyHashMismatch { context: context.into() });
    }
    Ok(())
}

/// The `source.migrated_by` value both importers record: this crate's own
/// name + version, so a pre-fix import stays queryable by tool version.
#[must_use]
pub fn migrated_by() -> String {
    concat!("odm-migrate/", env!("CARGO_PKG_VERSION")).to_string()
}

/// Whether a node body is a **stub** — ≤ 1 non-blank line (the lone
/// synthesized `# {name}` heading the old self-host importer wrote before
/// slice03 removed that transform). Shared between the coverage detector
/// (s01, [`crate::coverage`]) and the update-in-place repair op (s04,
/// [`crate::selfhost::repair`]) so both agree on exactly the same set —
/// "don't re-derive the stub predicate" (s04 ledger F-8).
#[must_use]
pub fn is_stub_body(body: &str) -> bool {
    body.lines().filter(|line| !line.trim().is_empty()).count() <= 1
}

/// Builds the `source` record (ODD-0025 §2.0/§2.2) a migrated node carries:
/// the source path(s), the source doc's class (odm-migrate's `DocClass`
/// vocabulary, e.g. `"arc-plan"`, `"slice-doc"`, `"odd"`), this module's
/// [`NORMALIZATION`], [`migrated_by`], and the date the migration ran.
#[must_use]
pub fn build_source(
    paths: Vec<std::path::PathBuf>,
    class: impl Into<String>,
    migrated_on: NaiveDate,
) -> Source {
    Source {
        paths,
        class: class.into(),
        normalization: Some(NORMALIZATION.to_string()),
        migrated_by: Some(migrated_by()),
        migrated_on: Some(migrated_on),
        synthesis: None,
        attestation: None,
        migrated_from: Vec::new(),
    }
}

/// The real `(created, updated)` dates for `source_path`, from `repo_root`'s
/// git history — RH F-20's mechanism
/// ([`odm_store::worktree::first_commit_date`]/`last_commit_date`, the same
/// `git log --diff-filter=A --reverse`/`git log -1` derivation
/// [`crate::replan::derive_plan`] already uses for `--replan`), now threaded
/// into the **creation** paths themselves (arc-migration-fidelity s10,
/// operator-identified: every import path stamped `created` as "the day the
/// import ran" instead of the file's real history — RH F-20 built the
/// mechanism but only wired it into the separate, manually-invoked
/// `--replan` re-derivation, never into `self_host`/`migrate`/the artifact
/// and note minters themselves).
///
/// Falls back to `fallback` for either date git has no record of — an
/// untracked fixture (tests), or a path outside `repo_root`'s history —
/// rather than erroring: a missing git record is not a fidelity violation,
/// it is the honest absence of a fact `verify_body_hash`-style hard-gating
/// would be the wrong tool for (there is nothing to gate against).
#[must_use]
pub fn git_derived_dates(
    repo_root: &Path,
    source_path: &Path,
    fallback: NaiveDate,
) -> (NaiveDate, NaiveDate) {
    let relative = source_path.strip_prefix(repo_root).unwrap_or(source_path);
    let parse =
        |text: Option<String>| text.and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());
    let created = odm_store::worktree::first_commit_date(repo_root, relative)
        .ok()
        .and_then(parse)
        .unwrap_or(fallback);
    let updated = odm_store::worktree::last_commit_date(repo_root, relative)
        .ok()
        .and_then(parse)
        .unwrap_or(fallback);
    (created, updated)
}

/// Finds the git toplevel worktree root containing `start` — the anchor a
/// plan tree's storable paths relativize against (ODD-0025 §2.2,
/// arc-migration-fidelity s08 F-1). Walks up looking for a `.git` entry; a
/// **linked worktree's `.git` is a file**, not a directory (it holds a
/// `gitdir: …` pointer), so this checks existence, not `is_dir()` — the same
/// shape `odm_store::config`'s private `repo_root` helper uses, reimplemented
/// here rather than exposed cross-crate, since this concern is local to how
/// `source.paths` gets stored.
///
/// `start` must be absolute; a relative `start` would make the `.exists()`
/// probes implicitly (and fragilely) cwd-relative.
#[must_use]
pub fn git_toplevel(start: &Path) -> Option<PathBuf> {
    let mut dir = Some(start);
    while let Some(d) = dir {
        if d.join(".git").exists() {
            return Some(d.to_path_buf());
        }
        dir = d.parent();
    }
    None
}

/// The anchor `plan_root`'s storable paths relativize against: the git
/// toplevel of the worktree containing it (**never** a multi-worktree
/// layout's superproject root — anchoring there would bake in
/// `.worktrees/1.0.x/`, s08's whole reason for existing), or `plan_root`
/// itself if it isn't inside a git repo at all (a plan tree used standalone).
#[must_use]
pub fn anchor_for(plan_root: &Path) -> PathBuf {
    git_toplevel(plan_root).unwrap_or_else(|| plan_root.to_path_buf())
}

/// Relativizes `path` to `anchor` into **canonical** form (ODD-0025 §2.2,
/// arc-migration-fidelity s08 F-1/F-2): forward slashes, no leading `/`, no
/// `./`/`..` components, no trailing slash — identical regardless of whether
/// `path` arrived absolute or already relative, and regardless of how many
/// `.`/`..`/trailing-slash decorations the original argument carried (`Path`'s
/// own component model already normalizes those away; only `..` needs
/// explicit handling here, via popping the last collected part).
///
/// Case is preserved, **not** folded (s08 F-2's decided case rule): paths are
/// always derived from an actual directory listing (`discover`/`enumerate_docs`
/// walk `read_dir`/`WalkDir`, never echo a user-typed argument's spelling), so
/// the stored case always matches the filesystem's — and therefore git's —
/// tracked case, on any OS, without needing a filesystem-dependent fold that
/// would risk collapsing two distinctly-named files on a general (non-macOS)
/// checkout.
///
/// This is the sole seam through which a path becomes a stored `source.paths`
/// entry or a `by_source` lookup key; paired with [`resolve_from_anchor`] for
/// the opposite direction (s08 F-3) — one anchor, both directions, so they
/// cannot drift.
#[must_use]
pub fn relativize(anchor: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(anchor).unwrap_or(path);
    let mut parts: Vec<String> = Vec::new();
    for component in relative.components() {
        match component {
            std::path::Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            std::path::Component::ParentDir => {
                parts.pop();
            }
            std::path::Component::CurDir
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => {}
        }
    }
    parts.join("/")
}

/// Resolves a stored (canonical, anchor-relative) `source.paths` entry back
/// to an absolute filesystem path — the exact inverse of [`relativize`],
/// through the same anchor (s08 F-3), for [`crate::selfhost::reconcile_source`]'s
/// read. Also correctly resolves a still-absolute legacy entry (pre-s08):
/// [`Path::join`] replaces the whole path outright when the joined-on
/// argument is itself absolute, so no separate branch is needed for the
/// transition case.
#[must_use]
pub fn resolve_from_anchor(anchor: &Path, stored: &str) -> PathBuf {
    anchor.join(stored)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----- s10 F-H: created/updated come from git history, not "today" ------

    /// A `TempDir` git repo with `relative` committed at `created_on`, then
    /// touched again (a second commit, same content is fine — git still logs
    /// it) at `updated_on`. Uses `GIT_AUTHOR_DATE`/`GIT_COMMITTER_DATE` for a
    /// deterministic, controlled commit date — no dependency on wall-clock
    /// time, which `Date.now()`-style flakiness would otherwise introduce.
    fn repo_with_dated_commits(
        relative: &str,
        created_on: &str,
        updated_on: &str,
    ) -> tempfile::TempDir {
        let dir = tempfile::TempDir::new().unwrap();
        let root = dir.path();
        let run = |args: &[&str], date: &str| {
            let status = std::process::Command::new("git")
                .args(args)
                .current_dir(root)
                .env("GIT_AUTHOR_DATE", format!("{date}T12:00:00"))
                .env("GIT_COMMITTER_DATE", format!("{date}T12:00:00"))
                .status()
                .expect("git available");
            assert!(status.success(), "git {args:?} failed");
        };
        run(&["init", "-q"], created_on);
        run(&["config", "user.email", "test@example.com"], created_on);
        run(&["config", "user.name", "Test"], created_on);

        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "first version\n").unwrap();
        run(&["add", relative], created_on);
        run(&["commit", "-q", "-m", "add"], created_on);

        std::fs::write(&path, "second version\n").unwrap();
        run(&["add", relative], updated_on);
        run(&["commit", "-q", "-m", "touch"], updated_on);

        dir
    }

    #[test]
    fn git_derived_dates_reads_real_first_and_last_commit_dates() {
        let repo = repo_with_dated_commits("docs/x.md", "2020-01-15", "2023-06-30");
        let fallback = NaiveDate::from_ymd_opt(2026, 7, 29).unwrap();

        let (created, updated) =
            git_derived_dates(repo.path(), &repo.path().join("docs/x.md"), fallback);

        assert_eq!(created, NaiveDate::from_ymd_opt(2020, 1, 15).unwrap());
        assert_eq!(updated, NaiveDate::from_ymd_opt(2023, 6, 30).unwrap());
    }

    #[test]
    fn git_derived_dates_falls_back_outside_any_repo() {
        let dir = tempfile::TempDir::new().unwrap();
        let fallback = NaiveDate::from_ymd_opt(2026, 7, 29).unwrap();

        let (created, updated) =
            git_derived_dates(dir.path(), &dir.path().join("untracked.md"), fallback);

        assert_eq!(created, fallback, "no git history — falls back, doesn't error");
        assert_eq!(updated, fallback);
    }

    #[test]
    fn verify_body_hash_passes_on_identical_bodies() {
        assert!(verify_body_hash("# Title\n\nbody\n", "# Title\n\nbody\n", "#1").is_ok());
    }

    #[test]
    fn verify_body_hash_passes_across_crlf_and_trim_differences() {
        // Only line-ending + surrounding-whitespace differences — still a pass.
        assert!(verify_body_hash("# Title\r\n\r\nbody\r\n", "  # Title\n\nbody\n  ", "#1").is_ok());
    }

    #[test]
    fn verify_body_hash_fails_on_an_internal_change() {
        let err =
            verify_body_hash("# Title\n\noriginal\n", "# Title\n\nmutated\n", "#1").unwrap_err();
        assert!(matches!(err, MigrateError::BodyHashMismatch { context } if context == "#1"));
    }

    #[test]
    fn is_stub_body_matches_lone_heading_only() {
        assert!(is_stub_body("# Title\n"));
        assert!(is_stub_body("# Title\n\n   \n"), "whitespace-only lines don't count");
        assert!(is_stub_body(""));
        assert!(!is_stub_body("# Title\n\nReal content here.\n"));
    }

    #[test]
    fn build_source_records_the_normalization_and_tool_version() {
        let source = build_source(
            vec!["docs/design-v1.0.0/arc01-alpha/arc-plan.md".into()],
            "arc-plan",
            NaiveDate::from_ymd_opt(2026, 7, 27).unwrap(),
        );
        assert_eq!(source.normalization.as_deref(), Some("trim+lf"));
        assert!(source.migrated_by.as_deref().is_some_and(|v| v.starts_with("odm-migrate/")));
        assert_eq!(source.class, "arc-plan");
    }

    // ----- s08 F-1/F-2/F-3: anchor + relativize + resolve -----------------

    #[test]
    fn git_toplevel_finds_a_worktrees_own_git_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        let toplevel = tmp.path().join("worktree-root");
        let nested = toplevel.join("docs/design-v1.0.0/arc01-alpha");
        std::fs::create_dir_all(&nested).unwrap();
        // A linked worktree's `.git` is a FILE, not a directory.
        std::fs::write(toplevel.join(".git"), "gitdir: /elsewhere/.git/worktrees/x\n").unwrap();

        assert_eq!(git_toplevel(&nested), Some(toplevel.clone()));
        assert_eq!(git_toplevel(&toplevel), Some(toplevel));
    }

    #[test]
    fn git_toplevel_is_none_outside_any_repo() {
        let tmp = tempfile::TempDir::new().unwrap();
        let nested = tmp.path().join("no-git-here");
        std::fs::create_dir_all(&nested).unwrap();
        assert_eq!(git_toplevel(&nested), None);
    }

    #[test]
    fn anchor_for_falls_back_to_plan_root_without_a_repo() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert_eq!(anchor_for(tmp.path()), tmp.path());
    }

    #[test]
    fn relativize_strips_the_anchor_and_normalizes() {
        let anchor = Path::new("/repo/worktree");
        assert_eq!(
            relativize(anchor, Path::new("/repo/worktree/docs/design-v1.0.0/arc-plan.md")),
            "docs/design-v1.0.0/arc-plan.md"
        );
    }

    #[test]
    fn relativize_is_invariant_to_arg_spelling() {
        let anchor = Path::new("/repo/worktree");
        // Absolute, already-relative, a `./`-prefixed relative, and one with
        // an internal `..` all collapse to the identical canonical string.
        let absolute = Path::new("/repo/worktree/docs/design-v1.0.0/arc-plan.md");
        let already_relative = Path::new("docs/design-v1.0.0/arc-plan.md");
        let dot_prefixed = Path::new("./docs/design-v1.0.0/arc-plan.md");
        let with_dotdot = Path::new("docs/design-v1.0.0/other/../arc-plan.md");

        let expected = "docs/design-v1.0.0/arc-plan.md";
        assert_eq!(relativize(anchor, absolute), expected);
        assert_eq!(relativize(anchor, already_relative), expected);
        assert_eq!(relativize(anchor, dot_prefixed), expected);
        assert_eq!(relativize(anchor, with_dotdot), expected);
    }

    #[test]
    fn relativize_and_resolve_from_anchor_round_trip() {
        let anchor = Path::new("/repo/worktree");
        let original = anchor.join("docs/design-v1.0.0/arc-plan.md");
        let relative = relativize(anchor, &original);
        assert_eq!(resolve_from_anchor(anchor, &relative), original);
    }

    #[test]
    fn resolve_from_anchor_handles_a_legacy_absolute_stored_path() {
        // A pre-s08 node's `source.paths` entry is still absolute; resolving
        // it "from" any anchor must still yield that same absolute path
        // (`Path::join` replaces outright on an absolute second argument) —
        // the property the transition rewrite (F-4) leans on.
        let anchor = Path::new("/repo/worktree");
        let legacy_absolute =
            "/Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design-v1.0.0/arc-plan.md";
        assert_eq!(resolve_from_anchor(anchor, legacy_absolute), Path::new(legacy_absolute));
    }
}
