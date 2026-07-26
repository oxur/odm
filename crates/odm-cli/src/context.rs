//! Current-project/arc context, persisted under `<store>/.odm/context.json`.
//!
//! `use project X` / `use arc X` record a selection here so later commands need
//! not repeat `--project`/`--arc`; `context` reads it back. This is CLI state,
//! not a node — it lives outside `nodes/` and is not part of the source graph.
//!
//! It is keyed to the **store**, not the invocation directory, because it names
//! node ids: a selection is only meaningful against the corpus that contains
//! those ids, so it must travel with the corpus (as `.odm/index` already does).
//!
//! The path is therefore derived from the [`Store`] handle rather than taken as
//! a root, so a caller cannot pass the invocation directory by mistake. It could
//! before: `use`/`context` were given the store root while `orient` was given
//! the invocation root, which was invisible for as long as the two were the same
//! directory — and became "`use` succeeds, `orient` reports no current arc" the
//! moment odm's own store moved into a worktree (RH C-5).

use std::fs;
use std::path::PathBuf;

use anyhow::Context as _;
use odm_core::Id;
use odm_store::Store;
use serde::{Deserialize, Serialize};

const CONTEXT_DIR: &str = ".odm";
const CONTEXT_FILE: &str = "context.json";

/// The current selection of project and/or arc.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Context {
    /// The current project node, if one is selected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<Id>,
    /// The current arc node, if one is selected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arc: Option<Id>,
}

impl Context {
    fn path(store: &Store) -> PathBuf {
        store.root().join(CONTEXT_DIR).join(CONTEXT_FILE)
    }

    /// Loads the context belonging to `store`. A missing file is not an error —
    /// it yields an empty context.
    pub fn load(store: &Store) -> anyhow::Result<Self> {
        let path = Self::path(store);
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(&path)
            .with_context(|| format!("reading context file {}", path.display()))?;
        serde_json::from_str(&text)
            .with_context(|| format!("parsing context file {}", path.display()))
    }

    /// Persists the context under `<store>/.odm/`, creating the directory if
    /// needed.
    pub fn save(&self, store: &Store) -> anyhow::Result<()> {
        let dir = store.root().join(CONTEXT_DIR);
        fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
        let text = serde_json::to_string_pretty(self).context("serializing context")?;
        fs::write(Self::path(store), text).context("writing context file")
    }
}
