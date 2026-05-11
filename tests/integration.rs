use h1_emergence::graph::Graph;
use h1_emergence::timeseries::H1Tracker;
use h1_emergence::types::H1Config;
use std::process::Command;

#[test]
fn test_cli_analyze() {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "analyze",
            "--graph",
            "1-2,2-3,3-1",
        ])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("β1"));
    assert!(stdout.contains("Emergence detected"));
    assert!(stdout.contains("V-2"));
}

#[test]
fn test_cli_detect() {
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "detect", "--edges", "6", "--vertices", "4"])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // K4: V=4, E=6, C=1 → β1=3, V-2=2 → emergence
    assert!(stdout.contains("β1 (E - V + C):     3"));
    assert!(stdout.contains("EMERGENCE"));
}

#[test]
fn test_cli_analyze_json() {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            "analyze",
            "--graph",
            "1-2,2-3,3-1",
            "--json",
        ])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should output valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["betti_1"], 1);
    assert_eq!(parsed["vertices"], 3);
    assert_eq!(parsed["edges"], 3);
}

#[test]
fn test_sliding_window_integration() {
    let mut tracker = H1Tracker::new(H1Config::default());

    // Build a graph step by step and track H1
    let mut g = Graph::new();

    // Phase 1: tree → no cycles
    g.add_edge(1, 2);
    let r = tracker.analyze(&g);
    assert!(!r.detected);
    assert_eq!(r.betti_1, 0);

    g.add_edge(2, 3);
    let r = tracker.analyze(&g);
    assert!(!r.detected);
    assert_eq!(r.betti_1, 0);

    g.add_edge(3, 4);
    let r = tracker.analyze(&g);
    assert!(!r.detected);
    assert_eq!(r.betti_1, 0);

    // Phase 2: add cycle (square)
    g.add_edge(4, 1);
    let r = tracker.analyze(&g);
    // V=4, E=4, C=1 → β1=1, V-2=2 → no emergence
    assert!(!r.detected, "Single cycle should not trigger emergence");
    assert_eq!(r.betti_1, 1);

    // Phase 3: add diagonals → K4
    g.add_edge(1, 3);
    let r = tracker.analyze(&g);
    // V=4, E=5, C=1 → β1=2, V-2=2 → at threshold
    assert!(!r.detected, "β1 = V-2 should not trigger emergence (not >)");
    assert_eq!(r.betti_1, 2);

    g.add_edge(2, 4);
    let r = tracker.analyze(&g);
    // V=4, E=6, C=1 → β1=3, V-2=2 → emergence!
    assert!(r.detected, "K4 should trigger emergence");
    assert_eq!(r.betti_1, 3);

    // Check tracker state
    assert_eq!(tracker.get_history().len(), 6);
    assert!(tracker.events.len() >= 1);
}
