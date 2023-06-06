use crate::graph::*;
use bimap::BiHashMap;

/// This wraps a graph and mappings of vertices and edges from another graph.
#[derive(Clone)]
pub struct MappedGraph<G> {
    pub graph: G,
    pub vmap: BiHashMap<VertexId, VertexId>,
    pub emap: BiHashMap<EdgeId, EdgeId>,
}

impl<G> QueryableGraph for MappedGraph<G>
where
    G: QueryableGraph,
{
    fn vertex_size(&self) -> usize {
        self.graph.vertex_size()
    }

    fn iter_vertices(&self) -> Box<dyn Iterator<Item = VertexId> + '_> {
        let it = self
            .graph
            .iter_vertices()
            .map(|l| *self.vmap.get_by_left(&l).unwrap());
        Box::new(it)
    }

    fn contains_vertex(&self, v: &VertexId) -> bool {
        if let Some(l) = self.vmap.get_by_right(v) {
            self.graph.contains_vertex(l)
        } else {
            false
        }
    }

    fn edge_size(&self) -> usize {
        self.graph.edge_size()
    }

    fn iter_edges(&self) -> Box<dyn Iterator<Item = Edge> + '_> {
        let it = self.graph.iter_edges().map(|e| Edge {
            id: *self.emap.get_by_left(&e.id).unwrap(),
            source: *self.vmap.get_by_left(&e.source).unwrap(),
            sink: *self.vmap.get_by_left(&e.sink).unwrap(),
        });
        Box::new(it)
    }

    fn contains_edge(&self, e: &EdgeId) -> bool {
        if let Some(e) = self.emap.get_by_right(e) {
            self.graph.contains_edge(e)
        } else {
            false
        }
    }

    fn find_edge(&self, e: &EdgeId) -> Option<Edge> {
        if let Some(l) = self.emap.get_by_right(e) {
            self.graph.find_edge(l).map(|le| Edge {
                id: *e,
                source: *self.vmap.get_by_left(&le.source).unwrap(),
                sink: *self.vmap.get_by_left(&le.sink).unwrap(),
            })
        } else {
            None
        }
    }

    fn edges_connecting(
        &self,
        source: &VertexId,
        sink: &VertexId,
    ) -> Box<dyn Iterator<Item = Edge> + '_> {
        let src = *source;
        let snk = *sink;
        match (self.vmap.get_by_right(source), self.vmap.get_by_left(sink)) {
            (Some(lsrc), Some(lsnk)) => {
                let it = self.graph.edges_connecting(lsrc, lsnk).map(move |e| Edge {
                    id: *self.emap.get_by_left(&e.id).unwrap(),
                    source: src,
                    sink: snk,
                });
                Box::new(it)
            }
            _ => Box::new(std::iter::empty()),
        }
    }

    fn in_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_> {
        let sink = *v;
        if let Some(lv) = self.vmap.get_by_right(v) {
            let it = self.graph.in_edges(lv).map(move |e| Edge {
                id: *self.emap.get_by_left(&e.id).unwrap(),
                source: *self.vmap.get_by_left(&e.source).unwrap(),
                sink,
            });
            Box::new(it)
        } else {
            Box::new(std::iter::empty())
        }
    }

    fn out_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_> {
        let source = *v;
        if let Some(lv) = self.vmap.get_by_right(v) {
            let it = self.graph.out_edges(lv).map(move |e| Edge {
                id: *self.emap.get_by_left(&e.id).unwrap(),
                source,
                sink: *self.vmap.get_by_left(&e.sink).unwrap(),
            });
            Box::new(it)
        } else {
            Box::new(std::iter::empty())
        }
    }
}

impl<G> std::fmt::Debug for MappedGraph<G>
where
    G: QueryableGraph,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.debug())
    }
}

impl<G1, G2> std::cmp::PartialEq<MappedGraph<G2>> for MappedGraph<G1>
where
    G1: QueryableGraph,
    G2: QueryableGraph,
{
    fn eq(&self, other: &MappedGraph<G2>) -> bool {
        if self.graph.vertex_size() != other.graph.vertex_size() {
            return false;
        }
        for vid in self.graph.iter_vertices() {
            let op_vid = self.vmap.get_by_left(&vid).unwrap();
            if let Some(other_vid) = other.vmap.get_by_right(op_vid) {
                if !other.graph.contains_vertex(other_vid) {
                    return false;
                }
            } else {
                return false;
            }
        }

        if self.graph.edge_size() != other.graph.edge_size() {
            return false;
        }
        for e in self.graph.iter_edges() {
            if !self.edge_in_other(other, e) {
                return false;
            }
        }
        for vid in self.graph.iter_vertices() {
            for e in self.graph.in_edges(&vid) {
                if !self.edge_in_other(other, e) {
                    return false;
                }
            }
            for e in self.graph.out_edges(&vid) {
                if !self.edge_in_other(other, e) {
                    return false;
                }
            }
        }
        for vid in other.graph.iter_vertices() {
            for e in other.graph.in_edges(&vid) {
                if !other.edge_in_other(self, e) {
                    return false;
                }
            }
            for e in other.graph.out_edges(&vid) {
                if !other.edge_in_other(self, e) {
                    return false;
                }
            }
        }
        true
    }
}

impl<G> Eq for MappedGraph<G> where G: QueryableGraph {}

impl<G> Default for MappedGraph<G>
where
    G: GrowableGraph + EdgeShrinkableGraph + VertexShrinkableGraph,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<G> MappedGraph<G>
where
    G: GrowableGraph + EdgeShrinkableGraph + VertexShrinkableGraph,
{
    pub fn new() -> Self {
        Self {
            graph: G::new(),
            vmap: BiHashMap::new(),
            emap: BiHashMap::new(),
        }
    }

    #[cfg(test)]
    pub fn apply(&mut self, ops: &directed::Ops) {
        for op in ops.iter() {
            match op {
                directed::Op::AddVertex(vid) => {
                    let my_vid = self.graph.add_vertex();
                    self.vmap.insert(my_vid, *vid);
                }
                directed::Op::RemoveVertex(vid) => {
                    if let Some(my_vid) = self.vmap.get_by_right(vid) {
                        for e in self.graph.remove_vertex(my_vid) {
                            self.emap.remove_by_left(&e.id);
                        }
                    }
                }
                directed::Op::AddEdge((source, sink, eid)) => {
                    match (self.vmap.get_by_right(source), self.vmap.get_by_right(sink)) {
                        (Some(my_src), Some(my_sink)) => {
                            let my_eid = self.graph.add_edge(*my_src, *my_sink);
                            self.emap.insert(my_eid, *eid);
                        }
                        _ => {}
                    }
                }
                directed::Op::RemoveEdge(eid) => {
                    if let Some(my_eid) = self.emap.get_by_right(eid) {
                        self.graph.remove_edge(my_eid);
                    }
                }
            }
        }
    }
}

impl<G1> MappedGraph<G1>
where
    G1: QueryableGraph,
{
    pub fn transform<G2>(&self) -> MappedGraph<G2>
    where
        G2: GrowableGraph,
    {
        let mut res = G2::new();
        let mut vmap = BiHashMap::new();
        let mut emap = BiHashMap::new();

        for v in self.graph.iter_vertices() {
            let new_v = res.add_vertex();
            let right_v = self.vmap.get_by_left(&v).unwrap();
            vmap.insert(new_v, *right_v);
        }
        for e in self.graph.iter_edges() {
            let right_src = self.vmap.get_by_left(&e.source).unwrap();
            let right_snk = self.vmap.get_by_left(&e.sink).unwrap();
            let left_src = vmap.get_by_right(right_src).unwrap();
            let left_snk = vmap.get_by_right(right_snk).unwrap();
            let new_e = res.add_edge(*left_src, *left_snk);
            let right_e = self.emap.get_by_left(&e.id).unwrap();
            emap.insert(new_e, *right_e);
        }

        MappedGraph {
            graph: res,
            vmap,
            emap,
        }
    }

    fn edge_in_other<G2>(&self, other: &MappedGraph<G2>, e: Edge) -> bool
    where
        G2: QueryableGraph,
    {
        let op_eid = self.emap.get_by_left(&e.id).unwrap();
        if let Some(other_eid) = other.emap.get_by_right(op_eid) {
            if !other.graph.contains_edge(other_eid) {
                return false;
            }
        } else {
            return false;
        }
        true
    }
}

#[cfg(test)]
impl<G> From<&directed::Ops> for MappedGraph<G>
where
    G: GrowableGraph + EdgeShrinkableGraph + VertexShrinkableGraph,
{
    fn from(ops: &directed::Ops) -> Self {
        let mut res = Self::new();
        res.apply(ops);
        res
    }
}
