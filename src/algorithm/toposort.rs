use crate::graph::*;
use ahash::RandomState;
use keyed_priority_queue::KeyedPriorityQueue;
use std::cmp::Reverse;

pub trait TopologicalSort
where
    Self: QueryableGraph + Sized,
{
    fn toposort(&self) -> Box<dyn Iterator<Item = VertexId> + '_> {
        Box::new(ToposortIter::new(self))
    }
}

impl<G: QueryableGraph> TopologicalSort for G {}

struct ToposortIter<'a, G>
where
    G: QueryableGraph,
{
    graph: ShadowedSubgraph<'a, G>,
    degree_queue: KeyedPriorityQueue<VertexId, Reverse<usize>, RandomState>,
}

impl<'a, G> Iterator for ToposortIter<'a, G>
where
    G: QueryableGraph,
{
    type Item = VertexId;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((vert, in_degree)) = self.degree_queue.pop() {
            if in_degree.0 > 0 {
                None
            } else {
                for e in self.graph.remove_vertex(&vert) {
                    let in_degree = self.degree_queue.get_priority(&e.sink).unwrap();
                    self.degree_queue
                        .set_priority(&e.sink, Reverse(in_degree.0 - 1))
                        .unwrap();
                }
                Some(vert)
            }
        } else {
            None
        }
    }
}

impl<'a, G> ToposortIter<'a, G>
where
    G: QueryableGraph,
{
    fn new(graph: &'a G) -> Self {
        let mut res = Self {
            graph: ShadowedSubgraph::new(graph),
            degree_queue: KeyedPriorityQueue::with_capacity_and_hasher(
                graph.vertex_size(),
                RandomState::new(),
            ),
        };
        for v in graph.iter_vertices() {
            let in_degree = graph.in_edges(&v).count();
            res.degree_queue.push(v, Reverse(in_degree));
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::super::SimpleCycles;
    use super::*;
    use crate::graph::directed::*;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn toposort(ops: Ops) {
        let graph_from_ops: MappedGraph<TreeBackedGraph> = (&ops).into();
        let mut graph = graph_from_ops.graph;
        loop {
            let to_remove: Vec<_> = graph
                .simple_cycles()
                .map(|mut cyc| cyc.next().unwrap())
                .collect();
            if to_remove.is_empty() {
                break;
            }
            for e in to_remove.into_iter() {
                graph.remove_edge(&e.id);
            }
        }
        let mut cloned_graph = graph.clone();
        for v in graph.toposort() {
            assert_eq!(cloned_graph.in_edges(&v).collect::<Vec<_>>(), vec![]);
            let _ = cloned_graph.remove_vertex(&v);
        }
        assert_eq!(cloned_graph.vertex_size(), 0);
    }
}
