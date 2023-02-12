use crate::graph::*;

pub trait GrowableGraph {
    fn new() -> Self;
    fn add_vertex(&mut self) -> VertexId;
    fn add_edge(&mut self, source: VertexId, sink: VertexId) -> EdgeId;
}

pub trait EdgeShrinkableGraph {
    fn remove_edge(&mut self, edge: &EdgeId) -> Option<Edge>;
}

pub trait VertexShrinkableGraph: EdgeShrinkableGraph {
    fn remove_vertex(&mut self, vertex: &VertexId) -> Box<dyn Iterator<Item = Edge> + 'static>;
}

pub trait QueryableGraph {
    fn vertex_size(&self) -> usize;
    fn vertices(&self) -> Box<dyn Iterator<Item = VertexId> + '_>;
    fn contains_vertex(&self, v: &VertexId) -> bool;
    fn adjacent(&self, source: &VertexId, sink: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_>;

    fn edge_size(&self) -> usize;
    fn edges(&self) -> Box<dyn Iterator<Item = Edge> + '_>;
    fn contains_edge(&self, e: &EdgeId) -> bool;
    fn edge(&self, e: &EdgeId) -> Option<Edge>;
    fn in_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_>;
    fn out_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_>;
}
