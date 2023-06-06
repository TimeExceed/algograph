use crate::graph::*;
use ahash::RandomState;
use std::collections::HashSet;

/// A subgraph by shadowing some of vertices and edges in the underlying graph.
///
/// Removing vertices and edges from a [ShadowedSubgraph] just shadows them.
/// Therefore, shrinking a [ShadowedSubgraph] keeps the underlying graph unchanged.
pub struct ShadowedSubgraph<'a, G> {
    lower_graph: &'a G,
    shadowed_vertices: HashSet<VertexId, RandomState>,
    shadowed_edges: HashSet<EdgeId, RandomState>,
}

impl<'a, G> DirectedOrNot for ShadowedSubgraph<'a, G>
where
    G: DirectedOrNot,
{
    const DIRECTED_OR_NOT: bool = G::DIRECTED_OR_NOT;
}

impl<'a, G> Subgraph for ShadowedSubgraph<'a, G>
where
    G: QueryableGraph,
{
    type LowerGraph = &'a G;

    fn new(lower_graph: Self::LowerGraph) -> Self {
        Self {
            lower_graph,
            shadowed_edges: HashSet::with_hasher(RandomState::new()),
            shadowed_vertices: HashSet::with_hasher(RandomState::new()),
        }
    }

    fn disclose_edge(&mut self, e: EdgeId) -> &mut Self {
        if let Some(edge) = self.lower_graph.find_edge(&e) {
            self.shadowed_edges.remove(&e);
            self.disclose_vertex(edge.source).disclose_vertex(edge.sink);
        }
        self
    }

    fn disclose_vertex(&mut self, v: VertexId) -> &mut Self {
        self.shadowed_vertices.remove(&v);
        self
    }
}

impl<'a, G> EdgeShrinkableGraph for ShadowedSubgraph<'a, G>
where
    G: QueryableGraph,
{
    fn remove_edge(&mut self, edge: &EdgeId) -> Option<Edge> {
        if self.shadowed_edges.contains(edge) {
            return None;
        }
        if let Some(e) = self.lower_graph.find_edge(edge) {
            self.shadowed_edges.insert(e.id);
            Some(e)
        } else {
            None
        }
    }
}

impl<'a, G> VertexShrinkableGraph for ShadowedSubgraph<'a, G>
where
    G: QueryableGraph,
{
    fn remove_vertex(&mut self, vertex: &VertexId) -> Box<dyn Iterator<Item = Edge> + 'static> {
        if self.shadowed_vertices.contains(vertex) {
            return Box::new(std::iter::empty());
        }
        if !self.lower_graph.contains_vertex(vertex) {
            return Box::new(std::iter::empty());
        }
        let edges = self
            .lower_graph
            .in_edges(vertex)
            .chain(self.lower_graph.out_edges(vertex));
        let mut res = vec![];
        for e in edges {
            if self.remove_edge(&e.id).is_some() {
                res.push(e);
            }
        }
        self.shadowed_vertices.insert(*vertex);
        Box::new(res.into_iter())
    }
}

impl<'a, G> QueryableGraph for ShadowedSubgraph<'a, G>
where
    G: QueryableGraph,
{
    fn vertex_size(&self) -> usize {
        self.lower_graph.vertex_size() - self.shadowed_vertices.len()
    }

    fn iter_vertices(&self) -> Box<dyn Iterator<Item = VertexId> + '_> {
        let it = self
            .lower_graph
            .iter_vertices()
            .filter(|v| !self.shadowed_vertices.contains(v));
        Box::new(it)
    }

    fn contains_vertex(&self, v: &VertexId) -> bool {
        !self.shadowed_vertices.contains(v) && self.lower_graph.contains_vertex(v)
    }

    fn edges_connecting(
        &self,
        source: &VertexId,
        sink: &VertexId,
    ) -> Box<dyn Iterator<Item = Edge> + '_> {
        if self.shadowed_vertices.contains(source) {
            return Box::new(std::iter::empty());
        }
        if self.shadowed_vertices.contains(sink) {
            return Box::new(std::iter::empty());
        }
        let it = self
            .lower_graph
            .edges_connecting(source, sink)
            .filter(|e| !self.shadowed_edges.contains(&e.id));
        Box::new(it)
    }

    fn edge_size(&self) -> usize {
        self.lower_graph.edge_size() - self.shadowed_edges.len()
    }

    fn iter_edges(&self) -> Box<dyn Iterator<Item = Edge> + '_> {
        let it = self
            .lower_graph
            .iter_edges()
            .filter(|e| !self.shadowed_edges.contains(&e.id));
        Box::new(it)
    }

    fn contains_edge(&self, e: &EdgeId) -> bool {
        !self.shadowed_edges.contains(e) && self.lower_graph.contains_edge(e)
    }

    fn find_edge(&self, e: &EdgeId) -> Option<Edge> {
        if self.shadowed_edges.contains(e) {
            return None;
        }
        self.lower_graph.find_edge(e)
    }

    fn in_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_> {
        if self.shadowed_vertices.contains(v) {
            return Box::new(std::iter::empty());
        }
        let it = self
            .lower_graph
            .in_edges(v)
            .filter(|e| !self.shadowed_edges.contains(&e.id));
        Box::new(it)
    }

    fn out_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_> {
        if self.shadowed_vertices.contains(v) {
            return Box::new(std::iter::empty());
        }
        let it = self
            .lower_graph
            .out_edges(v)
            .filter(|e| !self.shadowed_edges.contains(&e.id));
        Box::new(it)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::directed::*;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn shadowed_subgraph(ops: Ops) {
        let (add, remove): (Vec<_>, Vec<_>) = ops.iter().cloned().partition(|op| match op {
            Op::AddVertex(_) => true,
            Op::AddEdge(_) => true,
            Op::RemoveVertex(_) => false,
            Op::RemoveEdge(_) => false,
        });
        let add_ops = Ops { ops: add };
        let remove_ops = Ops { ops: remove };
        let base: MappedGraph<TreeBackedGraph> = (&add_ops).into();
        let oracle = {
            let mut oracle = base.clone();
            oracle.apply(&remove_ops);
            oracle
        };
        let trial = {
            let mut trial = MappedGraph {
                graph: ShadowedSubgraph::new(&base.graph),
                vmap: base.vmap.clone(),
                emap: base.emap.clone(),
            };
            for op in remove_ops.iter() {
                match op {
                    Op::RemoveVertex(vid) => {
                        if let Some(my_vid) = trial.vmap.get_by_right(vid) {
                            for e in trial.graph.remove_vertex(my_vid) {
                                trial.emap.remove_by_left(&e.id);
                            }
                        }
                    }
                    Op::RemoveEdge(eid) => {
                        if let Some(my_eid) = trial.emap.get_by_right(eid) {
                            trial.graph.remove_edge(my_eid);
                        }
                    }
                    _ => unreachable!(),
                }
            }
            trial
        };
        assert_eq!(oracle, trial);
    }
}
