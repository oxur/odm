//! `odm-store` — persistence for odm.
//!
//! The `nodes/YYYY/MM/<ULID>.md` layout, atomic writes, git integration (via
//! `gix`), `odm.toml`, and scan/load live here. This is a stub for the v1.0.0
//! workspace skeleton (slice 01); the store lands in slice 04.

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        assert_eq!(env!("CARGO_PKG_NAME"), "odm-store");
    }
}
