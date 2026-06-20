//! `odm-graph` — pure DAG/tree engine over abstract ids.
//!
//! Edges, topological sort, Kahn cycle detection, tears, ready/blocked/path,
//! and staleness live here. This is a stub for the v1.0.0 workspace skeleton
//! (slice 01); the engine is implemented in Arc 02.

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        assert_eq!(env!("CARGO_PKG_NAME"), "odm-graph");
    }
}
