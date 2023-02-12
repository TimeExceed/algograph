mod petgraph_backed;
pub use self::petgraph_backed::*;
mod tree_backed;
pub use self::tree_backed::*;

#[cfg(test)]
mod tests {
    use crate::graph::{digraph::*, *};
    use bimap::BiHashMap;
    use rs_quickcheck_util::*;
    use std::collections::BTreeSet;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum Op {
        AddVertex(VertexId),
        RemoveVertex(VertexId),
        AddEdge((VertexId, VertexId, EdgeId)),
        RemoveEdge(EdgeId),
    }

    #[derive(Clone)]
    pub(crate) struct Ops {
        ops: Vec<Op>,
    }

    impl std::fmt::Debug for Ops {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.ops)
        }
    }

    impl Ops {
        pub(crate) fn iter(&self) -> impl Iterator<Item = &Op> + '_ {
            self.ops.iter()
        }
    }

    impl quickcheck::Arbitrary for Ops {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut vid_factory = VertexIdFactory::new();
            let mut eid_factory = EdgeIdFactory::new();
            let mut known_vid = BTreeSet::new();
            let mut known_eid = BTreeSet::new();
            let ops = gen_bytes(g, b"abc.", b'.', 0..)
                .iter()
                .filter_map(|_| match u8::arbitrary(g) % 4 {
                    0 => {
                        let vid = vid_factory.one_more();
                        known_vid.insert(vid);
                        Some(Op::AddVertex(vid))
                    }
                    1 => {
                        if known_vid.is_empty() {
                            None
                        } else {
                            let vid = {
                                let idx = usize::arbitrary(g) % known_vid.len();
                                *known_vid.iter().nth(idx).unwrap()
                            };
                            known_vid.remove(&vid);
                            Some(Op::RemoveVertex(vid))
                        }
                    }
                    2 => {
                        if known_vid.is_empty() {
                            None
                        } else {
                            let src_vid = {
                                let idx = usize::arbitrary(g) % known_vid.len();
                                *known_vid.iter().nth(idx).unwrap()
                            };
                            let sink_vid = {
                                let idx = usize::arbitrary(g) % known_vid.len();
                                *known_vid.iter().nth(idx).unwrap()
                            };
                            let eid = eid_factory.one_more();
                            known_eid.insert(eid);
                            Some(Op::AddEdge((src_vid, sink_vid, eid)))
                        }
                    }
                    3 => {
                        if known_eid.is_empty() {
                            None
                        } else {
                            let eid = {
                                let idx = usize::arbitrary(g) % known_eid.len();
                                *known_eid.iter().nth(idx).unwrap()
                            };
                            known_eid.remove(&eid);
                            Some(Op::RemoveEdge(eid))
                        }
                    }
                    _ => unreachable!(),
                })
                .collect();
            Self { ops }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            let l = self.ops.len();
            let me = self.clone();
            let it = std::iter::successors(Some(l / 2), move |n| {
                let nxt = (n + l) / 2 + 1;
                if nxt >= l {
                    None
                } else {
                    Some(nxt)
                }
            })
            .map(move |n| {
                let mut res = me.clone();
                res.ops = me.ops[0..n].iter().map(|x| x.clone()).collect();
                res
            });
            Box::new(it)
        }
    }

    pub(crate) struct OpsFormedGraph<G> {
        pub(crate) graph: G,
        pub(crate) vmap: BiHashMap<VertexId, VertexId>,
        pub(crate) emap: BiHashMap<EdgeId, EdgeId>,
    }

    impl<G> std::fmt::Debug for OpsFormedGraph<G>
    where
        G: QueryableGraph,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(f, "")?;
            for v in self.graph.vertices() {
                let op_vid = self.vmap.get_by_left(&v).unwrap();
                writeln!(f, "{:?}", op_vid)?;
                for e in self.graph.out_edges(&v) {
                    let op_eid = self.emap.get_by_left(&e.id).unwrap();
                    let op_snk = self.vmap.get_by_left(&e.sink).unwrap();
                    writeln!(f, "  --{:?}-> {:?}", op_eid, op_snk)?;
                }
            }
            Ok(())
        }
    }

    impl<G> From<&Ops> for OpsFormedGraph<G>
    where
        G: GrowableGraph + EdgeShrinkableGraph + VertexShrinkableGraph,
    {
        fn from(ops: &Ops) -> Self {
            let mut vmap = BiHashMap::new();
            let mut emap = BiHashMap::new();
            let mut res = G::new();
            for op in ops.iter() {
                match op {
                    Op::AddVertex(vid) => {
                        let my_vid = res.add_vertex();
                        vmap.insert(my_vid, *vid);
                    }
                    Op::RemoveVertex(vid) => {
                        if let Some(my_vid) = vmap.get_by_right(vid) {
                            for e in res.remove_vertex(my_vid) {
                                emap.remove_by_left(&e.id);
                            }
                        }
                    }
                    Op::AddEdge((source, sink, eid)) => {
                        match (vmap.get_by_right(source), vmap.get_by_right(sink)) {
                            (Some(my_src), Some(my_sink)) => {
                                let my_eid = res.add_edge(*my_src, *my_sink);
                                emap.insert(my_eid, *eid);
                            }
                            _ => {}
                        }
                    }
                    Op::RemoveEdge(eid) => {
                        if let Some(my_eid) = emap.get_by_right(eid) {
                            res.remove_edge(my_eid);
                        }
                    }
                }
            }
            Self {
                graph: res,
                vmap,
                emap,
            }
        }
    }

    impl<G1, G2> std::cmp::PartialEq<OpsFormedGraph<G2>> for OpsFormedGraph<G1>
    where
        G1: QueryableGraph,
        G2: QueryableGraph,
    {
        fn eq(&self, other: &OpsFormedGraph<G2>) -> bool {
            if self.graph.vertex_size() != other.graph.vertex_size() {
                return false;
            }
            for vid in self.graph.vertices() {
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
            for e in self.graph.edges() {
                if !self.edge_in_ohter(other, e) {
                    return false;
                }
            }
            for vid in self.graph.vertices() {
                for e in self.graph.in_edges(&vid) {
                    if !self.edge_in_ohter(other, e) {
                        return false;
                    }
                }
                for e in self.graph.out_edges(&vid) {
                    if !self.edge_in_ohter(other, e) {
                        return false;
                    }
                }
            }
            for vid in other.graph.vertices() {
                for e in other.graph.in_edges(&vid) {
                    if !other.edge_in_ohter(self, e) {
                        return false;
                    }
                }
                for e in other.graph.out_edges(&vid) {
                    if !other.edge_in_ohter(self, e) {
                        return false;
                    }
                }
            }
            true
        }
    }

    impl<G1> OpsFormedGraph<G1>
    where
        G1: QueryableGraph,
    {
        fn edge_in_ohter<G2>(&self, other: &OpsFormedGraph<G2>, e: Edge) -> bool
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
}
