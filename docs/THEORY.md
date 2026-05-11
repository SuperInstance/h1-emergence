# Algebraic Topology for Programmers

This document explains the mathematical intuition behind H1 emergence detection.
No prior topology knowledge required.

## The Core Insight

**Topology measures connectivity, not distance.**

While ML looks at patterns in feature space, topology looks at how things are connected.
For emergence detection, structure IS the signal.

## What is Homology?

Homology is a way to count **holes** in a space:

| Dimension | What it counts | Graph analog |
|-----------|---------------|--------------|
| H₀ | Connected components | Number of pieces |
| H₁ | 1-dimensional holes | Independent cycles |
| H₂ | 2-dimensional voids | Enclosed cavities |

### H₀: Connected Components

```
A---B   C---D   E
   2 components     1 component
```

H₀ = number of connected components. Every point connects to every other point
in its component through some path.

### H₁: The First Betti Number

H₁ (β₁) counts **independent cycles** — loops that don't bound a filled-in area.

```
Triangle:         Square:           K4 (complete):
A---B             A---B             A---B
 \ /              |   |             |\ /|
  C               |   |             | X |
                  D---C             |/ \|
                                    C---D
β₁ = 1            β₁ = 1            β₁ = 3
```

**Why K4 has 3 cycles:** A complete graph on 4 vertices has 6 edges. The
spanning tree needs 3 edges. The remaining 3 edges each create one
independent cycle. These correspond to the 3 independent triangular faces
of a tetrahedron.

## The Formula

```
β₁ = E − V + C
```

Where:
- **E** = number of edges
- **V** = number of vertices
- **C** = number of connected components

This is the **Euler characteristic** for 1-dimensional complexes (graphs).

### Intuition

A tree has V−1 edges and 1 component:
```
β₁ = (V−1) − V + 1 = 0
```

Each additional edge beyond V−C creates exactly one new independent cycle.

## Laman's Theorem and Rigidity

**Laman's theorem** (1970) characterizes minimally rigid graphs in 2D:

- A graph with V vertices is **minimally rigid** if it has exactly 2V−3 edges
  and every subgraph also has ≤ 2V'−3 edges.
- **Generic rigidity:** E ≥ 2V−3 means the framework is rigid (can't flex)
  in generic position.

### The Threshold

```
V − 2 = Laman degrees of freedom
```

Think of it this way:
- First vertex: fixed in place (0 DOF)
- Second vertex: can move on a line (1 DOF)
- Third vertex: can be anywhere (2 DOF)
- Each additional vertex adds 2 more DOF

Total degrees of freedom: **2V − 3** (position minus rotations/translations)
Each edge removes 1 DOF.

So:
- E edges consume E degrees of freedom
- 2V−3 is the total DOF available
- When E > 2V−3: **over-constrained** (emergent behavior)

### Connection to β₁

Since β₁ = E − V + C and C = 1 for a connected graph:
```
β₁ > V − 2  ⟺  E − V + 1 > V − 2  ⟺  E > 2V − 3
```

**Emergence = over-constrained = β₁ > V − 2 = E > 2V − 3**

They're the same condition seen from three different angles.

## Why V − 2 (Not V or V−1)?

| Threshold | Meaning | Formula |
|-----------|---------|---------|
| V − 1 | Tree-bound: β₁ = 0 | E = V − 1 |
| V − 2 | Laman bound: emergence threshold | β₁ = V − 2 |
| V − 1 | Max tight | β₁ = V − 1 |
| V | Well beyond emergence | β₁ = V |
| V + 1 | Highly over-constrained | β₁ = V + 1 |

The Laman threshold V−2 emerges naturally from the rigidity theory:
it's the point where constraints exceed configurational degrees of freedom.

## Fundamental Cycles

Fundamental cycles are computed from a **spanning tree**:

1. Find a spanning tree (DFS/BFS) covering all vertices
2. For each edge NOT in the tree, trace the unique path through the tree
   between its endpoints
3. The edge + tree path = one fundamental cycle

```
Spanning tree (bold):      Non-tree edge (--) creates cycle:
A═══B                       A═══B
    ║                            ║
    C═══D                        C═══D
         ║                           ║
         E---F                       E---F  ← cycle: E-F-C-D-E
```

## Persistence

In time-series analysis, **persistence** measures whether a cycle survives
across multiple observations:

- A cycle seen once: possible noise
- A cycle seen twice: possibly real
- A cycle seen three+ times: **persistent structure**

Persistence scoring uses cycle fingerprints (sorted edge multisets hashed
for quick comparison).

## Emergence Severity

```
emergence_severity = β₁ / (V − 2) − 1
```

- **severeity = 0**: At threshold (β₁ exactly equals V−2)
- **severity > 0**: Emergence detected
- **severity = 1.0**: β₁ is 2× the threshold
- **severity = 2.0**: β₁ is 3× the threshold

## Geometric Interpretation

```
E vs. V for a connected graph:

E = V-1    Tree (minimally connected)
    ↓
E = 2V-3   Laman bound (minimally rigid)
    ↓
E = V+1    Emergence starts (for V=4: β₁ > 2)
    ↓
E = V(V-1)/2  Complete graph (maximally connected)
```

In the complete graph K_V:
```
β₁_max = V(V−1)/2 − V + 1 = (V−1)(V−2)/2
```

For V=10: β₁_max = 36, threshold = 8. Emergence at β₁ = 9.
The complete graph is 4.5× over the threshold.

## Further Reading

- **Edelsbrunner, Harer:** *Computational Topology* — the standard text
- **Laman, G.:** *On graphs and rigidity of plane skeletal structures* (1970)
- **Ghrist, R.:** *Elementary Applied Topology* — accessible introduction
- **Carlsson, G.:** *Topology and data* (2009) — TDA survey

## Key Takeaway

```
Emergence = β₁ > V − 2
           = E > 2V − 3
           = over-constrained
           = structure that can't flex
```

When a system becomes over-constrained, it's no longer just connected —
it has **emergent properties** that can't be explained by individual
connections alone. That's the signature of emergence.
