use std::fs::File;
use std::io::{BufReader, BufRead};
use std::collections::HashMap;
use std::error::Error;
use plotters::prelude::*;
use rand::Rng;
// infers the number of nodes in the graph by finding the highest node index in the edgelist file
// #input: 'filename': path to the edgelist file (format: from to [weight])
//#output: returns one more than the highest node ID, representing total node count.
pub fn infer_graph_size(filename: &str) -> usize {
    let file = File::open(filename).expect("Could not open file");
    let reader = BufReader::new(file);
    let mut max_index = 0;
    for line in reader.lines().flatten() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let from: usize = parts[0].parse().unwrap_or(0);
            let to: usize = parts[1].parse().unwrap_or(0);
            max_index = max_index.max(from.max(to));
        }
    }
    max_index + 1
}

//plots a histogram from a binned degree count
// #input: 'binned_histogram': map from degree bin --> count of nodes
//#output: saves a PNG file 'degree_histogram.png' visualizing degree distribution
pub fn plot_histogram(binned_histogram: &HashMap<usize, usize>) -> Result<(), Box<dyn std::error::Error>> {
    let mut data: Vec<(usize, usize)> = binned_histogram.iter().map(|(&k, &v)| (k, v)).collect();
    data.sort_by_key(|&(bin, _)| bin); // sort bins left to right

    let max_y = data.iter().map(|&(_, y)| y).max().unwrap_or(1); //find y-axis upper bound

    let root = BitMapBackend::new("degree_histogram.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Node Out-Degree Histogram (Binned by 5s)", ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0u32..220u32, 0usize..(max_y + 5))?;

    chart
        .configure_mesh()
        .x_desc("Degree Range Start (e.g. 0 = 0–4)")
        .y_desc("Number of Nodes")
        .draw()?;

    chart.draw_series(data.iter().map(|&(bin, count)| {
        let bar_width = 4;
        let x0 = bin as u32 + ((5 - bar_width) / 2); // center the bar
        let x1 = x0 + bar_width;

        Rectangle::new(
            [(x0, 0), (x1, count)],
            BLUE.filled(),
        )
    }))?;

    Ok(())
}

//struct representing a directed, weighted graph using an adjacency list
// 'n': total number of nodes
//'outgoing': list of (to, weight) pairs for each node
pub struct Graph {
    //construct an empty graph with 'n' nodes
    pub n: usize,
    pub outgoing: Vec<Vec<(usize, f64)>>,
}

impl Graph {
    pub fn new(n: usize) -> Self {
        Graph {
            n,
            outgoing: vec![Vec::new(); n],
        }
    }

    //adds a directed edge from 'from' -> 'to' with a given weight
    pub fn add_edge(&mut self, from: usize, to: usize, weight: f64) {
        self.outgoing[from].push((to, weight));
    }

    //loads a graph from a file, assuming each line is:
    //'from to weight' (ignores header and malformed lines)
    pub fn from_file(filename: &str, n: usize) -> std::io::Result<Self> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        //skip first line (header)
        let _first_line = lines.next().ok_or(
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Empty file")
        )??;

        let mut graph = Graph::new(n);

        for line in lines {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue; //skip malformed lines
            }
            let from: usize = parts[0].parse().map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid from label")
            })?;
            let to: usize = parts[1].parse().map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid to label")
            })?;
            let weight: f64 = parts[2].parse().map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid weight label")
            })?;
            if from < n && to < n {
                graph.add_edge(from, to, weight);
            }
        }
        Ok(graph)
    }

    //converts the adjacency list into a 2D matrix where entry (i, j) = weight from i -> j
    pub fn to_adjacency_matrix(&self) -> Vec<Vec<f64>> {
        let mut matrix = vec![vec![0.0; self.n]; self.n];
        for from in 0..self.n {
            for &(to, weight) in &self.outgoing[from] {
                matrix[from][to] = weight;
            }
        }
        matrix
    }
}

// visualizes the graph using a scatter plot and connects nodes with edges
// highlights the top 'top_n' centrality nodes in read
// #inputs: 'graph': the graph to visualize, 'centrality_scores': list of (node, centrality) tuples, 'top_n': number of most central nodes to highlight
//#output: saves 'graph_visualization.png'
pub fn visualize_graph(
    graph: &Graph,
    centrality_scores: &Vec<(usize, f64)>,
    top_n: usize,
) -> Result<(), Box<dyn Error>> {
    use rand::Rng; // make sure you have this inside the function!

    //random layout: assign (x, y) positions to each node
    let mut rng = rand::thread_rng();
    let mut positions = vec![(0.0, 0.0); graph.n];
    for pos in &mut positions {
        pos.0 = rng.gen_range(50.0..750.0);
        pos.1 = rng.gen_range(50.0..550.0);
    }

    let root = BitMapBackend::new("graph_visualization.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Congress Graph Visualization", ("sans-serif", 30))
        .margin(20)
        .build_cartesian_2d(0f32..800f32, 0f32..600f32)?;

    chart.configure_mesh().disable_mesh().draw()?;

    //get top N central nodes to highlight
    let top_nodes: Vec<usize> = centrality_scores.iter().take(top_n).map(|&(node_idx, _)| node_idx).collect();

    //draw edges between nodes as lines
    for (from_idx, edges) in graph.outgoing.iter().enumerate() {
        for &(to_idx, _) in edges {
            let (x0, y0) = positions[from_idx];
            let (x1, y1) = positions[to_idx];
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(x0, y0), (x1, y1)],
                &BLACK,
            )))?;
        }
    }

        //draw nodes as colored circles (highlight top ones)
    for (node_idx, &(x, y)) in positions.iter().enumerate() {
        let radius = if top_nodes.contains(&node_idx) { 7 } else { 3 };
        let color = if top_nodes.contains(&node_idx) { &RED } else { &BLUE };

        chart.draw_series(std::iter::once(Circle::new(
            (x, y),
            radius,
            color.filled(),
        )))?;
    }
    Ok(())
}

// plots a histogram of degree centrality (in %)
//# input: 'centrality_scores': list of (nodes, centrality) values
//#output: saves 'centrality_histogram.png'
pub fn plot_centrality_histogram(centrality_scores: &Vec<(usize, f64)>) -> Result<(), Box<dyn Error>> {
    // bin centralities
    let mut bins: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for &(_, centrality) in centrality_scores {
        let bin = (centrality * 100.0).floor() as usize;
        *bins.entry(bin).or_insert(0) += 1;
    }

    let mut data: Vec<(usize, usize)> = bins.into_iter().collect();
    data.sort_by_key(|&(bin, _)| bin);

    let max_y = data.iter().map(|&(_, y)| y).max().unwrap_or(1);

    let root = BitMapBackend::new("centrality_histogram.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Degree Centrality Distribution", ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0u32..100u32, 0usize..(max_y + 5))?;

    chart
        .configure_mesh()
        .x_desc("Degree Centrality (%)")
        .y_desc("Number of Nodes")
        .draw()?;

    chart.draw_series(data.iter().map(|&(bin, count)| {
        let bar_width = 2;
        let x0 = bin as u32;
        let x1 = x0 + bar_width;

        Rectangle::new(
            [(x0, 0), (x1, count)],
            BLUE.filled(),
        )
    }))?;

    Ok(())
}

