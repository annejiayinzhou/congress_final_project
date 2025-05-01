use std::collections::HashMap;
use std::fs::write;

// Update this to match your crate name in Cargo.toml (e.g., "congress_project")
use congress_project::graph::{Graph, plot_histogram};

#[test case 1]
fn test_add_edge() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1, 1.5);
    assert_eq!(g.outgoing[0], vec![(1, 1.5)]);
}

#[test case 2]
fn test_adjacency_matrix() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1, 2.0);
    g.add_edge(1, 2, 3.5);
    let matrix = g.to_adjacency_matrix();
    assert_eq!(matrix[0][1], 2.0);
    assert_eq!(matrix[1][2], 3.5);
    assert_eq!(matrix[2][0], 0.0);
}

#[test case 3]
fn test_plot_histogram_mock() {
    let mut hist = HashMap::new();
    hist.insert(0, 2);
    hist.insert(5, 3);
    hist.insert(10, 1);
    let result = plot_histogram(&hist);
    assert!(result.is_ok());
}

#[test case 4]
fn test_from_file_with_mock_data() {
    let path = "test_edgelist.txt";
    let content = "Header\n0 1 1.0\n1 2 1.0\n2 0 1.0\n";
    write(path, content).unwrap();

    let g = Graph::from_file(path, 3).expect("should read file");
    assert_eq!(g.outgoing[0], vec![(1, 1.0)]);
    assert_eq!(g.outgoing[1], vec![(2, 1.0)]);
    assert_eq!(g.outgoing[2], vec![(0, 1.0)]);

    std::fs::remove_file(path).unwrap();
}
