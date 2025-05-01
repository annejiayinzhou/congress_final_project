use std::error::Error;
use std::fs::File;
use std::io::Write;
use polars::prelude::*;
use congress_project::graph::{Graph, infer_graph_size, plot_histogram, visualize_graph};
use congress_project::graph::{plot_centrality_histogram};
use plotters::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let filename = "data/edgelist.txt";

    let n = infer_graph_size(filename);
    println!("Inferred graph size: {}", n);

    let graph = Graph::from_file(filename, n)?;
    println!("Graph loaded with {} nodes.", graph.n);

    // this is building the data frame
    let mut from_nodes = Vec::new();
    let mut to_nodes = Vec::new();
    let mut weights = Vec::new();

    for (node_idx, edges) in graph.outgoing.iter().enumerate() {
        for &(target, weight) in edges {
            from_nodes.push(node_idx as u32);
            to_nodes.push(target as u32);
            weights.push(weight as f32);
        }
    }

    let df = df![
        "from" => from_nodes,
        "to" => to_nodes,
        "weight" => weights,
    ]?;

    // first ten rows! (like python)
    println!("\nFirst 10 edges (like pandas head()):");
    println!("{:?}", df.head(Some(10)));

    // building + printing the adjacency matrix
    println!("\nAdjacency Matrix (weights, not just 1s):");
    let matrix = graph.to_adjacency_matrix();
    let n = matrix.len();

    // this is the column headers
    print!("\t");
    for col_idx in 0..n {
        print!("{} \t", col_idx);
    }
    println!();

    // rows
    for (row_idx, row) in matrix.iter().enumerate() {
        print!("{}\t", row_idx);
        for &weight in row {
            if weight == 0.0 {
                print!(".\t");
            } else {
                print!("{:.3}\t", weight);
            }
        }
        println!();
    }

    // count connectivity (out-degree)
    let mut degrees = Vec::new();
    for (node_idx, edges) in graph.outgoing.iter().enumerate() {
        degrees.push((node_idx, edges.len()));
    }

    // this is sort by degree descending
    degrees.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\nTop 10 Most Connected Nodes:");
    for &(node_idx, degree) in degrees.iter().take(10) {
        println!("Node {}: {} outgoing connections", node_idx, degree);
    }

    let most_connected = degrees.first().unwrap();
    println!("\nNode with most outgoing connections: Node {} with {} edges.", most_connected.0, most_connected.1);

    // plot degree histogram
    let mut out_degree_histogram = std::collections::HashMap::new();
    for &(_, degree) in &degrees {
        let bin = (degree / 5) * 5;
        *out_degree_histogram.entry(bin).or_insert(0) += 1;
    }

    plot_histogram(&out_degree_histogram)?;

    // calculate and save degree centrality
    println!("\nTop 10 Most Influential Congress Members by Degree Centrality:");
    let total_nodes = graph.n;
    let mut centrality_scores = Vec::new();
    for &(node_idx, degree) in &degrees {
        let centrality = degree as f64 / (total_nodes as f64 - 1.0);
        centrality_scores.push((node_idx, centrality));
    }

    for &(node_idx, centrality) in centrality_scores.iter().take(10) {
        println!("Node {}: Centrality {:.4}", node_idx, centrality);
    }

    // plot degree centrality distribution
    plot_centrality_histogram(&centrality_scores)?;

    // save centrality scores into csv
    save_centrality_to_csv(&centrality_scores)?;

    visualize_graph(&graph, &centrality_scores, 10)?;


    Ok(())

}

// helper function to save centrality into CSV
fn save_centrality_to_csv(centrality_scores: &Vec<(usize, f64)>) -> Result<(), Box<dyn Error>> {
    let mut file = File::create("centrality.csv")?;
    writeln!(file, "node_id,centrality")?;
    for &(node_idx, centrality) in centrality_scores {
        writeln!(file, "{},{}", node_idx, centrality)?;
    }
    Ok(())
}

