use crate::types::{Cycle, EmergenceResult};
use std::collections::{HashMap, HashSet, VecDeque};

/// Edge in the graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge {
    pub u: u64,
    pub v: u64,
}

impl Edge {
    pub fn new(u: u64, v: u64) -> Self {
        if u <= v {
            Self { u, v }
        } else {
            Self { u: v, v: u }
        }
    }
}

/// A graph represented as adjacency list
#[derive(Debug, Clone)]
pub struct Graph {
    adj: HashMap<u64, Vec<u64>>,
    edges: Vec<Edge>,
    vertices: HashSet<u64>,
}

impl Graph {
    /// Create an empty graph
    pub fn new() -> Self {
        Self {
            adj: HashMap::new(),
            edges: Vec::new(),
            vertices: HashSet::new(),
        }
    }

    /// Build a graph from edge list string like "1-2,2-3,3-1"
    pub fn from_edge_string(s: &str) -> Self {
        let mut graph = Self::new();
        for part in s.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let nums: Vec<u64> = part
                .split('-')
                .filter_map(|x| x.trim().parse().ok())
                .collect();
            if nums.len() == 2 {
                graph.add_edge(nums[0], nums[1]);
            }
        }
        graph
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, u: u64, v: u64) {
        let edge = Edge::new(u, v);
        if !self.edges.contains(&edge) {
            self.edges.push(edge);
        }
        self.adj.entry(u).or_default().push(v);
        self.adj.entry(v).or_default().push(u);
        self.vertices.insert(u);
        self.vertices.insert(v);
    }

    /// Number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Number of edges
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Get sorted vertex list
    pub fn vertex_list(&self) -> Vec<u64> {
        let mut v: Vec<u64> = self.vertices.iter().copied().collect();
        v.sort();
        v
    }

    /// Compute connected components via union-find (iterative BFS for large graphs)
    pub fn connected_components(&self) -> usize {
        let mut visited = HashSet::new();
        let mut components = 0;

        for &v in &self.vertices {
            if !visited.contains(&v) {
                components += 1;
                // BFS
                let mut queue = VecDeque::new();
                queue.push_back(v);
                visited.insert(v);

                while let Some(node) = queue.pop_front() {
                    if let Some(neighbors) = self.adj.get(&node) {
                        for &n in neighbors {
                            if !visited.contains(&n) {
                                visited.insert(n);
                                queue.push_back(n);
                            }
                        }
                    }
                }
            }
        }

        components
    }

    /// Compute spanning tree (forest) via DFS
    /// Returns (tree_edges, visited_set)
    pub fn spanning_tree(&self) -> (Vec<Edge>, HashSet<u64>) {
        let mut tree_edges = Vec::new();
        let mut visited = HashSet::new();

        // Sort vertices for deterministic output
        let mut vertices: Vec<u64> = self.vertices.iter().copied().collect();
        vertices.sort();

        for &start in &vertices {
            if visited.contains(&start) {
                continue;
            }

            // Iterative DFS
            let mut stack = vec![(start, None::<u64>)];
            visited.insert(start);

            while let Some((node, _parent)) = stack.pop() {
                if let Some(neighbors) = self.adj.get(&node) {
                    for &n in neighbors {
                        if !visited.contains(&n) {
                            visited.insert(n);
                            tree_edges.push(Edge::new(node, n));
                            stack.push((n, Some(node)));
                        }
                    }
                }
            }
        }

        (tree_edges, visited)
    }

    /// Find fundamental cycles using spanning tree
    /// For each non-tree edge, trace the path through the tree to find the cycle
    pub fn fundamental_cycles(&self) -> Vec<Cycle> {
        let (tree_edges, _) = self.spanning_tree();

        // Build tree adjacency from tree edges
        let mut tree_adj: HashMap<u64, Vec<u64>> = HashMap::new();
        for edge in &tree_edges {
            tree_adj.entry(edge.u).or_default().push(edge.v);
            tree_adj.entry(edge.v).or_default().push(edge.u);
        }

        // Set of tree edges for quick lookup
        let tree_edge_set: HashSet<Edge> = tree_edges.iter().copied().collect();

        let mut cycles = Vec::new();

        for edge in &self.edges {
            if tree_edge_set.contains(edge) {
                continue;
            }

            // This is a non-tree edge. Find cycle by tracing path in tree.
            if let Some(path) = find_path_in_tree(&tree_adj, edge.u, edge.v) {
                // The cycle is edge.u -> ... -> edge.v -> edge.u
                // path goes from edge.u to edge.v
                let cycle_nodes = path.clone();
                // Don't duplicate edge.v at the end since it connects back to edge.u
                let holonomy = cycle_nodes.len() as f64 / self.vertex_count().max(1) as f64;

                cycles.push(Cycle {
                    nodes: cycle_nodes,
                    length: path.len(),
                    holonomy,
                    first_detected: 0,
                    is_persistent: false,
                });
            }
        }

        cycles
    }

    /// Compute β1 = E - V + C
    pub fn betti_1(&self) -> usize {
        let v = self.vertex_count();
        let e = self.edge_count();
        let c = self.connected_components();
        if e >= v {
            e - v + c
        } else {
            let raw = e as isize - v as isize + c as isize;
            if raw > 0 { raw as usize } else { 0 }
        }
    }

    /// Full emergence analysis
    pub fn analyze(&self) -> EmergenceResult {
        let vertices = self.vertex_count();
        let edges = self.edge_count();
        let components = self.connected_components();
        let cycles = self.fundamental_cycles();

        EmergenceResult::new(vertices, edges, components, cycles)
    }

    /// Quick emergence check from parameters
    pub fn quick_check(edges: usize, vertices: usize, components: usize) -> EmergenceResult {
        EmergenceResult::new(vertices, edges, components, vec![])
    }
}

/// Find path between two nodes in a tree using BFS
fn find_path_in_tree(
    tree_adj: &HashMap<u64, Vec<u64>>,
    start: u64,
    end: u64,
) -> Option<Vec<u64>> {
    if start == end {
        return Some(vec![start]);
    }

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut parent: HashMap<u64, u64> = HashMap::new();

    queue.push_back(start);
    visited.insert(start);

    while let Some(node) = queue.pop_front() {
        if node == end {
            // Reconstruct path
            let mut path = Vec::new();
            let mut current = node;
            path.push(current);
            while let Some(&p) = parent.get(&current) {
                path.push(p);
                current = p;
            }
            path.reverse();
            return Some(path);
        }

        if let Some(neighbors) = tree_adj.get(&node) {
            for &n in neighbors {
                if !visited.contains(&n) {
                    visited.insert(n);
                    parent.insert(n, node);
                    queue.push_back(n);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_one_cycle() {
        // Triangle: 3 vertices, 3 edges → β1 = 3 - 3 + 1 = 1
        let g = Graph::from_edge_string("1-2,2-3,3-1");
        assert_eq!(g.vertex_count(), 3);
        assert_eq!(g.edge_count(), 3);
        assert_eq!(g.connected_components(), 1);
        assert_eq!(g.betti_1(), 1);
        let cycles = g.fundamental_cycles();
        assert_eq!(cycles.len(), 1, "Triangle should have 1 fundamental cycle");
    }

    #[test]
    fn test_square_one_cycle() {
        // Square: 4 vertices, 4 edges → β1 = 4 - 4 + 1 = 1
        let g = Graph::from_edge_string("1-2,2-3,3-4,4-1");
        assert_eq!(g.vertex_count(), 4);
        assert_eq!(g.edge_count(), 4);
        assert_eq!(g.betti_1(), 1);
        let cycles = g.fundamental_cycles();
        assert_eq!(cycles.len(), 1, "Square should have 1 fundamental cycle");
    }

    #[test]
    fn test_k4_three_cycles() {
        // K4: 4 vertices, 6 edges → β1 = 6 - 4 + 1 = 3
        let g = Graph::from_edge_string("1-2,1-3,1-4,2-3,2-4,3-4");
        assert_eq!(g.vertex_count(), 4);
        assert_eq!(g.edge_count(), 6);
        assert_eq!(g.betti_1(), 3);
        let cycles = g.fundamental_cycles();
        // K4 has 6 edges, spanning tree has 3 edges, so 3 non-tree edges → 3 fundamental cycles
        assert_eq!(cycles.len(), 3, "K4 should have 3 fundamental cycles");
    }

    #[test]
    fn test_tree_zero_cycles() {
        // Tree: 4 vertices, 3 edges → β1 = 3 - 4 + 1 = 0
        let g = Graph::from_edge_string("1-2,2-3,3-4");
        assert_eq!(g.betti_1(), 0);
        let cycles = g.fundamental_cycles();
        assert_eq!(cycles.len(), 0, "Tree should have 0 cycles");
    }

    #[test]
    fn test_betti_computation() {
        // V=4, E=5, C=1 → β1 = 5 - 4 + 1 = 2
        let g = Graph::from_edge_string("1-2,2-3,3-4,4-1,1-3");
        assert_eq!(g.betti_1(), 2);
        assert_eq!(g.connected_components(), 1);
    }

    #[test]
    fn test_emergence_detection() {
        // V=4, E=6, C=1 → β1 = 3, V-2 = 2 → emergence detected (β1 > V-2)
        let g = Graph::from_edge_string("1-2,1-3,1-4,2-3,2-4,3-4");
        let result = g.analyze();
        assert!(result.detected, "K4 should have emergence detected");
        assert!(result.over_constrained);
        assert!(result.laman_rigid);
        assert_eq!(result.betti_1, 3);
        assert_eq!(result.v_minus_2, 2);
        assert!(result.emergence_severity > 0.0);
    }

    #[test]
    fn test_under_constrained_graph() {
        // V=4, E=2, C=1 → β1 = 0 (E - V + C = 2 - 4 + 1 = -1 → clamped to 0)
        let g = Graph::from_edge_string("1-2,3-4");
        // This is actually 2 components since 1-2 and 3-4 are disconnected
        assert_eq!(g.connected_components(), 2);
        // Actually: V=4, E=2, C=2 → β1 = 2 - 4 + 2 = 0
        let result = g.analyze();
        assert!(!result.detected, "Under-constrained graph should not detect emergence");
        assert!(!result.laman_rigid);
    }
}
