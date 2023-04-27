use crate::graph::*;
use std::io::Write;

/// A trait for low-level growable graphs.
pub trait GrowableGraph {
    /// Generate a new and empty graph.
    fn new() -> Self;
    /// Add a new vertex into the graph.
    fn add_vertex(&mut self) -> VertexId;
    /// Add a new edge from `source` to `sink` for directed graphs or between them for undirected graphs.
    fn add_edge(&mut self, source: VertexId, sink: VertexId) -> EdgeId;
}

/// A trait for low-level graphs whose edges can be removed.
pub trait EdgeShrinkableGraph {
    /// Remove an edge from the graph.
    ///
    /// If the edge ID is not in the graph, `None` is returned;
    /// otherwise, it returns complete information about the edge.
    fn remove_edge(&mut self, edge: &EdgeId) -> Option<Edge>;
}

/// A trait for low-level graphs whose vertices can be removed.
pub trait VertexShrinkableGraph: EdgeShrinkableGraph {
    /// Remove a vertex from the graph and all edges connected to this vertex.
    ///
    /// It returns an iterator over all edges connected to the vertex.
    /// Each edge will occur exactly once during the iteration.
    /// Thus, self-loops will occur once as well.
    ///
    /// For undirected graphs, the removed vertex can be either the sources or the sinks of returned edges.
    /// It is implementation-specific.
    ///
    /// If the vertex is not in the graph, it returns an empty iterator.
    fn remove_vertex(&mut self, vertex: &VertexId) -> Box<dyn Iterator<Item = Edge> + 'static>;
}

/// A trait for querying vertices and edges about low-level graphs.
pub trait QueryableGraph {
    /// Number of vertices in the graph.
    fn vertex_size(&self) -> usize;
    /// Iteration over all vertices in the graph.
    fn iter_vertices(&self) -> Box<dyn Iterator<Item = VertexId> + '_>;
    /// Whether a vertex is in the graph or not.
    fn contains_vertex(&self, v: &VertexId) -> bool;

    /// Number of edges in the graph.
    fn edge_size(&self) -> usize;
    /// Iteration over all edges in the graph.
    fn iter_edges(&self) -> Box<dyn Iterator<Item = Edge> + '_>;
    /// Whether an edge is in the graph or not.
    fn contains_edge(&self, e: &EdgeId) -> bool;
    /// Returns information about a specified edge in the graph.
    fn find_edge(&self, e: &EdgeId) -> Option<Edge>;
    /// Iteration over all edges between two vertices in undirected graphs
    /// or those from `source` to `sink` in directed graphs.
    fn edges_connecting(
        &self,
        source: &VertexId,
        sink: &VertexId,
    ) -> Box<dyn Iterator<Item = Edge> + '_>;
    /// Iteration over all edges going into the vertex `v`.
    ///
    /// For undirected graphs, the sinks of returned edges must be `v`.
    fn in_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_>;
    /// Iteration over all edges going out of `v`.
    ///
    /// For undirected graphs, the sources of returned edges must be `v`.
    fn out_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_>;

    /// Returns something can inspect into the graph.
    fn debug(&self) -> Box<dyn std::fmt::Debug + '_>
    where
        Self: Sized,
    {
        Box::new(super::GraphDebug::new(self))
    }
}

/// Whether a graph is directed or not.
pub trait DirectedOrNot {
    /// When the graph is directed, it is true; otherwise, it is false.
    const DIRECTED_OR_NOT: bool;
}

/// A trait for subgraphs.
///
/// New vertices and edges are disallowed to add into subgraphs.
/// But those from the underlying graph can be uncovered.
pub trait Subgraph {
    type LowerGraph;

    fn new(lower_graph: Self::LowerGraph) -> Self;
    /// Uncovers a vertex.
    ///
    /// Edges connecting to this vertex will not be uncovered.
    ///
    /// Uncovering an already uncovered vertex takes no effect at all.
    fn uncover_vertex(&mut self, v: VertexId) -> &mut Self;
    /// Uncovers an edge, and both endpoints of this edge as well.
    ///
    /// Uncovering an already uncovered edge takes no effect at all.
    fn uncover_edge(&mut self, v: EdgeId) -> &mut Self;
}


pub trait DumpInGraphviz
where Self: QueryableGraph + DirectedOrNot
{
    fn dump_in_graphviz(&self, graph_name: &str) -> String {
        let mut res = String::new();
        if Self::DIRECTED_OR_NOT {
            res.push_str("digraph ");
        } else {
            res.push_str("graph ");
        }
        res.push_str(graph_name);
        res.push_str(" {");
        res.push_str("}");
        res
    }
}
