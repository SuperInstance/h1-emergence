use serde::{Deserialize, Serialize};

/// Configuration for H1 emergence detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H1Config {
    /// Sliding window for time-series analysis
    pub window_size: usize,
    /// β1 / (V-2) ratio to trigger emergence alert
    pub threshold_ratio: f64,
    /// Minimum vertices for meaningful analysis
    pub min_graph_size: usize,
    /// Track H1 changes over time
    pub track_evolution: bool,
}

impl Default for H1Config {
    fn default() -> Self {
        Self {
            window_size: 100,
            threshold_ratio: 1.0,
            min_graph_size: 3,
            track_evolution: true,
        }
    }
}

/// Result of emergence detection on a graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergenceResult {
    /// Whether emergence was detected
    pub detected: bool,
    /// H1 dimension = E - V + C
    pub betti_1: usize,
    /// Number of vertices
    pub vertices: usize,
    /// Number of edges
    pub edges: usize,
    /// Number of connected components
    pub components: usize,
    /// Laman threshold = V - 2
    pub v_minus_2: usize,
    /// β1 / (V-2) - 1 (0 = at threshold, >0 = emergence)
    pub emergence_severity: f64,
    /// E >= 2V - 3
    pub laman_rigid: bool,
    /// β1 > V - 2
    pub over_constrained: bool,
    /// Detected emergent cycles
    pub cycles: Vec<Cycle>,
}

impl EmergenceResult {
    /// Create a new emergence result for the given graph state.
    pub fn new(vertices: usize, edges: usize, components: usize, cycles: Vec<Cycle>) -> Self {
        let v_minus_2 = if vertices >= 2 { vertices - 2 } else { 0 };
        let betti_1 = if edges >= vertices {
            edges - vertices + components
        } else {
            // If fewer edges than vertices, β1 = E - V + C, which can be 0 or negative
            // For under-constrained graphs, we treat negative as 0.
            let raw = edges as isize - vertices as isize + components as isize;
            if raw > 0 { raw as usize } else { 0 }
        };
        // This formulation works for all cases: β1 = E - V + C
        // But we also need to handle the edge case where edges < vertices
        // β1 should never be reported as negative, but the formula holds
        // mathematically. For under-constrained graphs, β1 = 0.
        let emergence_severity = if v_minus_2 > 0 {
            (betti_1 as f64 / v_minus_2 as f64) - 1.0
        } else {
            -1.0
        };
        let laman_rigid = edges >= 2 * vertices - 3;
        let over_constrained = betti_1 > v_minus_2;
        let detected = over_constrained;

        Self {
            detected,
            betti_1,
            vertices,
            edges,
            components,
            v_minus_2,
            emergence_severity,
            laman_rigid,
            over_constrained,
            cycles,
        }
    }
}

/// A cycle detected in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cycle {
    /// Nodes in the cycle (in order)
    pub nodes: Vec<u64>,
    /// Length of the cycle
    pub length: usize,
    /// Emergence magnitude in this cycle (currently equals length-normalized betti contribution)
    pub holonomy: f64,
    /// Timestamp of first detection
    pub first_detected: u64,
    /// Whether the cycle survived across multiple windows
    pub is_persistent: bool,
}

/// A snapshot of H1 state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H1Snapshot {
    /// Timestamp of this snapshot
    pub timestamp: u64,
    /// H1 dimension
    pub betti_1: usize,
    /// Whether emergence was detected
    pub emergence_detected: bool,
    /// Emergence severity
    pub emergence_severity: f64,
}

impl H1Snapshot {
    pub fn new(timestamp: u64, result: &EmergenceResult) -> Self {
        Self {
            timestamp,
            betti_1: result.betti_1,
            emergence_detected: result.detected,
            emergence_severity: result.emergence_severity,
        }
    }
}

/// Event representing an emergence detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergenceEvent {
    pub timestamp: u64,
    pub betti_1: usize,
    pub emergence_severity: f64,
    pub cycles: Vec<Cycle>,
}
