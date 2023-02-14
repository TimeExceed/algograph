use crate::graph::{digraph::*, Edge, EdgeId, VertexId};
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
where
    G: QueryableGraph,
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

    fn adjacent(
        &self,
        source: &VertexId,
        sink: &VertexId,
    ) -> Box<dyn Iterator<Item = crate::graph::Edge> + '_> {
        let it = self
            .lower_graph
            .adjacent(source, sink)
            .filter(|e| self.selected_edges.contains(&e.id));
        Box::new(it)
    }

    fn edge_size(&self) -> usize {
        self.selected_edges.len()
    }

    fn edges(&self) -> Box<dyn Iterator<Item = crate::graph::Edge> + '_> {
        let it = self
            .selected_edges
            .iter()
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
            self.lower_graph.edge(edge)
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
    use crate::graph::digraph::r#impl::{tests::*, TreeBackedGraph};
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
        let remove_ops = Ops {
            ops: remove.clone(),
        };
        let base: OpsFormedGraph<TreeBackedGraph> = (&add_ops).into();
        let oracle = {
            let mut oracle = base.clone();
            oracle.apply(&remove_ops);
            oracle
        };
        let trial = {
            let mut trial = OpsFormedGraph {
                graph: SelectedSubgraph::new(
                    &base.graph,
                    base.graph.edges().map(|e| e.id),
                    base.graph.vertices(),
                ),
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
