use crate::graph::*;
use crate::tagged::traits::{GrowableTaggedGraph, TaggedGraph};
use ahash::RandomState;
use bimap::BiHashMap;
use std::hash::Hash;

/// A naive implementation of tagged graphs.
#[derive(Clone)]
pub struct NaiveTaggedGraph<V, E, G = directed::TreeBackedGraph>
where
    V: Hash + Eq + Clone,
    E: Hash + Eq + Clone,
{
    lower_graph: G,
    vertices: BiHashMap<VertexId, V, RandomState, RandomState>,
    edges: BiHashMap<EdgeId, E, RandomState, RandomState>,
}

impl<V, E, G> DirectedOrNot for NaiveTaggedGraph<V, E, G>
where
    V: Hash + Eq + Clone,
    E: Hash + Eq + Clone,
    G: DirectedOrNot,
{
    const DIRECTED_OR_NOT: bool = G::DIRECTED_OR_NOT;
}

impl<V, E, G> Default for NaiveTaggedGraph<V, E, G>
where
    V: Hash + Eq + Clone,
    E: Hash + Eq + Clone + super::Edge,
    G: GrowableGraph,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<V, E, G> super::TaggedGraph for NaiveTaggedGraph<V, E, G>
where
    V: Hash + Eq + Clone,
    E: Hash + Eq + Clone + super::Edge,
{
    type LowerGraph = G;
    type Vertex = V;
    type Edge = E;

    fn lower_graph(&self) -> &Self::LowerGraph {
        &self.lower_graph
    }

    fn vertex_by_id(&self, vid: &VertexId) -> Option<&Self::Vertex> {
        self.vertices.get_by_left(vid)
    }

    fn id_by_vertex(&self, vert: &Self::Vertex) -> Option<VertexId> {
        self.vertices.get_by_right(vert).copied()
    }

    fn edge_by_id(&self, eid: &EdgeId) -> Option<&Self::Edge> {
        self.edges.get_by_left(eid)
    }

    fn id_by_edge(&self, edge: &Self::Edge) -> Option<EdgeId> {
        self.edges.get_by_right(edge).copied()
    }
}

impl<V, E, G> super::GrowableTaggedGraph for NaiveTaggedGraph<V, E, G>
where
    V: Hash + Eq + Clone,
    E: Hash + Eq + Clone + super::Edge,
    G: GrowableGraph,
{
    fn new() -> Self {
        Self {
            lower_graph: G::new(),
            vertices: BiHashMap::with_hashers(RandomState::new(), RandomState::new()),
            edges: BiHashMap::with_hashers(RandomState::new(), RandomState::new()),
        }
    }

    fn overwrite_vertex(&mut self, vert: Self::Vertex) -> VertexId {
        if let Some((vid, _)) = self.vertices.remove_by_right(&vert) {
            self.vertices.insert(vid, vert);
            vid
        } else {
            let vid = self.lower_graph.add_vertex();
            self.vertices.insert(vid, vert);
            vid
        }
    }

    fn add_edge(&mut self, edge: Self::Edge) -> EdgeId {
        let vid_src = edge.source();
        let vid_snk = edge.sink();
        let eid = self.lower_graph.add_edge(vid_src, vid_snk);
        self.edges.insert(eid, edge);
        eid
    }

    fn update_edge(&mut self, eid: EdgeId, new: Self::Edge) {
        {
            let old = self.edge_by_id(&eid).unwrap();
            assert_eq!(old.source(), new.source());
            assert_eq!(old.sink(), new.sink());
        }
        let _ = self.edges.insert(eid, new);
    }
}

impl<V, E, G> super::EdgeShrinkableTaggedGraph for NaiveTaggedGraph<V, E, G>
where
    V: Hash + Eq + Clone,
    E: Hash + Eq + Clone + super::Edge,
    G: EdgeShrinkableGraph,
{
    fn remove_edge(&mut self, eid: &EdgeId) -> Option<Self::Edge> {
        self.lower_graph
            .remove_edge(eid)
            .and_then(|_| self.edges.remove_by_left(eid))
            .map(|(_, e)| e)
    }
}

impl<V, E, G> super::VertexShrinkableTaggedGraph for NaiveTaggedGraph<V, E, G>
where
    V: Hash + Eq + Clone,
    E: Hash + Eq + Clone + super::Edge,
    G: VertexShrinkableGraph,
{
    fn remove_vertex(
        &mut self,
        vid: &VertexId,
    ) -> Box<dyn Iterator<Item = (EdgeId, Self::Edge)> + '_> {
        if self.vertices.remove_by_left(vid).is_some() {
            let eids: Vec<_> = self.lower_graph.remove_vertex(vid).map(|e| e.id).collect();
            let res: Vec<_> = eids
                .iter()
                .map(|eid| self.edges.remove_by_left(eid).unwrap())
                .collect();
            Box::new(res.into_iter())
        } else {
            Box::new(std::iter::empty())
        }
    }
}

impl<V, E, G> super::QueryableTaggedGraph for NaiveTaggedGraph<V, E, G>
where
    V: Hash + Eq + Clone,
    E: Hash + Eq + Clone + super::Edge,
    G: QueryableGraph,
{
    fn vertex_size(&self) -> usize {
        self.vertices.len()
    }

    fn iter_vertices(&self) -> Box<dyn Iterator<Item = (VertexId, &Self::Vertex)> + '_> {
        let it = self
            .lower_graph
            .iter_vertices()
            .map(|vid| (vid, self.vertex_by_id(&vid).unwrap()));
        Box::new(it)
    }

    fn edge_size(&self) -> usize {
        self.edges.len()
    }

    fn iter_edges(&self) -> Box<dyn Iterator<Item = (EdgeId, &Self::Edge)> + '_> {
        let it = self
            .lower_graph
            .iter_edges()
            .map(|e| (e.id, self.edge_by_id(&e.id).unwrap()));
        Box::new(it)
    }

    fn edges_connecting(
        &self,
        source: &VertexId,
        sink: &VertexId,
    ) -> Box<dyn Iterator<Item = (EdgeId, &Self::Edge)> + '_> {
        let it = self
            .lower_graph
            .edges_connecting(source, sink)
            .map(|e| (e.id, self.edge_by_id(&e.id).unwrap()));
        Box::new(it)
    }

    fn in_edges(&self, vid: &VertexId) -> Box<dyn Iterator<Item = (EdgeId, &Self::Edge)> + '_> {
        let it = self
            .lower_graph
            .in_edges(vid)
            .map(|e| (e.id, self.edge_by_id(&e.id).unwrap()));
        Box::new(it)
    }

    fn out_edges(&self, vid: &VertexId) -> Box<dyn Iterator<Item = (EdgeId, &Self::Edge)> + '_> {
        let it = self
            .lower_graph
            .out_edges(vid)
            .map(|e| (e.id, self.edge_by_id(&e.id).unwrap()));
        Box::new(it)
    }
}
