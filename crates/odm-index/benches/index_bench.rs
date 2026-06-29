//! Arc 04 capstone benchmark harness (slice08) — measures the index engine
//! end-to-end at 1k / 10k / 100k and prints a table (ODD-0014 §4).
//!
//! `harness = false`: this is a plain release-mode binary, not criterion — we
//! want whole-operation latencies at scale, not micro-bench statistics. Run:
//!
//! ```text
//! cargo bench -p odm-index                 # default scales 1k/10k/100k
//! cargo bench -p odm-index -- 1000 10000   # a subset (faster)
//! ```
//!
//! It reuses the real engine unchanged — [`build`] (cold), [`reconcile`] (warm),
//! [`Snapshot::load`] (decode), and the index→graph adapter + `odm-core`
//! graph/satisfaction for the consumer read — over a seeded [`synth`] corpus.
//! The durable claim is the **scaling** (warm ≪ cold; delta-cost flat; load
//! sub-second at 100k); the absolute ms are machine/toolchain context.

use std::path::Path;
use std::time::{Duration, Instant};

use odm_core::gates::GateSets;
use odm_core::graph::NodeGraph;
use odm_core::satisfaction::Satisfaction;
use odm_core::status::Evidence;
use odm_index::{
    IndexRecord, Snapshot, build, default_index_path, frontmatters_from_records, reconcile, synth,
};
use odm_store::Store;
use tempfile::TempDir;

/// The default scales when none are passed on the command line.
const DEFAULT_SCALES: &[usize] = &[1_000, 10_000, 100_000];

/// A fixed seed so every run measures the same corpus.
const SEED: u64 = 0x0DA0_BEEF_C0DE_0042;

/// A far-future index stamp (~year 3000) so every corpus file's mtime is **older
/// than the index** — the steady state of normal use (the index was written
/// after the last edit). This makes the warm sweep `lstat`-only, which is the
/// "scales with the delta, not the corpus" path we mean to measure. The racy
/// same-tick path (every file `mtime >= index_timestamp` ⇒ re-hash) is the
/// deliberate correctness fallback, exercised by slice03's tests — not the
/// warm-win measurement. (ODD-0014 §2.3/§3.2.)
const FUTURE_STAMP: i64 = 32_503_680_000;

/// Milliseconds from a `Duration`.
fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

/// Persists the index over `records` stamped in the far future, so the next warm
/// reconcile treats every unchanged file as non-racy clean (lstat-only).
fn stamp_future(index: &Path, records: Vec<IndexRecord>) {
    Snapshot::new(FUTURE_STAMP, records).persist(index).expect("persist future-stamped index");
}

fn main() {
    let scales: Vec<usize> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
    let scales: &[usize] = if scales.is_empty() { DEFAULT_SCALES } else { &scales };

    let gates = GateSets::from_toml_str(synth::GATE_CONFIG).expect("gate config");

    println!("# odm-index benchmark (slice08) — times in ms, index size in KB");
    println!(
        "{:>8} {:>9} {:>11} {:>11} {:>9} {:>9} {:>11}",
        "nodes", "cold", "warmNoChg", "warmDelta", "load", "idx_KB", "consumer"
    );

    for &n in scales {
        let dir = TempDir::new().expect("tempdir");
        let store = Store::open(dir.path());
        let ids = synth::generate_corpus(&store, n, SEED);
        let index = default_index_path(store.root());

        // 1. Cold build — full walk + parse + hash (O(corpus)).
        let t = Instant::now();
        let cold_snap = build(&store).expect("cold build");
        let cold = ms(t.elapsed());
        // Steady state: stamp the index after the corpus so the warm sweep is
        // lstat-only (not the racy re-hash fallback — see FUTURE_STAMP).
        stamp_future(&index, cold_snap.records);

        // 2. Warm reconcile, no change — the lstat sweep, no re-parse.
        let t = Instant::now();
        let r = reconcile(&store, &index).expect("warm no-change");
        let warm_nochg = ms(t.elapsed());
        assert!(!r.delta.is_changed(), "no-change reconcile must be clean");

        // 3. Warm reconcile, small delta — one changed file (delta-cost).
        let edited = store.path_of(ids[n / 2]);
        let mut bytes = std::fs::read(&edited).expect("read node");
        bytes.extend_from_slice(b"\nbenchmark edit\n");
        std::fs::write(&edited, &bytes).expect("write node");
        let t = Instant::now();
        let r = reconcile(&store, &index).expect("warm delta");
        let warm_delta = ms(t.elapsed());
        assert_eq!(r.delta.changed.len(), 1, "exactly one file changed");
        // The delta reconcile re-stamped to `now`; restore the steady state for
        // the load + consumer measurements below.
        stamp_future(&index, r.snapshot.records);

        // 4. Snapshot load (decode) + on-disk size.
        let t = Instant::now();
        let _ = Snapshot::load(&index).expect("load");
        let load = ms(t.elapsed());
        let idx_kb = std::fs::metadata(&index).expect("stat index").len() as f64 / 1024.0;

        // 5. Consumer read — reconcile → adapter → graph → satisfaction → next
        //    (the `odm next` path). Settles slice07's eager-rebuild question.
        let t = Instant::now();
        let snap = reconcile(&store, &index).expect("consumer reconcile").snapshot;
        let fms = frontmatters_from_records(&snap.records, &gates);
        let graph = NodeGraph::build(&fms);
        let sat = Satisfaction::compute(&fms, &gates, Evidence::Reproduced);
        let ready = graph.next(&sat);
        let consumer = ms(t.elapsed());
        std::hint::black_box(&ready);

        println!(
            "{n:>8} {cold:>9.1} {warm_nochg:>11.1} {warm_delta:>11.1} \
             {load:>9.1} {idx_kb:>9.1} {consumer:>11.1}"
        );
    }
}
