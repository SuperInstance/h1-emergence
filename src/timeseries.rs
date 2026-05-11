use crate::graph::Graph;
use crate::types::{EmergenceEvent, EmergenceResult, H1Config, H1Snapshot};
use std::collections::HashMap;

/// Sliding window for tracking H1 evolution over time
#[derive(Debug, Clone)]
pub struct H1Tracker {
    pub config: H1Config,
    /// Timestamped snapshots
    pub snapshots: Vec<H1Snapshot>,
    /// Events detected
    pub events: Vec<EmergenceEvent>,
    /// Internal mapping of cycle fingerprints for persistence tracking
    cycle_persistence: HashMap<u64, (u64, usize)>, // cycle_fingerprint -> (first_detected, persist_count)
    current_timestamp: u64,
}

impl H1Tracker {
    pub fn new(config: H1Config) -> Self {
        Self {
            config,
            snapshots: Vec::new(),
            events: Vec::new(),
            cycle_persistence: HashMap::new(),
            current_timestamp: 0,
        }
    }

    /// Analyze a graph and track H1 evolution
    pub fn analyze(&mut self, graph: &Graph) -> EmergenceResult {
        self.current_timestamp += 1;
        let result = graph.analyze();
        let snapshot = H1Snapshot::new(self.current_timestamp, &result);
        self.snapshots.push(snapshot);

        if result.detected {
            // Track cycle persistence
            let mut persistent_cycles = Vec::new();
            for cycle in &result.cycles {
                let fingerprint = cycle_fingerprint(&cycle.nodes);
                let entry = self.cycle_persistence.entry(fingerprint).or_insert((
                    self.current_timestamp,
                    0,
                ));
                entry.1 += 1;

                let mut tracked_cycle = cycle.clone();
                tracked_cycle.first_detected = entry.0;
                tracked_cycle.is_persistent = entry.1 > 1;
                persistent_cycles.push(tracked_cycle);
            }

            self.events.push(EmergenceEvent {
                timestamp: self.current_timestamp,
                betti_1: result.betti_1,
                emergence_severity: result.emergence_severity,
                cycles: persistent_cycles,
            });
        }

        result
    }

    /// Get recent events (within last N timestamps)
    pub fn recent_events(&self, n: usize) -> Vec<&EmergenceEvent> {
        self.events.iter().rev().take(n).collect()
    }

    /// Get all snapshots
    pub fn get_history(&self) -> &[H1Snapshot] {
        &self.snapshots
    }

    /// Check if emergence has been sustained over recent windows
    pub fn sustained_emergence(&self, window: usize) -> bool {
        if self.events.len() < window {
            return false;
        }
        self.events.iter().rev().take(window).count() == window
    }

    /// Get the current tracking state as a summary string
    pub fn summary(&self) -> String {
        let total_events = self.events.len();
        let total_snapshots = self.snapshots.len();
        let persistent_cycles = self
            .cycle_persistence
            .values()
            .filter(|(_, count)| *count > 1)
            .count();

        format!(
            "H1Tracker: {} snapshots, {} emergence events, {} persistent cycles",
            total_snapshots, total_events, persistent_cycles
        )
    }
}

/// Compute a fingerprint for a cycle (sorted multi-set of edges)
fn cycle_fingerprint(nodes: &[u64]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut edges: Vec<(u64, u64)> = Vec::new();
    for i in 0..nodes.len() {
        let u = nodes[i];
        let v = nodes[(i + 1) % nodes.len()];
        edges.push(if u <= v { (u, v) } else { (v, u) });
    }
    edges.sort();

    let mut hasher = DefaultHasher::new();
    edges.hash(&mut hasher);
    hasher.finish()
}

/// Parse a time-series file of graph snapshots.
/// Each line is a JSON graph snapshot or edge list.
pub fn parse_timeseries_file(path: &str) -> Result<Vec<Graph>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path, e))?;

    let mut graphs = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Try parsing as JSON edge list
        if line.starts_with('[') {
            let edges: Vec<[u64; 2]> =
                serde_json::from_str(line).map_err(|e| format!("JSON parse error: {}", e))?;
            let mut g = Graph::new();
            for [u, v] in edges {
                g.add_edge(u, v);
            }
            graphs.push(g);
        } else {
            // Try edge string format "1-2,2-3,3-1"
            let g = Graph::from_edge_string(line);
            if g.vertex_count() > 0 {
                graphs.push(g);
            }
        }
    }

    Ok(graphs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliding_window_detection() {
        let config = H1Config::default();
        let mut tracker = H1Tracker::new(config);

        // First observation: tree (no emergence)
        let g1 = Graph::from_edge_string("1-2,2-3,3-4");
        let r1 = tracker.analyze(&g1);
        assert!(!r1.detected);

        // Second observation: add edge making a cycle (V=4, E=4, C=1 → β1=1, V-2=2 → no emergence)
        let g2 = Graph::from_edge_string("1-2,2-3,3-4,4-1");
        let r2 = tracker.analyze(&g2);
        assert!(!r2.detected, "Single cycle should not trigger emergence");

        // Third observation: full K4 (V=4, E=6, C=1 → β1=3 > V-2=2 → emergence!)
        let g3 = Graph::from_edge_string("1-2,1-3,1-4,2-3,2-4,3-4");
        let r3 = tracker.analyze(&g3);
        assert!(r3.detected, "K4 should trigger emergence");

        // Should have event tracked
        assert_eq!(tracker.events.len(), 1);
    }

    #[test]
    fn test_persistence_scoring() {
        let config = H1Config {
            track_evolution: true,
            ..Default::default()
        };
        let mut tracker = H1Tracker::new(config);

        let k4 = Graph::from_edge_string("1-2,1-3,1-4,2-3,2-4,3-4");

        // Detect same cycles twice
        tracker.analyze(&k4);
        tracker.analyze(&k4);

        // Should have persistent cycles
        let last_event = tracker.events.last().unwrap();
        let persistent_count = last_event.cycles.iter().filter(|c| c.is_persistent).count();
        assert!(
            persistent_count > 0,
            "Repeated cycles should be marked persistent"
        );

        assert!(tracker.sustained_emergence(2));
    }
}
