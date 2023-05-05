use crate::graph::*;
use ahash::RandomState;
use std::collections::HashSet;

/// A subgraph with selected vertices and edges.
///
/// Vertices and edges can be covered by removing them from this subgraph.
/// Thus, while removing vertices and/or edges from this subgraph,
/// the underlying graph will be kept unchanged.
pub struct SelectedSubgraph<'a, G> {
    lower_graph: &'a G,
    selected_vertices: HashSet<VertexId, RandomState>,
    selected_edges: HashSet<EdgeId, RandomState>,
}

impl<'a, G> DirectedOrNot for SelectedSubgraph<'a, G>
where
    G: DirectedOrNot,
{
    const DIRECTED_OR_NOT: bool = G::DIRECTED_OR_NOT;
}

impl<'a, G> Subgraph for SelectedSubgraph<'a, G>
where
    G: QueryableGraph,
{
    type LowerGraph = &'a G;

    fn new(lower_graph: Self::LowerGraph) -> Self {
        Self {
            lower_graph,
            selected_vertices: HashSet::with_hasher(RandomState::new()),
            selected_edges: HashSet::with_hasher(RandomState::new()),
        }
    }

    fn uncover_vertex(&mut self, v: VertexId) -> &mut Self {
        self.selected_vertices.insert(v);
        self
    }

    fn uncover_edge(&mut self, e: EdgeId) -> &mut Self {
        if let Some(edge) = self.lower_graph.find_edge(&e) {
            self.selected_edges.insert(e);
            self.uncover_vertex(edge.source).uncover_vertex(edge.sink);
        }
        self
    }
}

impl<'a, G> QueryableGraph for SelectedSubgraph<'a, G>
where
    G: QueryableGraph,
{
    fn vertex_size(&self) -> usize {
        self.selected_vertices.len()
    }

    fn iter_vertices(&self) -> Box<dyn Iterator<Item = VertexId> + '_> {
        let it = self.selected_vertices.iter().copied();
        Box::new(it)
    }

    fn contains_vertex(&self, v: &VertexId) -> bool {
        self.selected_vertices.contains(v)
    }

    fn edges_connecting(
        &self,
        source: &VertexId,
        sink: &VertexId,
    ) -> Box<dyn Iterator<Item = crate::graph::Edge> + '_> {
        let it = self
            .lower_graph
            .edges_connecting(source, sink)
            .filter(|e| self.selected_edges.contains(&e.id));
        Box::new(it)
    }

    fn edge_size(&self) -> usize {
        self.selected_edges.len()
    }

    fn iter_edges(&self) -> Box<dyn Iterator<Item = crate::graph::Edge> + '_> {
        let it = self
            .selected_edges
            .iter()
            .map(|e| self.lower_graph.find_edge(e).unwrap());
        Box::new(it)
    }

    fn contains_edge(&self, e: &EdgeId) -> bool {
        self.selected_edges.contains(e)
    }

    fn find_edge(&self, e: &EdgeId) -> Option<crate::graph::Edge> {
        if !self.selected_edges.contains(e) {
            return None;
        }
        self.lower_graph.find_edge(e)
    }

    fn in_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = crate::graph::Edge> + '_> {
        let it = self
            .lower_graph
            .in_edges(v)
            .filter(|e| self.selected_edges.contains(&e.id));
        Box::new(it)
    }

    fn out_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = crate::graph::Edge> + '_> {
        let it = self
            .lower_graph
            .out_edges(v)
            .filter(|e| self.selected_edges.contains(&e.id));
        Box::new(it)
    }
}

impl<'a, G> EdgeShrinkableGraph for SelectedSubgraph<'a, G>
where
    G: QueryableGraph,
{
    fn remove_edge(&mut self, edge: &EdgeId) -> Option<crate::graph::Edge> {
        if self.selected_edges.remove(edge) {
            self.lower_graph.find_edge(edge)
        } else {
            None
        }
    }
}

impl<'a, G> VertexShrinkableGraph for SelectedSubgraph<'a, G>
where
    G: QueryableGraph,
{
    fn remove_vertex(
        &mut self,
        vertex: &VertexId,
    ) -> Box<dyn Iterator<Item = crate::graph::Edge> + 'static> {
        if self.selected_vertices.remove(vertex) {
            let edges: HashSet<Edge, RandomState> = self
                .lower_graph
                .in_edges(vertex)
                .chain(self.lower_graph.out_edges(vertex))
                .filter(|e| self.selected_edges.contains(&e.id))
                .collect();
            for e in edges.iter() {
                self.selected_edges.remove(&e.id);
            }
            Box::new(edges.into_iter())
        } else {
            Box::new(std::iter::empty())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::directed::*;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn selected_subgraph(ops: Ops) {
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
                graph: {
                    let mut g = SelectedSubgraph::new(&base.graph);
                    for e in base.graph.iter_edges() {
                        g.uncover_edge(e.id);
                    }
                    for v in base.graph.iter_vertices() {
                        g.uncover_vertex(v);
                    }
                    g
                },
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
