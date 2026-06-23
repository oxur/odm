//! Current-project/arc context, persisted under `<root>/.odm/context.json`.
//!
//! `use project X` / `use arc X` record a selection here so later commands need
//! not repeat `--project`/`--arc`; `context` reads it back. This is CLI state,
//! not a node — it lives outside `nodes/` and is not part of the source graph.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use odm_core::Id;
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
    fn path(root: &Path) -> PathBuf {
        root.join(CONTEXT_DIR).join(CONTEXT_FILE)
    }

    /// Loads the context for the store rooted at `root`. A missing file is not
    /// an error — it yields an empty context.
    pub fn load(root: &Path) -> anyhow::Result<Self> {
        let path = Self::path(root);
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(&path)
            .with_context(|| format!("reading context file {}", path.display()))?;
        serde_json::from_str(&text)
            .with_context(|| format!("parsing context file {}", path.display()))
    }

    /// Persists the context under `<root>/.odm/`, creating the directory if
    /// needed.
    pub fn save(&self, root: &Path) -> anyhow::Result<()> {
        let dir = root.join(CONTEXT_DIR);
        fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
        let text = serde_json::to_string_pretty(self).context("serializing context")?;
        fs::write(Self::path(root), text).context("writing context file")
    }
}
