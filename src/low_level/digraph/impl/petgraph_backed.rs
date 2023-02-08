use crate::low_level::{digraph::*, *};
use petgraph::{
    graph::{EdgeIndex, NodeIndex},
    stable_graph::StableDiGraph,
    visit::EdgeRef,
    Direction,
};
use std::collections::BTreeSet;

#[derive(Clone)]
pub struct PetgraphBackedGraph(StableDiGraph<(), (VertexId, VertexId), usize>);

impl GrowableGraph for PetgraphBackedGraph {
    fn new() -> Self {
        Self(StableDiGraph::<(), (VertexId, VertexId), usize>::with_capacity(0, 0))
    }

    fn add_vertex(&mut self) -> VertexId {
        let vid = self.0.add_node(());
        VertexId::new(vid.index())
    }

    fn add_edge(&mut self, source: VertexId, sink: VertexId) -> EdgeId {
        let a = NodeIndex::new(source.to_raw());
        let b = NodeIndex::new(sink.to_raw());
        let eid = self.0.add_edge(a, b, (source, sink));
        EdgeId::new(eid.index())
    }
}

impl EdgeShrinkableGraph for PetgraphBackedGraph {
    fn remove_edge(&mut self, edge: &EdgeId) -> Option<Edge> {
        let pg_eidx = EdgeIndex::new(edge.to_raw());
        if let Some((src, sink)) = self.0.remove_edge(pg_eidx) {
            Some(Edge {
                id: *edge,
                source: src,
                sink,
            })
        } else {
            None
        }
    }
}

impl VertexShrinkableGraph for PetgraphBackedGraph {
    fn remove_vertex(&mut self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + 'static> {
        let a = NodeIndex::new(v.to_raw());
        let res: BTreeSet<Edge> = [Direction::Incoming, Direction::Outgoing]
            .into_iter()
            .map(|dir| self.0.edges_directed(a, dir))
            .flatten()
            .map(|e| {
                let (src, sink) = e.weight();
                let eid = EdgeId::new(e.id().index());
                Edge {
                    id: eid,
                    source: *src,
                    sink: *sink,
                }
            })
            .collect();
        self.0.remove_node(a);
        Box::new(res.into_iter())
    }
}

impl QueryableGraph for PetgraphBackedGraph {
    fn vertex_size(&self) -> usize {
        self.0.node_count()
    }

    fn vertices(&self) -> Box<dyn Iterator<Item = VertexId> + '_> {
        let it = self.0.node_indices().map(|x| VertexId::new(x.index()));
        Box::new(it)
    }

    fn contains_vertex(&self, v: &VertexId) -> bool {
        let nidx = NodeIndex::new(v.to_raw());
        self.0.contains_node(nidx)
    }

    fn edge_size(&self) -> usize {
        self.0.edge_count()
    }

    fn edges(&self) -> Box<dyn Iterator<Item = Edge> + '_> {
        let it = self.0.edge_indices().map(|x| {
            let id = EdgeId::new(x.index());
            let (source, sink) = self.0.edge_weight(x).unwrap();
            Edge {
                id,
                source: *source,
                sink: *sink,
            }
        });
        Box::new(it)
    }

    fn contains_edge(&self, e: &EdgeId) -> bool {
        let eidx = EdgeIndex::new(e.to_raw());
        self.0.edge_weight(eidx).is_some()
    }

    fn edge(&self, e: &EdgeId) -> Option<Edge> {
        let eidx = EdgeIndex::new(e.to_raw());
        self.0.edge_weight(eidx).map(|(src, sink)| Edge {
            id: *e,
            source: *src,
            sink: *sink,
        })
    }

    fn in_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_> {
        let nidx = NodeIndex::new(v.to_raw());
        let it = self.0.edges_directed(nidx, Direction::Incoming).map(|x| {
            let id = EdgeId::new(x.id().index());
            let source = VertexId::new(x.source().index());
            let sink = VertexId::new(x.target().index());
            Edge { id, source, sink }
        });
        Box::new(it)
    }

    fn out_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_> {
        let nidx = NodeIndex::new(v.to_raw());
        let it = self.0.edges_directed(nidx, Direction::Outgoing).map(|x| {
            let id = EdgeId::new(x.id().index());
            let source = VertexId::new(x.source().index());
            let sink = VertexId::new(x.target().index());
            Edge { id, source, sink }
        });
        Box::new(it)
    }

    fn adjacent(&self, source: &VertexId, sink: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_> {
        let sink = *sink;
        let it = self.out_edges(source).filter(move |e| e.sink == sink);
        Box::new(it)
    }
}
