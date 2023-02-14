use crate::graph::{digraph::QueryableGraph, EdgeId, VertexId};
use ahash::RandomState;
use std::collections::HashSet;

pub struct SelectedSubgraph<'a, G> {
    lower_graph: &'a G,
    selected_vertices: HashSet<VertexId, RandomState>,
    selected_edges: HashSet<EdgeId, RandomState>,
}

impl<'a, G> SelectedSubgraph<'a, G>
where
    G: QueryableGraph,
{
    pub fn new<EI, VI>(lower_graph: &'a G, edges: EI, additional_vertices: VI) -> Self
    where
        EI: Iterator<Item = EdgeId>,
        VI: Iterator<Item = VertexId>,
    {
        let selected_edges: HashSet<EdgeId, RandomState> =
            edges.filter(|e| lower_graph.edge(e).is_some()).collect();
        let selected_vertices = selected_edges
            .iter()
            .map(|e| {
                let e = lower_graph.edge(e).unwrap();
                [e.source, e.sink]
            })
            .flatten()
            .chain(additional_vertices)
            .collect();
        Self {
            lower_graph,
            selected_vertices,
            selected_edges,
        }
    }
}

impl<'a, G> QueryableGraph for SelectedSubgraph<'a, G>
where G: QueryableGraph
{
    fn vertex_size(&self) -> usize {
        self.selected_vertices.len()
    }

    fn vertices(&self) -> Box<dyn Iterator<Item = VertexId> + '_> {
        let it = self.selected_vertices.iter().copied();
        Box::new(it)
    }

    fn contains_vertex(&self, v: &VertexId) -> bool {
        self.selected_vertices.contains(v)
    }

    fn adjacent(&self, source: &VertexId, sink: &VertexId) -> Box<dyn Iterator<Item = crate::graph::Edge> + '_> {
        let it = self.lower_graph.adjacent(source, sink)
            .filter(|e| self.selected_edges.contains(&e.id));
        Box::new(it)
    }

    fn edge_size(&self) -> usize {
        self.selected_edges.len()
    }

    fn edges(&self) -> Box<dyn Iterator<Item = crate::graph::Edge> + '_> {
        let it = self.selected_edges.iter()
            .map(|e| self.lower_graph.edge(e).unwrap());
        Box::new(it)
    }

    fn contains_edge(&self, e: &EdgeId) -> bool {
        self.selected_edges.contains(e)
    }

    fn edge(&self, e: &EdgeId) -> Option<crate::graph::Edge> {
        if !self.selected_edges.contains(e) {
            return None;
        }
        self.lower_graph.edge(e)
    }

    fn in_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = crate::graph::Edge> + '_> {
        let it = self.lower_graph.in_edges(v)
            .filter(|e| self.selected_edges.contains(&e.id));
        Box::new(it)
    }

    fn out_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = crate::graph::Edge> + '_> {
        let it = self.lower_graph.out_edges(v)
            .filter(|e| self.selected_edges.contains(&e.id));
        Box::new(it)
    }
}
