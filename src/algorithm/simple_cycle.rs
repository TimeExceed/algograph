use crate::graph::*;
use ahash::RandomState;
use std::collections::{HashMap, HashSet};

pub trait SimpleCycles
where
    Self: QueryableGraph + Sized,
{
    fn simple_cycles(&self) -> Box<dyn Iterator<Item = Box<dyn Iterator<Item = Edge> + '_>> + '_> {
        Box::new(CycleIterator::exhaust(self))
    }

    fn simple_cycles_reachable_from(
        &self,
        vert: &VertexId,
    ) -> Box<dyn Iterator<Item = Box<dyn Iterator<Item = Edge> + '_>> + '_> {
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
    come_from: HashMap<VertexId, Option<Edge>, RandomState>,
    stack: Vec<StackItem>,
    exhausted_vertices: HashSet<VertexId, RandomState>,
}

impl<'a, G> Iterator for CycleIterator<'a, G>
where
    G: QueryableGraph,
{
    type Item = Box<dyn Iterator<Item = Edge> + 'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(top_item) = self.stack.pop() {
                match top_item {
                    StackItem::Vertex(vert) => {
                        self.exhausted_vertices.insert(vert);
                    }
                    StackItem::Edge(edge) => {
                        if !self.graph.contains_edge(&edge.id) {
                            // intend to do nothing
                        } else {
                            self.graph.remove_edge(&edge.id);
                            if self.exhausted_vertices.contains(&edge.sink) {
                                // intend to do nothing
                            } else if let Some(_) = self.come_from.get(&edge.sink) {
                                let terminal = edge.sink.clone();
                                let mut res = vec![edge.clone()];
                                let mut edge = edge;
                                loop {
                                    if edge.source == terminal {
                                        break;
                                    }
                                    let e = self.come_from.get(&edge.source).unwrap();
                                    let e = e.clone().unwrap();
                                    res.push(e.clone());
                                    edge = e;
                                }
                                res.reverse();
                                return Some(Box::new(res.into_iter()));
                            } else {
                                self.come_from.insert(edge.sink, Some(edge.clone()));
                                self.extend_stack(edge.sink);
                            }
                        }
                    }
                }
            } else if let Some(vert) = self.to_exhaust_vertices.pop() {
                if !self.come_from.contains_key(&vert) {
                    self.come_from.insert(vert, None);
                    self.extend_stack(vert);
                }
            } else {
                return None;
            }
        }
    }
}

impl<'a, G> CycleIterator<'a, G>
where
    G: QueryableGraph,
{
    fn new(graph: &'a G) -> Self {
        Self {
            graph: ShadowedSubgraph::new(graph),
            to_exhaust_vertices: vec![],
            come_from: HashMap::with_hasher(RandomState::new()),
            stack: vec![],
            exhausted_vertices: HashSet::with_hasher(RandomState::new()),
        }
    }

    fn exhaust(graph: &'a G) -> Self {
        let mut res = Self::new(graph);
        for v in graph.iter_vertices() {
            res.to_exhaust_vertices.push(v);
        }
        res
    }

    fn start_from(graph: &'a G, vert: &VertexId) -> Self {
        let mut res = Self::new(graph);
        res.to_exhaust_vertices.push(*vert);
        res
    }

    fn extend_stack(&mut self, vert: VertexId) {
        self.stack.push(StackItem::Vertex(vert));
        for nxt_edge in self.graph.out_edges(&vert) {
            self.stack.push(StackItem::Edge(nxt_edge));
        }
    }
}

enum StackItem {
    Vertex(VertexId),
    Edge(Edge),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashSet, fmt::Write};

    mod directed {
        use super::super::SimpleCycles;
        use crate::graph::{
            directed::{Ops, TreeBackedGraph},
            *,
        };
        use quickcheck_macros::quickcheck;

        #[test]
        fn self_loop() {
            let mut g = TreeBackedGraph::new();
            let v = g.add_vertex();
            g.add_edge(v, v);
            let trial: Vec<_> = g
                .simple_cycles()
                .map(|cycle| super::fmt_cycle(cycle))
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
                .map(|cycle| super::fmt_cycle(cycle))
                .collect();
            let oracle = vec![format!("{v0:?} -> {v1:?} -> {v0:?}")];
            assert_eq!(trial, oracle);
        }

        #[quickcheck]
        fn simple_cycles_are_cyclic(ops: Ops) {
            let ops_formed: MappedGraph<TreeBackedGraph> = (&ops).into();
            let graph = &ops_formed.graph;
            for cycle in graph.simple_cycles() {
                let cycle: Vec<_> = cycle.collect();
                if !super::is_cyclic(cycle.clone().into_iter()) {
                    println!("{}", super::fmt_cycle(cycle.into_iter()));
                    panic!()
                }
            }
        }

        #[quickcheck]
        fn simple_cycles_are_simple(ops: Ops) {
            let ops_formed: MappedGraph<TreeBackedGraph> = (&ops).into();
            let graph = &ops_formed.graph;
            for cycle in graph.simple_cycles() {
                let cycle: Vec<_> = cycle.collect();
                if !super::is_simple(cycle.clone().into_iter()) {
                    println!("{}", super::fmt_cycle(cycle.into_iter()));
                    panic!()
                }
            }
        }
    }

    mod undirected {
        use super::super::SimpleCycles;
        use crate::graph::{directed::Ops, undirected::TreeBackedGraph, *};
        use quickcheck_macros::quickcheck;

        #[test]
        fn self_loop() {
            let mut g = TreeBackedGraph::new();
            let v = g.add_vertex();
            g.add_edge(v, v);
            let trial: Vec<_> = g
                .simple_cycles()
                .map(|cycle| super::fmt_cycle(cycle))
                .collect();
            let oracle = vec![format!("{v:?} -> {v:?}")];
            assert_eq!(trial, oracle);
        }

        #[test]
        fn no_cycle() {
            let mut g = TreeBackedGraph::new();
            let v0 = g.add_vertex();
            let v1 = g.add_vertex();
            g.add_edge(v0, v1);
            let trial: Vec<_> = g
                .simple_cycles_reachable_from(&v0)
                .map(|cycle| super::fmt_cycle(cycle))
                .collect();
            let oracle: Vec<String> = vec![];
            assert_eq!(trial, oracle);
        }

        #[test]
        fn back_and_forth_cycle() {
            let mut g = TreeBackedGraph::new();
            let v0 = g.add_vertex();
            let v1 = g.add_vertex();
            g.add_edge(v0, v1);
            g.add_edge(v0, v1);
            let trial: Vec<_> = g
                .simple_cycles_reachable_from(&v0)
                .map(|cycle| super::fmt_cycle(cycle))
                .collect();
            let oracle = vec![format!("{v0:?} -> {v1:?} -> {v0:?}")];
            assert_eq!(trial, oracle);
        }

        #[quickcheck]
        fn simple_cycles_are_cyclic(ops: Ops) {
            let ops_formed: MappedGraph<TreeBackedGraph> = (&ops).into();
            let graph = &ops_formed.graph;
            for cycle in graph.simple_cycles() {
                let cycle: Vec<_> = cycle.collect();
                if !super::is_cyclic(cycle.clone().into_iter()) {
                    println!("{}", super::fmt_cycle(cycle.into_iter()));
                    panic!()
                }
            }
        }

        #[quickcheck]
        fn simple_cycles_are_simple(ops: Ops) {
            let ops_formed: MappedGraph<TreeBackedGraph> = (&ops).into();
            let graph = &ops_formed.graph;
            for cycle in graph.simple_cycles() {
                let cycle: Vec<_> = cycle.collect();
                if !super::is_simple(cycle.clone().into_iter()) {
                    println!("{}", super::fmt_cycle(cycle.into_iter()));
                    panic!()
                }
            }
        }
    }

    fn is_cyclic<I>(cycle: I) -> bool
    where
        I: Iterator<Item = Edge>,
    {
        let edges: Vec<_> = cycle.collect();
        for (prev, next) in edges.iter().zip(edges.iter().chain(edges.iter()).skip(1)) {
            if prev.sink != next.source {
                return false;
            }
        }
        true
    }

    fn is_simple<I>(cycle: I) -> bool
    where
        I: Iterator<Item = Edge>,
    {
        let mut vs = HashSet::new();
        for e in cycle {
            if !vs.insert(e.sink) {
                return false;
            }
        }
        true
    }

    fn fmt_cycle<I>(cycle: I) -> String
    where
        I: Iterator<Item = Edge>,
    {
        let mut cycle = cycle.peekable();
        let mut res = String::new();
        if let Some(e) = cycle.peek() {
            write!(&mut res, "{:?}", e.source).unwrap();
            for e in cycle {
                write!(&mut res, " -> {:?}", e.sink).unwrap();
            }
        }
        res
    }
}
