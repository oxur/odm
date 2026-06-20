//! `odm-core` — the odm domain model.
//!
//! Node types, ULID identity, the frontmatter schema, edge & gate semantics,
//! satisfaction, link-integrity, and the rollup model live here. This is a stub
//! for the v1.0.0 workspace skeleton (slice 01); identity arrives in slice 02
//! and the schema in slice 03.

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        assert_eq!(env!("CARGO_PKG_NAME"), "odm-core");
    }
}
