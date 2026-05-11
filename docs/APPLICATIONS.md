# Real-World Applications

## 1. Network Monitoring — Routing Anomaly Detection

**Problem:** A network router starts creating unexpected routes. Traditional ML
models flag "anomalies" based on traffic patterns, but can't distinguish between
legitimate rerouting and malicious misconfiguration.

**H1 approach:** Model the network as a graph where vertices = routers, edges = routes.
Normal operation has β₁ tracking the expected topology (known redundant paths).
When a new route creates an unexpected cycle:

```
Before (expected):        After (anomaly):
A---B---C                 A---B---C
|       |                 |\  |  /|
D---E---F                 D---E---F

β₁ = 1 (expected loop)   β₁ = 3 (unexpected loops!)
V-2 = 4 → No emergence   V-2 = 4 → β₁ ≤ V-2 → Still below emergence
                          (but β₁ jumped from 1 to 3 — trend alert)
```

When β₁ exceeds V−2, the network has more redundancy than degrees of freedom
— a signature of routing misconfiguration or attack.

```
h1 stream --file network_snapshots.json

[  1] ✓ V=6, E=6, C=1, β1=1, V-2=4, severity=-0.750
[  2] ✓ V=6, E=6, C=1, β1=1, V-2=4, severity=-0.750
[  3] 🚨 V=6, E=10, C=1, β1=5, V-2=4, severity=0.250  ← Routing anomaly!
```

## 2. Social Network — Community Formation Detection

**Problem:** Detect when a social network transitions from loose connections
to a tightly-knit community (echo chamber formation).

**H1 approach:** Model users as vertices, follows/connections as edges.
A loose network has β₁ ≈ 0. As communities form, cycles appear:

```rust
use h1_emergence::graph::Graph;

// A loose social graph (early stage)
let g = Graph::from_edge_string(
    "1-2,2-3,3-4,4-5,5-6,1-7,7-8"
);
let result = g.analyze();
// β₁ = 7 - 8 + 1 = 0 → no cycles → loose network

// A dense community forms (emergence!)
let g = Graph::from_edge_string(
    "1-2,1-3,1-4,2-3,2-4,3-4,4-5,5-6,5-7,6-7"
);
let result = g.analyze();
// V=7, E=10, C=1 → β₁ = 10 - 7 + 1 = 4
// V-2 = 5 → β₁ = 4 ≤ 5 → approaching but not yet emergent
// Add one more edge...

let g = Graph::from_edge_string(
    "1-2,1-3,1-4,2-3,2-4,3-4,4-5,5-6,5-7,6-7,4-6"
);
let result = g.analyze();
// V=7, E=11, C=1 → β₁ = 5
// V-2 = 5 → β₁ = 5 → AT threshold!
// β₁ = V-2 means the community is just barely emergent
```

The transition from β₁ < V−2 to β₁ > V−2 marks the **phase transition**
from "collection of people" to "community with emergent properties"
(e.g., groupthink, echo chamber dynamics).

## 3. Market Data — Regime Shift Detection

**Problem:** Detect when a market transitions from normal volatility to
a new regime (crash, bubble, or structural change).

**H1 approach:** Model assets as vertices, correlations as edges
(edge exists when |correlation| > threshold). Normal markets have
stable β₁. Regime shifts change the correlation structure:

```
Normal market:                   Crash regime:
A---B   C---D                    A---B
                                |\ /|
                                | X |
                                |/ \|
                                C---D

β₁ ≈ 0 (uncorrelated)           β₁ = 3 (all correlated)
V-2 = 2 → No emergence          V-2 = 2 → β₁ > 2 → EMERGENCE!
```

**Why this works:** In normal markets, assets have diverse correlation
structures. In a crash, "everything goes down together" — the correlation
graph becomes nearly complete, pushing β₁ past the emergence threshold.

```
// USD, EUR, JPY, BTC correlation graph during normal market
let normal = Graph::from_edge_string("USD-EUR,JPY-BTC");
let result = normal.analyze();   // β₁ = 0

// During flash crash: all assets correlate
let crash = Graph::from_edge_string(
    "USD-EUR,USD-JPY,USD-BTC,EUR-JPY,EUR-BTC,JPY-BTC"
);
let result = crash.analyze();    // β₁ = 3, V-2 = 2 → 🚨
```

## 4. Protein Folding — Secondary Structure Detection

**Problem:** Detect when a protein transitions from random coil to
structured secondary elements (alpha helices, beta sheets).

**H1 approach:** Model amino acids as vertices, spatial proximity as edges.
Unfolded proteins have few edges (no structure). As folding progresses,
hydrogen bonds create cycles:

```
Unfolded (random coil):     Intermediate:          Folded (beta sheet):
C---C---C---C---C           C---C---C              C---C---C
                            |       |              |\  |  /|
                            |       |              | \ | / |
                            C---C---C              |  \|/  |
                                                   C---C---C
β₁ = 0                      β₁ = 1                 β₁ = 3
No emergence                Below threshold         🚨 EMERGENCE!
```

This corresponds to the **nucleation-condensation** model of protein folding:
once enough constraints form (β₁ > V−2), the structure "snaps" into
its folded state.

**Key insight:** The emergence threshold V−2 acts as a **critical point**
in the folding process. Before the threshold, the protein can still unfold.
Past the threshold, the structure is locked in — it's folded.

## 5. Robot Swarm — Coordination Pattern Detection

**Problem:** Detect when a swarm of simple robots transitions from
individual movement to coordinated collective behavior.

**H1 approach:** Model robots as vertices, communication/sensing links as edges.
Individual robots form a sparse graph (tree-like). When coordination emerges:

```
Individual movement:        Coordinated swarm:
A   B   C   D               A---B
                            |\ /|
                            | X |
                            |/ \|
                            C---D

β₁ = 0 (no cycles)          β₁ = 3 (swarm topology)
V-2 = 2 → No emergence      V-2 = 2 → β₁ > 2 → EMERGENCE!
```

**Why cycles = coordination:** When robots act independently, communication
links form transient connections. When they coordinate (flocking, formation
keeping), persistent cycles appear in the communication graph. The β₁
threshold marks the transition from "herd" to "swarm."

```rust
use h1_emergence::timeseries::H1Tracker;
use h1_emergence::types::H1Config;

let mut tracker = H1Tracker::new(H1Config::default());

// Feed swarm snapshots over time
for snapshot in swarm_snapshots {
    let result = tracker.analyze(&snapshot);
    if result.detected {
        // Swarm transition detected!
        trigger_swarm_response();
    }
}

fn trigger_swarm_response() {
    println!("🚨 Swarm coordination detected — issuing coordinated commands");
}
```

## Common Pattern

All five applications share the same signature:

1. Model the system as a **graph**
2. Track **β₁** (independent cycles) over time
3. Detect emergence when **β₁ > V − 2**

The threshold V−2 is **system-independent**. It doesn't matter if you're
analyzing routers, people, assets, proteins, or robots — the topology
detects emergence at the same algebraic boundary.

**One formula. One threshold. Any system.**
