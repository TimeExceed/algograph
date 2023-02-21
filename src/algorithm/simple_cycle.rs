use crate::graph::{
    digraph::{r#impl::ShadowedSubgraph, *},
    *,
};
use ahash::RandomState;
use std::collections::HashMap;

pub trait SimpleCycles
where
    Self: QueryableGraph + Sized,
{
    fn simple_cycles(
        &self,
    ) -> Box<dyn Iterator<Item = Box<dyn Iterator<Item = EdgeId> + '_>> + '_> {
        Box::new(CycleIterator::exhaust(self))
    }

    fn simple_cycles_reachable_from(
        &self,
        vert: &VertexId,
    ) -> Box<dyn Iterator<Item = Box<dyn Iterator<Item = EdgeId> + '_>> + '_> {
        Box::new(CycleIterator::start_from(self, vert))
    }
}

impl<G: QueryableGraph> SimpleCycles for G {}

struct CycleIterator<'a, G>
where
    G: QueryableGraph,
{
    graph: ShadowedSubgraph<'a, G>,
    to_exhaust_vertices: Vec<VertexId>,
    vertex_stack: Vec<VertexId>,
    vertex_backtrack: HashMap<VertexId, usize, RandomState>,
    edge_to_scan: Vec<Edge>,
    edge_stack: Vec<EdgeId>,
}

impl<'a, G> Iterator for CycleIterator<'a, G>
where
    G: QueryableGraph,
{
    type Item = Box<dyn Iterator<Item = EdgeId> + 'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(next_edge) = self.edge_to_scan.pop() {
                loop {
                    assert!(!self.vertex_stack.is_empty());
                    if let Some(top_vert) = self.vertex_stack.last() {
                        if *top_vert == next_edge.source {
                            break;
                        }
                        self.one_step_backward();
                    }
                }
                if let Some(vert_idx) = self.vertex_backtrack.get(&next_edge.sink) {
                    // found a loop
                    let mut res: Vec<EdgeId> =
                        (&self.edge_stack[*vert_idx..]).iter().copied().collect();
                    res.push(next_edge.id);
                    self.graph.remove_edge(&next_edge.id);
                    return Some(Box::new(res.into_iter()));
                } else {
                    self.one_step_forward_along(&next_edge);
                }
            } else {
                while !self.vertex_stack.is_empty() {
                    self.one_step_backward();
                }
                assert!(self.edge_stack.is_empty());
                if let Some(vert) = self.to_exhaust_vertices.pop() {
                    self.push_vertex(vert);
                } else {
                    return None;
                }
            }
        }
    }
}

impl<'a, G> CycleIterator<'a, G>
where
    G: QueryableGraph,
{
    fn exhaust(graph: &'a G) -> Self {
        let mut res = Self::new(graph);
        res.to_exhaust_vertices = graph.vertices().collect();
        res
    }

    fn start_from(graph: &'a G, vert: &VertexId) -> Self {
        let mut res = Self::new(graph);
        res.to_exhaust_vertices = vec![*vert];
        res
    }

    fn new(graph: &'a G) -> Self {
        Self {
            graph: ShadowedSubgraph::new(graph),
            to_exhaust_vertices: vec![],
            vertex_stack: vec![],
            edge_to_scan: vec![],
            vertex_backtrack: HashMap::with_hasher(RandomState::new()),
            edge_stack: vec![],
        }
    }

    fn one_step_forward_along(&mut self, edge: &Edge) {
        assert_eq!(self.vertex_stack.last(), Some(&edge.source));
        self.edge_stack.push(edge.id);
        self.push_vertex(edge.sink);
    }

    fn push_vertex(&mut self, vert: VertexId) {
        let n = self.vertex_stack.len();
        self.vertex_stack.push(vert);
        self.vertex_backtrack.insert(vert, n);
        for e in self.graph.out_edges(&vert) {
            self.edge_to_scan.push(e);
        }
    }

    fn one_step_backward(&mut self) {
        if let Some(v) = self.vertex_stack.pop() {
            self.vertex_backtrack.remove(&v);
            if let Some(edge) = self.edge_stack.pop() {
                assert_eq!(self.graph.edge(&edge).unwrap().sink, v);
                self.graph.remove_edge(&edge);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::digraph::r#impl::*;
    use quickcheck_macros::quickcheck;
    use std::{collections::HashSet, fmt::Write};

    #[test]
    fn self_loop() {
        let mut g = TreeBackedGraph::new();
        let v = g.add_vertex();
        g.add_edge(v, v);
        let trial: Vec<_> = g
            .simple_cycles()
            .map(|cycle| fmt_cycle(&g, cycle))
            .collect();
        let oracle = vec![format!("{v:?} -> {v:?}")];
        assert_eq!(trial, oracle);
    }

    #[test]
    fn back_and_forth_cycle() {
        let mut g = TreeBackedGraph::new();
        let v0 = g.add_vertex();
        let v1 = g.add_vertex();
        g.add_edge(v0, v1);
        g.add_edge(v1, v0);
        let trial: Vec<_> = g
            .simple_cycles_reachable_from(&v0)
            .map(|cycle| fmt_cycle(&g, cycle))
            .collect();
        let oracle = vec![format!("{v0:?} -> {v1:?} -> {v0:?}")];
        assert_eq!(trial, oracle);
    }

    #[quickcheck]
    fn simple_cycles_are_cyclic(ops: Ops) {
        let ops_formed: OpsFormedGraph<TreeBackedGraph> = (&ops).into();
        let graph = &ops_formed.graph;
        for cycle in graph.simple_cycles() {
            let cycle: Vec<_> = cycle.collect();
            if !is_cyclic(graph, cycle.clone().into_iter()) {
                println!("{}", fmt_cycle(graph, cycle.into_iter()));
                panic!()
            }
        }
    }

    fn is_cyclic<G, I>(graph: &G, cycle: I) -> bool
    where
        G: QueryableGraph,
        I: Iterator<Item = EdgeId>,
    {
        let edges: Vec<_> = cycle
            .map(|eid| {
                let e = graph.edge(&eid).unwrap();
                (e.source, e.sink)
            })
            .collect();
        for ((_, prev_snk), (nxt_src, _)) in
            edges.iter().zip(edges.iter().chain(edges.iter()).skip(1))
        {
            if prev_snk != nxt_src {
                return false;
            }
        }
        true
    }

    #[quickcheck]
    fn simple_cycles_are_simple(ops: Ops) {
        let ops_formed: OpsFormedGraph<TreeBackedGraph> = (&ops).into();
        let graph = &ops_formed.graph;
        for cycle in graph.simple_cycles() {
            let cycle: Vec<_> = cycle.collect();
            if !is_simple(graph, cycle.clone().into_iter()) {
                println!("{}", fmt_cycle(graph, cycle.into_iter()));
                panic!()
            }
        }
    }

    fn is_simple<G, I>(graph: &G, cycle: I) -> bool
    where
        G: QueryableGraph,
        I: Iterator<Item = EdgeId>,
    {
        let mut vs = HashSet::new();
        for snk in cycle.map(|eid| graph.edge(&eid).unwrap().sink) {
            if !vs.insert(snk) {
                return false;
            }
        }
        true
    }

    fn fmt_cycle<G, I>(g: &G, cycle: I) -> String
    where
        G: QueryableGraph,
        I: Iterator<Item = EdgeId>,
    {
        let mut cycle = cycle.peekable();
        let mut res = String::new();
        if let Some(eid) = cycle.peek() {
            write!(&mut res, "{:?}", g.edge(&eid).unwrap().source).unwrap();
            for e in cycle {
                let e = g.edge(&e).unwrap();
                write!(&mut res, " -> {:?}", e.sink).unwrap();
            }
        }
        res
    }
}
