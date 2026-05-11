# H1 Emergence

**Replace 12,000-line ML anomaly detection with 127 lines of algebraic topology.**

[![Crates.io](https://img.shields.io/badge/crate-h1--emergence-blue)](https://crates.io/crates/h1-emergence)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)

A Rust library + CLI tool for detecting **emergence events** in graphs and time-series data using the first Betti number (β₁).

## Why?

Traditional anomaly detection requires:
- Thousands of lines of ML pipeline code
- Training data, labels, and feature engineering
- Model retraining when the data distribution shifts
- Opaque black-box decisions

H1 emergence needs:
- **One formula:** β₁ = E − V + C
- **One comparison:** β₁ > V − 2 (over-constrained = emergent)
- **127 lines** of actual topology code

## Quick Start

```bash
# Analyze a static graph
cargo run -- analyze --graph "1-2,2-3,3-1,1-4,2-4,3-4"

# Quick emergence check
cargo run -- detect --edges 6 --vertices 4

# Full topology report
cargo run -- topology --vertices 5 --edges 8

# Interactive watch mode
echo "1 2
2 3
3 1
1 4
2 4
3 4" | cargo run -- watch
```

## Library Usage

```rust
use h1_emergence::graph::Graph;

// Build a graph
let g = Graph::from_edge_string("1-2,2-3,3-1,1-4");

// Analyze
let result = g.analyze();
println!("β1 = {}", result.betti_1);           // H1 dimension
println!("V-2 = {}", result.v_minus_2);        // Laman threshold
println!("Emergence: {}", result.detected);    // β1 > V - 2?
```

## Time-Series Tracking

```rust
use h1_emergence::{graph::Graph, timeseries::H1Tracker, types::H1Config};

let config = H1Config::default();
let mut tracker = H1Tracker::new(config);

// Feed graph snapshots over time
for graph in graph_snapshots {
    let result = tracker.analyze(&graph);
    if result.detected {
        println!("🚨 Emergence at β1 = {}", result.betti_1);
    }
}
```

## What is H1 Emergence?

**H₁** (the first homology group) measures **independent cycles** in a graph.

- β₁ = 0: A tree (no cycles, minimally rigid)
- β₁ = 1: One cycle (a loop)
- β₁ > V−2: **Over-constrained** — the structure has more constraints than degrees of freedom. This is **emergence**.

### ASCII Example

```
Tree (β₁=0):            Cycle (β₁=1):          Emergent (β₁=3, V=4):
A---B---C---D           A---B---C               A---B
                        |       |               |\ /|
                        D---E---F               | X |
                                                |/ \|
                                                C---D
  V=4, E=3, C=1          V=6, E=6, C=1          V=4, E=6, C=1
  β₁ = 3-4+1 = 0         β₁ = 6-6+1 = 1         β₁ = 6-4+1 = 3
  E ≥ 2V-3? No           E ≥ 2V-3? Yes           β₁ > V-2? Yes! 🚨
```

β₁ = 3 > V−2 = 2. The K₄ graph is **over-constrained**. This structure has emergent rigidity — it can't flex without breaking edges.

## CLI Commands

| Command | Description |
|---------|-------------|
| `h1 analyze --graph "1-2,2-3"` | Analyze a static graph |
| `h1 analyze --graph "..." --json` | JSON output |
| `h1 stream --file data.json` | Analyze time-series snapshots |
| `h1 detect --edges N --vertices N` | Quick emergence check |
| `h1 topology --vertices N --edges N` | Full topology report |
| `h1 watch` | Interactive stdin watch mode |

## Real-World Applications

See [docs/APPLICATIONS.md](docs/APPLICATIONS.md) for detailed scenarios:
1. **Network monitoring** — detect routing anomalies
2. **Social network analysis** — detect community formation
3. **Market data** — detect regime shifts
4. **Protein folding** — detect secondary structure emergence
5. **Robot swarms** — detect coordination patterns

## Theory

See [docs/THEORY.md](docs/THEORY.md) for the algebraic topology behind H1 emergence.

## Why This Beats ML

See [docs/ML-COMPARISON.md](docs/ML-COMPARISON.md) for a detailed comparison.

**TL;DR:** ML finds patterns in data. Topology finds structure in relationships. For emergence detection, structure IS the signal.

## License

MIT
