use super::bim_tools::{Bim, BimTransit, BimZone};
use petgraph::graph::UnGraph;
use uuid::uuid;

/// https://www.techiedelight.com/implement-graph-data-structure-c
/// Data structure to store a graph object
pub struct BimGraph {
	pub head: Vec<BimNode>,
}

/// Data structure to store adjacency list nodes of the graph
#[derive(Default, Clone)]
pub struct BimNode {
	pub dest: usize,
	pub eid: usize,
	pub next: Option<Box<BimNode>>,
}

/// Data structure to store a graph edge
pub struct BimEdge {
	pub src: usize,
	pub dest: usize,
	pub id: usize,
}

pub fn bim_graph_new(bim: &Bim) -> BimGraph {
	let edges = graph_create_edges(&bim.transits, &bim.zones);

	graph_create(&edges, bim.zones.len())
}

pub fn bim_petgraph_new(bim: &Bim) -> UnGraph<&BimZone, BimTransit> {
	let mut graph = UnGraph::new_undirected();
	for transition in &bim.transits {
		match transition.outputs.len() {
			2 => {
				let first_node = bim
					.zones
					.iter()
					.find(|zone| zone.uuid.eq(&transition.outputs[0]))
					.expect("Cannot find zone");
				let second_node = bim
					.zones
					.iter()
					.find(|zone| zone.uuid.eq(&transition.outputs[1]))
					.expect("Cannot find zone");

				let first_node_index = graph.add_node(first_node);
				let second_node_index = graph.add_node(second_node);
				graph.add_edge(first_node_index, second_node_index, transition.clone());
			}
			1 => {
				let first_node = bim
					.zones
					.iter()
					.find(|zone| zone.uuid.eq(&transition.outputs[0]))
					.expect("Cannot find zone");
				let second_node = bim
					.zones
					.iter()
					.find(|zone| zone.uuid.eq(&uuid!("00000000-0000-0000-0000-000000000000"))) // TODO: remove hardcoded uuid
					.expect("Cannot find zone");

				let first_node_index = graph.add_node(first_node);
				let second_node_index = graph.add_node(second_node);
				graph.add_edge(first_node_index, second_node_index, transition.clone());
			}
			number_of_outputs => panic!(
				"Transition {} have {number_of_outputs} outputs",
				transition.uuid
			),
		}
	}
	graph
}

/// Function to create an adjacency list from specified edges
pub fn graph_create(edges: &[BimEdge], node_count: usize) -> BimGraph {
	// initialize head pointer for all vertices
	let mut graph_head: Vec<BimNode> = vec![Default::default(); node_count];

	// add edges to the directed graph one by one
	for edge in edges {
		// get the source and destination vertex
		let src = edge.src;
		let dest = edge.dest;
		let eid = edge.id;

		// 1. allocate a new node of adjacency list from `src` to `dest`
		let src_to_dest_node = BimNode {
			dest,
			eid,
			// point new node to the current head
			next: Some(Box::new(graph_head[src].clone())),
		};

		// point head pointer to the new node
		graph_head[src] = src_to_dest_node;

		// 2. allocate a new node of adjacency list from `dest` to `src`
		let dest_to_src_node = BimNode {
			dest: src,
			eid,
			// point new node to the current head
			next: Some(Box::new(graph_head[dest].clone())),
		};

		// change head pointer to point to the new node
		graph_head[dest] = dest_to_src_node;
	}

	// allocate storage for the graph data structure
	BimGraph { head: graph_head }
}

pub fn graph_create_edges(transits: &[BimTransit], zones: &[BimZone]) -> Vec<BimEdge> {
	let mut edges: Vec<BimEdge> = vec![];

	for (i, transition) in transits.iter().enumerate() {
		let mut ids = [0, zones.len()];
		let mut j = 0usize;
		for (k, zone) in zones.iter().enumerate() {
			if zone
				.outputs
				.iter()
				.any(|output| output.eq(&transition.uuid))
				&& j != 2
			{
				ids[j] = k;
				j += 1;
			}
		}

		edges.push(BimEdge {
			id: i,
			src: ids[0],
			dest: ids[1],
		});
	}

	edges
}
