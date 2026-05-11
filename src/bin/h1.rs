use clap::{Parser, Subcommand};
use h1_emergence::graph::Graph;
use h1_emergence::timeseries::{parse_timeseries_file, H1Tracker};
use h1_emergence::types::*;
use std::io::{self, BufRead};

#[derive(Parser)]
#[command(name = "h1", about = "H1 cohomology emergence detection", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a static graph from edge list
    Analyze {
        /// Edge list string, e.g. "1-2,2-3,3-1,1-4"
        #[arg(short, long)]
        graph: String,
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Analyze time-series of graph snapshots from file
    Stream {
        /// Path to data file (one graph per line)
        #[arg(short, long)]
        file: String,
    },
    /// Quick emergence check from parameters
    Detect {
        /// Number of edges
        #[arg(short, long)]
        edges: usize,
        /// Number of vertices
        #[arg(short, long)]
        vertices: usize,
        /// Number of connected components (default: 1)
        #[arg(short, long, default_value = "1")]
        components: usize,
    },
    /// Full topology report
    Topology {
        /// Number of vertices
        #[arg(short, long)]
        vertices: usize,
        /// Number of edges
        #[arg(short, long)]
        edges: usize,
    },
    /// Watch mode, reads edges from stdin line by line
    Watch,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { graph, json } => {
            let g = Graph::from_edge_string(&graph);
            let result = g.analyze();

            if json {
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            } else {
                print_result(&result);
            }
        }

        Commands::Stream { file } => {
            let graphs = match parse_timeseries_file(&file) {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };

            let config = H1Config::default();
            let mut tracker = H1Tracker::new(config);

            println!("Processing {} graph snapshots...\n", graphs.len());

            for (i, g) in graphs.iter().enumerate() {
                let result = tracker.analyze(g);
                let emoji = if result.detected { "🚨" } else { "✓" };
                println!(
                    "[{:3}] {} V={}, E={}, C={}, β1={}, V-2={}, severity={:.4}",
                    i + 1,
                    emoji,
                    result.vertices,
                    result.edges,
                    result.components,
                    result.betti_1,
                    result.v_minus_2,
                    result.emergence_severity
                );
                if result.detected && !result.cycles.is_empty() {
                    let persistent = result.cycles.iter().filter(|c| c.is_persistent).count();
                    println!(
                        "       {} cycles detected ({} persistent)",
                        result.cycles.len(),
                        persistent
                    );
                }
            }

            println!("\n--- Summary ---");
            println!("{}", tracker.summary());
            if tracker.sustained_emergence(3) {
                println!("⚠️  Sustained emergence detected across multiple windows!");
            }
        }

        Commands::Detect {
            edges,
            vertices,
            components,
        } => {
            let result = Graph::quick_check(edges, vertices, components);
            print_result(&result);
        }

        Commands::Topology { vertices, edges } => {
            println!("=== Topology Report ===");
            println!("Vertices (V):     {}", vertices);
            println!("Edges (E):        {}", edges);
            println!("Laman threshold (2V-3): {}", if vertices >= 2 {
                2 * vertices - 3
            } else {
                0
            });
            println!("Laman rigid (E >= 2V-3): {}", edges >= 2 * vertices.saturating_sub(3));
            println!("V - 2:            {}", vertices.saturating_sub(2));
            println!("Max β1 (K_V):     {}", if vertices >= 3 {
                vertices * (vertices - 1) / 2 - vertices + 1
            } else {
                0
            });

            println!("\n--- Emergence Zones ---");
            let v_minus_2 = vertices.saturating_sub(2);

            // Emergence threshold: β1 > V - 2
            // β1 = E - V + C (assuming C = 1)
            let emergence_min_edges = v_minus_2 + vertices - 1;
            println!("Emergence starts at E = {}", emergence_min_edges + 1);
            println!("(β1 = E - V + 1 > V - 2  ⟹  E > 2V - 3)");

            println!("\n--- Rigidity Zones ---");
            for e in 0..=(vertices * (vertices - 1) / 2) {
                let b1 = if e >= vertices { e - vertices + 1 } else { 0 };
                let threshold = vertices.saturating_sub(2);
                let over = b1 > threshold;
                let rigid = e >= 2 * vertices.saturating_sub(3);

                if over && rigid {
                    println!("  E={}: β1={} > V-2={}  🚨 EMERGENCE (over-constrained + Laman rigid)", e, b1, threshold);
                } else if over {
                    println!("  E={}: β1={} > V-2={}  ⚠️  Over-constrained", e, b1, threshold);
                } else if rigid {
                    println!("  E={}: β1={} ≤ V-2={}  ✓ Laman rigid (under threshold)", e, b1, threshold);
                } else if b1 > 0 {
                    println!("  E={}: β1={} ≤ V-2={}  ✓ Has cycles (flexible)", e, b1, threshold);
                } else {
                    println!("  E={}: β1=0  ✓ Tree (minimally rigid)", e);
                }
            }
        }

        Commands::Watch => {
            println!("H1 Watch Mode — reading edges from stdin (Ctrl+D to quit)");
            println!("Format: one edge per line, e.g. '1 2' or '1-2'");
            println!("Blank line resets the graph.\n");

            let mut graph = Graph::new();
            let mut reset_count = 0;

            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                let line = match line {
                    Ok(l) => l.trim().to_string(),
                    Err(_) => break,
                };

                if line.is_empty() {
                    // Reset
                    graph = Graph::new();
                    reset_count += 1;
                    println!("--- Graph reset (reset #{}) ---", reset_count);
                    continue;
                }

                // Parse edge: "u v" or "u-v"
                let nums: Vec<u64> = line
                    .split(|c| c == ' ' || c == '-')
                    .filter_map(|x| x.trim().parse().ok())
                    .collect();

                if nums.len() == 2 {
                    graph.add_edge(nums[0], nums[1]);
                    let result = graph.analyze();
                    let emoji = if result.detected { "🚨" } else { if result.betti_1 > 0 { "🔄" } else { "✓" } };
                    println!(
                        "{} Add {} {}: V={}, E={}, C={}, β1={}, V-2={}, severity={:.4}",
                        emoji,
                        nums[0],
                        nums[1],
                        result.vertices,
                        result.edges,
                        result.components,
                        result.betti_1,
                        result.v_minus_2,
                        result.emergence_severity
                    );
                    if result.detected {
                        for cycle in &result.cycles {
                            println!(
                                "   ↪ Cycle: [{}] length={}, holonomy={:.4}",
                                cycle
                                    .nodes
                                    .iter()
                                    .map(|n| n.to_string())
                                    .collect::<Vec<_>>()
                                    .join("-"),
                                cycle.length,
                                cycle.holonomy
                            );
                        }
                    }
                } else {
                    eprintln!("Bad edge format: '{}' — expected 'u v' or 'u-v'", line);
                }
            }
        }
    }
}

fn print_result(result: &EmergenceResult) {
    let emoji = if result.detected {
        "🚨"
    } else if result.betti_1 > 0 {
        "🔄"
    } else {
        "✓"
    };

    println!("{} H1 Emergence Analysis", emoji);
    println!("{}", str::repeat("─", 40));
    println!("Vertices (V):       {}", result.vertices);
    println!("Edges (E):          {}", result.edges);
    println!("Components (C):     {}", result.components);
    println!("β1 (E - V + C):     {}", result.betti_1);
    println!("V - 2:              {}", result.v_minus_2);
    println!("Emergence severity: {:.6}", result.emergence_severity);
    println!("Laman rigid:        {}", result.laman_rigid);
    println!("Over-constrained:   {}", result.over_constrained);
    println!("Emergence detected: {}", result.detected);

    if !result.cycles.is_empty() {
        println!("\nDetected cycles:");
        for (i, cycle) in result.cycles.iter().enumerate() {
            println!(
                "  {}: [{}] len={}, holonomy={:.4}",
                i + 1,
                cycle
                    .nodes
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join("-"),
                cycle.length,
                cycle.holonomy
            );
        }
    }

    if result.detected {
        println!("\n🚨 EMERGENCE: β1 ({}) > V-2 ({}) → Over-constrained!", result.betti_1, result.v_minus_2);
    } else if result.betti_1 > 0 {
        println!("\nℹ️  Has cycles but β1 ≤ V-2: structure is flexible.");
    } else {
        println!("\n✓ No cycles detected. Graph is a forest.");
    }

    println!("{}", str::repeat("─", 40));
}
