use crate::graph::*;
use petgraph::{
    graph::{EdgeIndex, NodeIndex},
    stable_graph::StableUnGraph,
    visit::EdgeRef,
};
use std::collections::BTreeSet;

#[derive(Clone)]
pub struct AdjacentListGraph(StableUnGraph<(), (VertexId, VertexId), usize>);

impl DirectedOrNot for AdjacentListGraph {
    const DIRECTED_OR_NOT: bool = false;
}

impl GrowableGraph for AdjacentListGraph {
    fn new() -> Self {
        Self(StableUnGraph::<(), (VertexId, VertexId), usize>::with_capacity(0, 0))
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

impl EdgeShrinkableGraph for AdjacentListGraph {
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

impl VertexShrinkableGraph for AdjacentListGraph {
    fn remove_vertex(&mut self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + 'static> {
        let a = NodeIndex::new(v.to_raw());
        let res: BTreeSet<Edge> = self
            .0
            .edges(a)
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

impl QueryableGraph for AdjacentListGraph {
    fn vertex_size(&self) -> usize {
        self.0.node_count()
    }

    fn iter_vertices(&self) -> Box<dyn Iterator<Item = VertexId> + '_> {
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

    fn iter_edges(&self) -> Box<dyn Iterator<Item = Edge> + '_> {
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

    fn find_edge(&self, e: &EdgeId) -> Option<Edge> {
        let eidx = EdgeIndex::new(e.to_raw());
        self.0.edge_weight(eidx).map(|(src, sink)| Edge {
            id: *e,
            source: *src,
            sink: *sink,
        })
    }

    fn in_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_> {
        let nidx = NodeIndex::new(v.to_raw());
        let it = self.0.edges(nidx).map(|x| {
            let id = EdgeId::new(x.id().index());
            let source = VertexId::new(x.source().index());
            let sink = VertexId::new(x.target().index());
            Edge { id, source, sink }
        });
        Box::new(it)
    }

    fn out_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_> {
        let it = self.in_edges(v).map(|e| Edge {
            id: e.id,
            source: e.sink,
            sink: e.source,
        });
        Box::new(it)
    }

    fn edges_connecting(
        &self,
        source: &VertexId,
        sink: &VertexId,
    ) -> Box<dyn Iterator<Item = Edge> + '_> {
        let src = NodeIndex::new(source.to_raw());
        let snk = NodeIndex::new(sink.to_raw());
        let it = self.0.edges_connecting(src, snk).map(|x| {
            let id = EdgeId::new(x.id().index());
            let source = VertexId::new(x.source().index());
            let sink = VertexId::new(x.target().index());
            Edge { id, source, sink }
        });
        Box::new(it)
    }
}
