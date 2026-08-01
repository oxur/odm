//! Integration tests for dev-doc mint-all (arc-migration-fidelity slice10,
//! operator decision 2026-07-28). Test names carry the substring `notes_`.

use std::collections::HashMap;
use std::path::Path;

use odm_core::NodeType;
use odm_core::frontmatter::Document;
use odm_migrate::Mode;
use odm_migrate::notes::mint_notes;
use odm_store::Store;
use tempfile::TempDir;

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn notes_by_path(store: &Store) -> HashMap<String, Document> {
    store
        .load_all()
        .expect("load_all")
        .into_iter()
        .filter(|d| d.frontmatter().node_type() == NodeType::Note)
        .map(|d| {
            let relative = d.frontmatter().source().expect("note carries source").paths[0].clone();
            (relative.display().to_string(), d)
        })
        .collect()
}

// ----- mint-all, uncontained, tagged by immediate subdirectory -------------

#[test]
fn notes_mint_all_uncontained_and_tagged() {
    let dev = TempDir::new().unwrap();
    write(dev.path(), "0001-direct.md", "# Direct note\nbody\n");
    write(dev.path(), "research/0001-survey.md", "# Survey\nbody\n");
    write(dev.path(), "index.md", "# Index\n"); // excluded infra file

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let report = mint_notes(&store, dev.path(), Mode::Commit).expect("mint");

    assert_eq!(report.minted_count(), 2, "index.md is excluded, not minted");

    let by_path = notes_by_path(&store);
    let direct = &by_path["0001-direct.md"];
    assert_eq!(direct.frontmatter().node_type(), NodeType::Note);
    assert!(direct.frontmatter().edges().part_of.is_none(), "notes are uncontained");
    assert!(direct.frontmatter().tags().is_empty(), "a direct child gets no tag");

    let nested = &by_path["research/0001-survey.md"];
    assert_eq!(nested.frontmatter().tags(), ["research"], "subdirectory becomes a tag");
    assert!(nested.frontmatter().edges().part_of.is_none());

    assert!(!by_path.contains_key("index.md"), "index.md was never minted");
}

// ----- idempotent + dry-run --------------------------------------------------

#[test]
fn notes_mint_is_idempotent() {
    let dev = TempDir::new().unwrap();
    write(dev.path(), "0001-a.md", "# A\nbody\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let first = mint_notes(&store, dev.path(), Mode::Commit).expect("first mint");
    assert_eq!(first.minted_count(), 1);
    let n = store.load_all().unwrap().len();

    let second = mint_notes(&store, dev.path(), Mode::Commit).expect("second mint");
    assert_eq!(second.minted_count(), 0, "already-covered docs are not re-minted");
    assert_eq!(store.load_all().unwrap().len(), n, "no duplicate nodes on disk");
}

#[test]
fn notes_mint_dry_run_writes_nothing() {
    let dev = TempDir::new().unwrap();
    write(dev.path(), "0001-a.md", "# A\nbody\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let report = mint_notes(&store, dev.path(), Mode::DryRun).expect("dry-run mint");
    assert!(report.dry_run);
    assert_eq!(report.minted_count(), 1);
    assert!(store.load_all().unwrap().is_empty(), "dry-run wrote nothing");
}

// ----- s15 F-4: a drifted, already-minted note re-snapshots in place -------

#[test]
fn notes_mint_reconciles_a_drifted_already_minted_note() {
    let dev = TempDir::new().unwrap();
    write(dev.path(), "0001-a.md", "# A\n\nOriginal body.\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    let first = mint_notes(&store, dev.path(), Mode::Commit).expect("first mint");
    assert_eq!(first.minted_count(), 1);
    let original_id = notes_by_path(&store)["0001-a.md"].frontmatter().id();

    write(dev.path(), "0001-a.md", "# A\n\nAmended body — drifted since the mint.\n");
    let second = mint_notes(&store, dev.path(), Mode::Commit).expect("second mint");
    assert_eq!(second.minted_count(), 0, "already covered — not re-minted");
    assert_eq!(second.reconciled_count(), 1, "drifted body is reconciled, not skipped");

    let by_path = notes_by_path(&store);
    let note = &by_path["0001-a.md"];
    assert_eq!(note.frontmatter().id(), original_id, "identity preserved");
    assert_eq!(note.body(), "# A\n\nAmended body — drifted since the mint.\n");

    // Un-drifted + re-run: a no-op.
    let third = mint_notes(&store, dev.path(), Mode::Commit).expect("third mint");
    assert_eq!(third.minted_count(), 0);
    assert_eq!(third.reconciled_count(), 0, "already faithful — idempotent no-op");
}

#[test]
fn notes_mint_reconcile_dry_run_writes_nothing() {
    let dev = TempDir::new().unwrap();
    write(dev.path(), "0001-a.md", "# A\n\nOriginal body.\n");

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    mint_notes(&store, dev.path(), Mode::Commit).expect("first mint");
    write(dev.path(), "0001-a.md", "# A\n\nAmended.\n");

    let before = notes_by_path(&store)["0001-a.md"].body().to_string();
    let dry = mint_notes(&store, dev.path(), Mode::DryRun).expect("dry-run mint");
    assert_eq!(dry.reconciled_count(), 1, "the plan still lists what would be reconciled");
    let after = notes_by_path(&store)["0001-a.md"].body().to_string();
    assert_eq!(before, after, "dry-run wrote nothing");
}

// ----- body is verbatim 1:1, under the hard body-hash gate -------------------

#[test]
fn notes_body_is_verbatim_1to1() {
    let dev = TempDir::new().unwrap();
    let body = "# A dev note\n\nSome development history worth preserving.\n";
    write(dev.path(), "0001-a.md", body);

    let store_dir = TempDir::new().unwrap();
    let store = Store::open(store_dir.path());
    mint_notes(&store, dev.path(), Mode::Commit).expect("mint");

    let by_path = notes_by_path(&store);
    assert_eq!(by_path["0001-a.md"].body().trim(), body.trim());
}
