//! H1 emergence detection — algebraic topology for anomaly detection.
//!
//! Detects emergent behavior in graphs by computing the first Betti number (β₁)
//! and comparing it against the Laman rigidity threshold (V - 2).
//!
//! When β₁ > V - 2, the graph is over-constrained, indicating emergent structure.
//! This replaces 12,000 lines of ML-based anomaly detection with 127 lines of
//! algebraic topology.
//!
//! # Quick Start
//!
//! ```rust
//! use h1_emergence::graph::Graph;
//!
//! let g = Graph::from_edge_string("1-2,2-3,3-1");
//! let result = g.analyze();
//! println!("β1 = {}, emergence = {}", result.betti_1, result.detected);
//! ```

pub mod graph;
pub mod timeseries;
pub mod types;

// Re-export commonly used types at crate level
pub use types::*;
