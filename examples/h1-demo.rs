/// H1 Emergence Demo — Track H1 evolution in a growing random graph.
///
/// Generates a random graph by iteratively adding random edges.
/// Tracks β1, the Laman threshold (V-2), and emergence events in real time.
/// Shows the moment of emergence when β1 crosses V-2.
use h1_emergence::graph::Graph;
use h1_emergence::timeseries::H1Tracker;
use h1_emergence::types::{EmergenceResult, H1Config};
use rand::Rng;
use std::collections::HashSet;

const MAX_VERTICES: u64 = 12;
const MAX_EDGES: usize = 40;

fn main() {
    println!("=== H1 Emergence Demo ===");
    println!("Building a random graph with {} vertices, tracking H1 evolution...\n", MAX_VERTICES);

    // Setup
    let mut rng = rand::thread_rng();
    let mut graph = Graph::new();
    let config = H1Config {
        track_evolution: true,
        window_size: MAX_EDGES,
        ..Default::default()
    };
    let mut tracker = H1Tracker::new(config);
    let mut added_edges: HashSet<(u64, u64)> = HashSet::new();

    // Phase 1: build a spanning tree first (minimally rigid)
    println!("Phase 1: Building spanning tree...");
    for v in 2..=MAX_VERTICES {
        let target = rng.gen_range(1..v);
        graph.add_edge(target, v);
        added_edges.insert((target.min(v), target.max(v)));
        let result = graph.analyze();
        print_step(&tracker, &result, "tree");
    }

    // Phase 2: keep adding random edges until no more can be added
    println!("\nPhase 2: Adding random edges...");
    let max_possible_edges = MAX_VERTICES as usize * (MAX_VERTICES as usize - 1) / 2;

    while added_edges.len() < max_possible_edges && added_edges.len() < MAX_EDGES {
        let u = rng.gen_range(1..=MAX_VERTICES);
        let v = {
            let mut v = rng.gen_range(1..=MAX_VERTICES);
            while v == u {
                v = rng.gen_range(1..=MAX_VERTICES);
            }
            v
        };
        let edge = (u.min(v), u.max(v));

        if added_edges.contains(&edge) {
            continue;
        }

        added_edges.insert(edge);
        graph.add_edge(u, v);
        let result = tracker.analyze(&graph);
        let phase = if result.detected { "🚨 EMERGENCE" } else { "normal" };
        print_step(&tracker, &result, phase);

        // Check if we'd be comparing old V-2 when vertices changed — but vertices are fixed here
        // so this is fine
    }

    // Summary
    println!("\n=== Demo Complete ===");
    println!("Total vertices: {}", MAX_VERTICES);
    println!("Total edges: {}", graph.edge_count());
    println!("Max possible edges (K_{}): {}", MAX_VERTICES, max_possible_edges);
    println!("{}", tracker.summary());

    let total_emergence = tracker.events.len();
    if total_emergence > 0 {
        let final_emergence = tracker.events.last().unwrap();
        println!(
            "\n🚨 {} emergence events detected.",
            total_emergence
        );
        println!("Final β1: {}", final_emergence.betti_1);
        println!(
            "Peak emergence severity: {:.4}",
            tracker
                .snapshots
                .iter()
                .map(|s| s.emergence_severity)
                .fold(f64::NEG_INFINITY, f64::max)
        );

        // Compare with simple thresholding (just β1 > 0)
        let simple_threshold_count = tracker
            .snapshots
            .iter()
            .filter(|s| s.betti_1 > 0)
            .count();
        println!("\n--- Comparison ---");
        println!(
            "Simple threshold (β1 > 0):     {} alerts ({}% of snapshots)",
            simple_threshold_count,
            simple_threshold_count * 100 / tracker.snapshots.len().max(1)
        );
        println!(
            "H1 emergence (β1 > V-2):       {} events",
            total_emergence
        );
        println!(
            "Reduction: {:.1}x fewer false positives",
            if total_emergence > 0 {
                simple_threshold_count as f64 / total_emergence as f64
            } else {
                simple_threshold_count as f64
            }
        );
    } else {
        println!("\nℹ️  No emergence detected. Graph stayed flexible.");
        println!("Final β1 = {}", graph.betti_1());
        println!("V-2 = {}", MAX_VERTICES.saturating_sub(2));
    }
}

fn print_step(tracker: &H1Tracker, result: &EmergenceResult, phase: &str) {
    let step = tracker.get_history().len();
    print!("[{:2}] {}: V={}, E={}, C={}, β1={}, V-2={}, sev={:.4}",
        step, phase, result.vertices, result.edges, result.components,
        result.betti_1, result.v_minus_2, result.emergence_severity);

    if phase == "🚨 EMERGENCE" {
        let persistent = result.cycles.iter().filter(|c| c.is_persistent).count();
        if persistent > 0 {
            print!(" ({} persistent cycles)", persistent);
        } else {
            print!(" ({} new cycles)", result.cycles.len());
        }
    }
    println!();
}
