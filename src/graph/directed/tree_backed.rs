use crate::graph::*;
use std::collections::{BTreeMap, BTreeSet};

/// A directed graph with balanced computational complexity.
///
/// |                    | Complexity                                                                                   |
/// | ------------------ | -------------------------------------------------------------------------------------------- |
/// | `add_vertex`       | $O(\log \|V\|)$                                                                              |
/// | `add_edge`         | $O(\log \|V\| + \log \|E\|)$                                                                 |
/// | `remove_edge`      | $O(\log \|E\|)$                                                                              |
/// | `remove_vertex`    | $O(\log \|V\| + \|E'\|)$, where $E'$ is the set of edges connecting to the vertex to remove. |
/// | `vertex_size`      | $O(1)$                                                                                       |
/// | `iter_vertices`    | amortized $O(1)$ and $O(\log \|V\|)$ in the worst cases.                                     |
/// | `contains_vertex`  | $O(\log \|V\|)$                                                                              |
/// | `edge_size`        | $O(1)$                                                                                       |
/// | `iter_edges`       | amortized $O(1)$ and $O(\log \|E\|)$ in the worst cases.                                     |
/// | `contains_edge`    | $O(\log \|E\|)$                                                                              |
/// | `find_edge`        | $O(\log \|E\|)$                                                                              |
/// | `edges_connecting` | returns in $O(\log \|E\|)$. amortized $O(1)$ and $O(\log \|E\|)$ in the worst cases on each call to `.next`.|
/// | `in_edges`         | returns in $O(\log \|E\|)$. amortized $O(1)$ and $O(\log \|E\|)$ in the worst cases on each call to `.next`.|
/// | `out_edges`        | returns in $O(\log \|E\|)$. amortized $O(1)$ and $O(\log \|E\|)$ in the worst cases on each call to `.next`.|
#[derive(Clone)]
pub struct TreeBackedGraph {
    vid_factory: VertexIdFactory,
    eid_factory: EdgeIdFactory,
    vertices: BTreeSet<VertexId>,
    edges: BTreeMap<EdgeId, (VertexId, VertexId)>,
    in_edges: BTreeSet<(VertexId, VertexId, EdgeId)>,
    out_edges: BTreeSet<(VertexId, VertexId, EdgeId)>,
}

impl DirectedOrNot for TreeBackedGraph {
    const DIRECTED_OR_NOT: bool = true;
}

impl std::fmt::Debug for TreeBackedGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TreeBackedGraph {{")?;
        for v in self.vertices.iter() {
            writeln!(f, "{:?}:", v)?;
            for e in self.out_edges(v) {
                writeln!(f, "  -> {:?} by {:?}", e.sink, e.id)?;
            }
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl GrowableGraph for TreeBackedGraph {
    fn new() -> Self {
        Self {
            vid_factory: VertexIdFactory::new(),
            eid_factory: EdgeIdFactory::new(),
            vertices: BTreeSet::new(),
            edges: BTreeMap::new(),
            in_edges: BTreeSet::new(),
            out_edges: BTreeSet::new(),
        }
    }

    fn add_vertex(&mut self) -> VertexId {
        let vid = self.vid_factory.one_more();
        self.vertices.insert(vid);
        vid
    }

    fn add_edge(&mut self, source: VertexId, sink: VertexId) -> EdgeId {
        debug_assert!(self.vertices.contains(&source));
        debug_assert!(self.vertices.contains(&sink));
        let eid = self.eid_factory.one_more();
        self.edges.insert(eid, (source, sink));
        self.in_edges.insert((sink, source, eid));
        self.out_edges.insert((source, sink, eid));
        eid
    }
}

impl EdgeShrinkableGraph for TreeBackedGraph {
    fn remove_edge(&mut self, edge: &EdgeId) -> Option<Edge> {
        match self.edges.remove(edge) {
            None => return None,
            Some((src, snk)) => {
                self.in_edges.remove(&(snk, src, *edge));
                self.out_edges.remove(&(src, snk, *edge));
                Some(Edge {
                    id: *edge,
                    source: src,
                    sink: snk,
                })
            }
        }
    }
}

impl VertexShrinkableGraph for TreeBackedGraph {
    fn remove_vertex(&mut self, vertex: &VertexId) -> Box<dyn Iterator<Item = Edge> + 'static> {
        if !self.vertices.remove(vertex) {
            return Box::new(std::iter::empty());
        }
        let start = (*vertex, VertexId::MIN, EdgeId::MIN);
        let end = (vertex.next(), VertexId::MIN, EdgeId::MIN);
        let ins = self
            .in_edges
            .range(start..end)
            .map(|(snk, src, edge)| Edge {
                id: *edge,
                source: *src,
                sink: *snk,
            });
        let outs = self
            .out_edges
            .range(start..end)
            .map(|(src, snk, edge)| Edge {
                id: *edge,
                source: *src,
                sink: *snk,
            });
        let res: BTreeSet<_> = ins.chain(outs).collect();
        for x in res.iter() {
            self.remove_edge(&x.id);
        }
        Box::new(res.into_iter())
    }
}

impl QueryableGraph for TreeBackedGraph {
    fn vertex_size(&self) -> usize {
        self.vertices.len()
    }

    fn iter_vertices(&self) -> Box<dyn Iterator<Item = VertexId> + '_> {
        Box::new(self.vertices.iter().copied())
    }

    fn contains_vertex(&self, v: &VertexId) -> bool {
        self.vertices.contains(v)
    }

    fn edge_size(&self) -> usize {
        self.edges.len()
    }

    fn iter_edges(&self) -> Box<dyn Iterator<Item = Edge> + '_> {
        Box::new(self.edges.iter().map(|(e, (src, snk))| Edge {
            id: *e,
            source: *src,
            sink: *snk,
        }))
    }

    fn contains_edge(&self, e: &EdgeId) -> bool {
        self.edges.contains_key(e)
    }

    fn find_edge(&self, e: &EdgeId) -> Option<Edge> {
        self.edges.get(e).map(|(src, snk)| Edge {
            id: *e,
            source: *src,
            sink: *snk,
        })
    }

    fn in_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_> {
        let start = (*v, VertexId::MIN, EdgeId::MIN);
        let end = (v.next(), VertexId::MIN, EdgeId::MIN);
        let it = self.in_edges.range(start..end).map(|(snk, src, e)| Edge {
            id: *e,
            source: *src,
            sink: *snk,
        });
        Box::new(it)
    }

    fn out_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_> {
        let start = (*v, VertexId::MIN, EdgeId::MIN);
        let end = (v.next(), VertexId::MIN, EdgeId::MIN);
        let it = self.out_edges.range(start..end).map(|(src, snk, e)| Edge {
            id: *e,
            source: *src,
            sink: *snk,
        });
        Box::new(it)
    }

    fn edges_connecting<'a, 'b>(
        &'a self,
        source: &'b VertexId,
        sink: &'b VertexId,
    ) -> Box<dyn Iterator<Item = Edge> + 'a> {
        let source = *source;
        let sink = *sink;
        let start = (source, sink, EdgeId::MIN);
        let end = (source, sink, EdgeId::MAX);
        let it = self
            .out_edges
            .range(start..=end)
            .map(move |(_, _, eid)| Edge {
                id: *eid,
                source,
                sink,
            });
        Box::new(it)
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::{directed::*, MappedGraph};
    use quickcheck_macros::*;

    #[quickcheck]
    fn tree_backed_gen(ops: Ops) {
        let oracle: MappedGraph<AdjacentListGraph> = (&ops).into();
        let trial: MappedGraph<TreeBackedGraph> = (&ops).into();
        assert_eq!(oracle, trial);
    }
}
