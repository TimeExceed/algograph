//! Minimum-cost maximum flow on directed tagged graphs.
//!
//! This module solves the minimum-cost maximum flow (MCMF) problem
//! with the successive shortest path (SSP) method:
//! repeatedly find a shortest augmenting path (by cost) from the source
//! to the sink in the residual network, and push flow along it.
//! Shortest paths are computed by the SPFA algorithm, which tolerates
//! edges with negative costs (the backward residual edges always have
//! costs negated from the original ones).
//!
//! The residual network is *not* materialized.  Forward residual edges
//! are probed through [`QueryableTaggedGraph::out_edges`] and backward
//! ones through [`QueryableTaggedGraph::in_edges`], with a per-edge flow
//! table kept inside the algorithm, so the input graph is never modified.

use super::*;
use crate::graph::*;
use crate::tagged::QueryableTaggedGraph;
use ahash::{AHashMap, AHashSet};
use num_traits::bounds::UpperBounded;
use num_traits::identities::Zero;
use pinned_deque::PinnedDeque;
use std::cmp::Ord;
use std::ops::Add;

/// Computes the minimum-cost maximum flow of a directed tagged graph.
///
/// # Preconditions
///
/// *   The graph must be **directed**; the method panics otherwise.
/// *   The graph must contain no **negative-cost cycles**.
///     Individual edges may have negative costs, but any cycle in the
///     graph must have a non-negative total cost.
///     Violating this precondition panics with a negative-cycle message.
/// *   The answer (flow value and total cost) is assumed to fit into
///     `u64` and `i64` respectively.
///
/// # Algorithm
///
/// The successive shortest path method: each iteration finds a shortest
/// augmenting path from the source to the sink in the residual network
/// with SPFA, pushes as much flow as the bottleneck of the path allows,
/// and repeats until the sink is unreachable.  The input graph is never
/// modified; only a per-edge flow table is maintained inside the call.
///
/// # Complexity
///
/// | Item             | Complexity                 |
/// | ---------------- | -------------------------- |
/// | One SPFA run     | $O(VE)$ amortized          |
/// | Augmentations    | $O(F)$, $F$ the max flow   |
/// | Total            | $O(F \cdot VE)$ worst case |
///
/// # Examples
///
/// ```
/// use algograph::algorithm::mcmf::{FlowEdge, spfa::ShortestPath};
/// use algograph::graph::*;
/// use algograph::graph::directed::TreeBackedGraph;
/// use algograph::tagged::Edge as TaggedEdge;
/// use algograph::tagged::{GrowableTaggedGraph, NaiveTaggedGraph};
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct MyEdge {
///     source: VertexId,
///     sink: VertexId,
///     capacity: i64,
///     cost: i64,
/// }
///
/// impl TaggedEdge for MyEdge {
///     fn source(&self) -> VertexId {
///         self.source
///     }
///     fn sink(&self) -> VertexId {
///         self.sink
///     }
/// }
///
/// impl FlowEdge for MyEdge {
///     type Capacity = i64;
///     type Cost = i64;
///
///     fn capacity(&self) -> i64 {
///         self.capacity
///     }
///     fn cost(&self) -> i64 {
///         self.cost
///     }
/// }
///
/// let mut graph = NaiveTaggedGraph::<u32, MyEdge, TreeBackedGraph>::new();
/// let s = graph.overwrite_vertex(0);
/// let a = graph.overwrite_vertex(1);
/// let t = graph.overwrite_vertex(2);
/// graph.add_edge(MyEdge {
///     source: s,
///     sink: a,
///     capacity: 5,
///     cost: 1,
/// });
/// graph.add_edge(MyEdge {
///     source: a,
///     sink: t,
///     capacity: 3,
///     cost: 2,
/// });
///
/// let res = graph.min_cost_max_flow(&0u32, &2u32);
/// assert_eq!(res.max_flow, 3);
/// assert_eq!(res.min_cost, 9);
/// ```
pub trait ShortestPath
where
    Self: QueryableTaggedGraph + DirectedOrNot + Sized,
    Self::LowerGraph: QueryableGraph,
    Self::Edge: FlowEdge,
{
    /// Computes the minimum-cost maximum flow from `source` to `sink`.
    ///
    /// When `source` equals `sink`, a zero flow with zero cost is returned.
    fn min_cost_max_flow(
        &self,
        source: &Self::Vertex,
        sink: &Self::Vertex,
    ) -> McmfResult<<Self::Edge as FlowEdge>::Capacity, <Self::Edge as FlowEdge>::Cost> {
        assert!(Self::DIRECTED_OR_NOT, "A digraph is required to run MCMF.");
        let src = self
            .id_by_vertex(source)
            .expect("`source` must be in the graph.");
        let snk = self
            .id_by_vertex(sink)
            .expect("`sink` must be in the graph.");
        if src == snk {
            return McmfResult {
                max_flow: <Self::Edge as FlowEdge>::Capacity::zero(),
                min_cost: <Self::Edge as FlowEdge>::Cost::zero(),
                flows: AHashMap::default(),
            };
        }
        for (_, e) in self.iter_edges() {
            assert!(
                e.capacity() >= <Self::Edge as FlowEdge>::Capacity::zero(),
                "Negative capacity is not allowed."
            );
        }

        let mut solver = Solver::new(self);
        let mut max_flow = <Self::Edge as FlowEdge>::Capacity::zero();
        let mut min_cost = <Self::Edge as FlowEdge>::Cost::zero();
        while let Some(dist) = solver.spfa(src, snk) {
            let bottleneck = solver.augment(snk);
            max_flow += bottleneck;
            min_cost += dist * bottleneck;
        }
        solver.flow.retain(|_, f| !f.is_zero());
        McmfResult {
            max_flow,
            min_cost,
            flows: solver.flow,
        }
    }
}

impl<G> ShortestPath for G
where
    G: QueryableTaggedGraph + DirectedOrNot,
    G::LowerGraph: QueryableGraph,
    G::Edge: FlowEdge,
{
}

/// The internal state of the successive shortest path computation.
///
/// The residual network is probed on demand: forward residual edges come
/// from [`QueryableTaggedGraph::out_edges`] and backward ones from
/// [`QueryableTaggedGraph::in_edges`].  The only bookkeeping is the flow
/// pushed through each original edge.
struct Solver<'a, G>
where
    G: QueryableTaggedGraph,
    G::LowerGraph: QueryableGraph,
    G::Edge: FlowEdge,
{
    graph: &'a G,
    /// Flow currently pushed through each original edge.
    flow: AHashMap<EdgeId, <G::Edge as FlowEdge>::Capacity>,
    /// The previous vertex on the shortest path, indexed by the current vertex.
    /// This is filled by `spfa` and used by `augment`.
    prev: AHashMap<VertexId, (VertexId, EdgeId, Dir)>,

    /// Queue for BFS searching
    /// Only used by `spfa`. Put it here to reduce memory reallocations.
    queue: PinnedDeque<VertexId>,
    /// Tests if a vertex in the `queue`.
    /// Only used by `spfa`. Put it here to reduce memory reallocations.
    in_queue: AHashSet<VertexId>,
    // Shortest distance from the source
    /// Only used by `spfa`. Put it here to reduce memory reallocations.
    dist: AHashMap<VertexId, <G::Edge as FlowEdge>::Cost>,
    // counts how many times a vertex is dequeued, for negative-cycle detecting.
    /// Only used by `spfa`. Put it here to reduce memory reallocations.
    dequeue_count: AHashMap<VertexId, usize>,
}

impl<'a, G> Solver<'a, G>
where
    G: QueryableTaggedGraph,
    G::LowerGraph: QueryableGraph,
    G::Edge: FlowEdge,
{
    fn new(graph: &'a G) -> Self {
        let mut flow = AHashMap::with_capacity(graph.edge_size());
        for (e, _) in graph.iter_edges() {
            flow.insert(e.id, <G::Edge as FlowEdge>::Capacity::zero());
        }
        Self {
            graph,
            flow,
            prev: AHashMap::new(),
            queue: PinnedDeque::new(),
            in_queue: AHashSet::new(),
            dist: AHashMap::new(),
            dequeue_count: AHashMap::new(),
        }
    }

    /// Runs SPFA from `src` to `snk` in the residual network.
    ///
    /// Returns the shortest distance if `snk` is reachable, otherwise `None`.
    fn spfa(&mut self, src: VertexId, snk: VertexId) -> Option<<G::Edge as FlowEdge>::Cost> {
        self.queue.clear();
        self.in_queue.clear();
        self.dist.clear();
        self.dequeue_count.clear();

        self.dist.insert(src, <G::Edge as FlowEdge>::Cost::zero());
        self.queue.push_back(src);
        self.in_queue.insert(src);
        self.prev.clear();
        while let Some(x) = self.queue.pop_front() {
            self.in_queue.remove(&x);
            if let Some(cnt) = self.dequeue_count.get_mut(&x) {
                *cnt += 1;
                if *cnt > self.graph.vertex_size() {
                    panic!("There is a negative cycle in the residual network.");
                }
            } else {
                self.dequeue_count.insert(x, 1);
            }

            // Forward residual edges: the unused capacity of the original out-edges.
            for (e, upper) in self.graph.out_edges(&x) {
                let f = self.flow[&e.id];
                let y = e.sink;
                if upper.capacity() > f
                    && let Some(new_dist) = should_relax(&self.dist, x, y, upper.cost())
                {
                    self.dist.insert(y, new_dist);
                    self.prev.insert(y, (x, e.id, Dir::Forward));
                    if !self.in_queue.contains(&y) {
                        self.in_queue.insert(y);
                        self.queue.push_back(y);
                    }
                }
            }

            // Backward residual edges: the used capacity of the original in-edges.
            for (e, upper) in self.graph.in_edges(&x) {
                let f = self.flow[&e.id];
                let y = e.source;
                if !f.is_zero()
                    && let Some(new_dist) = should_relax(&self.dist, x, y, -upper.cost())
                {
                    self.dist.insert(y, new_dist);
                    self.prev.insert(y, (x, e.id, Dir::Backward));
                    if !self.in_queue.contains(&y) {
                        self.in_queue.insert(y);
                        self.queue.push_back(y);
                    }
                }
            }
        }
        self.dist.get(&snk).copied()
    }

    /// Pushes the bottleneck of the shortest path recorded by the last
    /// SPFA run, and returns that bottleneck.
    fn augment(&mut self, snk: VertexId) -> <G::Edge as FlowEdge>::Capacity {
        let mut bottleneck = <G::Edge as FlowEdge>::Capacity::max_value();

        let mut cur = snk;
        while let Some((prev_vert, eid, dir)) = self.prev.get(&cur) {
            let f = self.flow[eid];
            match dir {
                Dir::Forward => {
                    let cap = self.graph.edge_by_id(eid).unwrap().capacity();
                    bottleneck = bottleneck.min(cap - f);
                }
                Dir::Backward => bottleneck = bottleneck.min(f),
            }
            cur = *prev_vert;
        }

        let mut cur = snk;
        while let Some((prev_vert, eid, dir)) = self.prev.get(&cur) {
            let flow = self.flow.get_mut(eid).unwrap();
            match dir {
                Dir::Forward => *flow += bottleneck,
                Dir::Backward => *flow -= bottleneck,
            }
            cur = *prev_vert;
        }
        bottleneck
    }
}

/// Whether a residual edge goes along an original edge
/// or cancels flow on it through the backward direction.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Dir {
    Forward,
    Backward,
}

fn should_relax<C>(dist: &AHashMap<VertexId, C>, x: VertexId, y: VertexId, inc_dist: C) -> Option<C>
where
    C: Ord + Add<Output = C> + Copy + Clone,
{
    let new_dist = dist[&x] + inc_dist;
    let should_relax = dist.get(&y).is_none_or(|d| new_dist < *d);
    if should_relax { Some(new_dist) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::directed::TreeBackedGraph;
    use crate::tagged::{GrowableTaggedGraph, NaiveTaggedGraph, TaggedGraph};
    use crate::*;
    use quickcheck_macros::quickcheck;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct TestEdge {
        source: VertexId,
        sink: VertexId,
        capacity: i64,
        cost: i64,
    }

    impl tagged::Edge for TestEdge {
        fn source(&self) -> VertexId {
            self.source
        }

        fn sink(&self) -> VertexId {
            self.sink
        }
    }

    impl FlowEdge for TestEdge {
        type Capacity = i64;
        type Cost = i64;

        fn capacity(&self) -> i64 {
            self.capacity
        }

        fn cost(&self) -> i64 {
            self.cost
        }
    }

    fn build_graph(
        vertex_count: usize,
        edges: &[(usize, usize, i64, i64)],
    ) -> NaiveTaggedGraph<u32, TestEdge, TreeBackedGraph> {
        let mut graph = NaiveTaggedGraph::new();
        for i in 0..vertex_count {
            graph.overwrite_vertex(i as u32);
        }
        for &(u, v, capacity, cost) in edges {
            graph.add_edge(TestEdge {
                source: VertexId::new(u),
                sink: VertexId::new(v),
                capacity,
                cost,
            });
        }
        graph
    }

    fn edge_id<G>(graph: &G, u: usize, v: usize) -> EdgeId
    where
        G: QueryableTaggedGraph,
        G::LowerGraph: QueryableGraph,
    {
        graph
            .iter_edges()
            .find(|(e, _)| e.source.to_raw() == u && e.sink.to_raw() == v)
            .map(|(e, _)| e.id)
            .unwrap()
    }

    #[test]
    fn simple_chain() {
        let graph = build_graph(3, &[(0, 1, 5, 1), (1, 2, 3, 2)]);
        let res = graph.min_cost_max_flow(&0u32, &2u32);
        assert_eq!(res.max_flow, 3);
        assert_eq!(res.min_cost, 3 * (1 + 2));
    }

    #[test]
    fn two_parallel_paths() {
        let graph = build_graph(
            4,
            &[(0, 1, 1, 1), (1, 2, 1, 1), (0, 3, 5, 10), (3, 2, 5, 10)],
        );
        let res = graph.min_cost_max_flow(&0u32, &2u32);
        assert_eq!(res.max_flow, 6);
        // 1 unit through the cheap path, 5 units through the expensive one.
        assert_eq!(res.min_cost, (1 + 1) + 5 * (10 + 10));
    }

    #[test]
    fn negative_cost_edge() {
        let graph = build_graph(3, &[(0, 1, 2, -5), (1, 2, 2, 1), (0, 2, 1, 0)]);
        let res = graph.min_cost_max_flow(&0u32, &2u32);
        assert_eq!(res.max_flow, 3);
        assert_eq!(res.min_cost, 2 * (-5 + 1));
    }

    #[test]
    fn backward_augmentation() {
        // s=0, a=1, b=2, t=3.  The unique cheapest s-t path is s->a->b->t
        // (cost 0), so the first augmentation takes it; the second one must
        // cancel flow on a->b through its backward residual edge.
        let graph = build_graph(
            4,
            &[
                (0, 1, 1, 0),
                (1, 2, 1, 0),
                (2, 3, 1, 0),
                (0, 2, 1, 0),
                (2, 1, 1, 0),
                (1, 3, 1, 5),
            ],
        );
        let res = graph.min_cost_max_flow(&0u32, &3u32);
        assert_eq!(res.max_flow, 2);
        assert_eq!(res.min_cost, 5);
        let flow_ab = res.flows.get(&edge_id(&graph, 1, 2)).copied().unwrap_or(0);
        assert_eq!(flow_ab, 0);
    }

    #[test]
    fn source_equals_sink() {
        let graph = build_graph(2, &[(0, 1, 1, 5)]);
        let res = graph.min_cost_max_flow(&0u32, &0u32);
        assert_eq!(res.max_flow, 0);
        assert_eq!(res.min_cost, 0);
        assert!(res.flows.is_empty());
    }

    #[test]
    fn self_loop_and_parallel_edges() {
        let graph = build_graph(2, &[(0, 0, 1, 1), (0, 1, 1, 0), (0, 1, 2, 3)]);
        let res = graph.min_cost_max_flow(&0u32, &1u32);
        assert_eq!(res.max_flow, 3);
        // The self-loop is ignored; only the two parallel edges carry flow.
        assert_eq!(res.min_cost, 2 * 3);
    }

    #[test]
    #[should_panic(expected = "negative cycle")]
    fn negative_cycle_panics() {
        let graph = build_graph(
            4,
            &[(0, 1, 1, 0), (1, 2, 1, -5), (2, 1, 1, 2), (2, 3, 1, 0)],
        );
        let _ = graph.min_cost_max_flow(&0u32, &3u32);
    }

    /// A negative cycle that an enumerator of "non-tree edge + ancestor
    /// chain" fundamental cycles can miss: every cycle it reports is
    /// non-negative, yet the reachable cycle `2 -> 3 -> 4 -> 2` has total
    /// cost `0 + 0 + (-10) = -10 < 0` (its edges are absorbed as a tree edge
    /// and an ignored edge).  The in-run SPFA dequeue-count guard must still
    /// panic instead of looping forever.
    #[test]
    #[should_panic(expected = "negative cycle")]
    fn hidden_negative_cycle_panics() {
        let graph = build_graph(
            5,
            &[
                (0, 1, 1, 0),
                (1, 2, 1, 0),
                (2, 3, 1, 0),
                (3, 4, 1, 0),
                (2, 4, 1, 10),
                (4, 2, 1, -10),
                (4, 0, 1, 0),
            ],
        );
        let _ = graph.min_cost_max_flow(&0u32, &4u32);
    }

    /// A random directed tagged graph with non-negative costs,
    /// so that no negative cycle can appear.
    #[derive(Clone)]
    struct McmfTestGraph {
        graph: NaiveTaggedGraph<u32, TestEdge, TreeBackedGraph>,
    }

    impl std::fmt::Debug for McmfTestGraph {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let vertices: Vec<u32> = self.graph.iter_vertices().map(|(_, v)| *v).collect();
            let edges: Vec<_> = self
                .graph
                .iter_edges()
                .map(|(e, upper)| {
                    (
                        e.source.to_raw(),
                        e.sink.to_raw(),
                        upper.capacity(),
                        upper.cost(),
                    )
                })
                .collect();
            f.debug_struct("McmfTestGraph")
                .field("vertices", &vertices)
                .field("edges", &edges)
                .finish()
        }
    }

    impl quickcheck::Arbitrary for McmfTestGraph {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let vertex_count = 1 + usize::arbitrary(g) % 9;
            let mut graph = NaiveTaggedGraph::new();
            for i in 0..vertex_count {
                graph.overwrite_vertex(i as u32);
            }
            let edge_count = usize::arbitrary(g) % 20;
            let mut seen = std::collections::HashSet::new();
            for _ in 0..edge_count {
                let source = VertexId::new(usize::arbitrary(g) % vertex_count);
                let sink = VertexId::new(usize::arbitrary(g) % vertex_count);
                let capacity: i64 = (u8::arbitrary(g) % 6).into();
                let cost = i64::arbitrary(g).rem_euclid(11);
                // `NaiveTaggedGraph` keys edges by their values, so identical
                // edges must not be added twice.
                if seen.insert((source.to_raw(), sink.to_raw(), capacity, cost)) {
                    graph.add_edge(TestEdge {
                        source,
                        sink,
                        capacity,
                        cost,
                    });
                }
            }
            Self { graph }
        }
    }

    /// The value of a maximum s-t flow, computed by Edmonds-Karp
    /// as an independent oracle.
    fn max_flow_oracle<G>(
        graph: &G,
        src: VertexId,
        snk: VertexId,
    ) -> <G::Edge as FlowEdge>::Capacity
    where
        G: QueryableTaggedGraph,
        G::LowerGraph: QueryableGraph,
        G::Edge: FlowEdge,
    {
        let n = graph.vertex_size();
        let mut capacity = vec![vec![<G::Edge as FlowEdge>::Capacity::zero(); n]; n];
        for (e, upper) in graph.iter_edges() {
            capacity[e.source.to_raw()][e.sink.to_raw()] += upper.capacity();
        }
        let mut total = <G::Edge as FlowEdge>::Capacity::zero();
        loop {
            let mut parent = vec![None; n];
            let mut queue = PinnedDeque::new();
            queue.push_back(src.to_raw());
            parent[src.to_raw()] = Some(src.to_raw());
            while let Some(u) = queue.pop_front() {
                for v in 0..n {
                    if parent[v].is_none() && !capacity[u][v].is_zero() {
                        parent[v] = Some(u);
                        queue.push_back(v);
                    }
                }
            }
            if parent[snk.to_raw()].is_none() {
                break;
            }
            let mut bottleneck = <G::Edge as FlowEdge>::Capacity::max_value();
            let mut v = snk.to_raw();
            while v != src.to_raw() {
                let u = parent[v].unwrap();
                bottleneck = bottleneck.min(capacity[u][v]);
                v = u;
            }
            let mut v = snk.to_raw();
            while v != src.to_raw() {
                let u = parent[v].unwrap();
                capacity[u][v] -= bottleneck;
                capacity[v][u] += bottleneck;
                v = u;
            }
            total += bottleneck;
        }
        total
    }

    /// Whether the residual network of a flow contains a negative cycle.
    ///
    /// Bellman-Ford with all distances initialized to zero detects any
    /// negative cycle in the whole graph.
    fn has_negative_cycle<G>(
        graph: &G,
        flow: &AHashMap<EdgeId, <G::Edge as FlowEdge>::Capacity>,
    ) -> bool
    where
        G: QueryableTaggedGraph,
        G::LowerGraph: QueryableGraph,
        G::Edge: FlowEdge,
    {
        let n = graph.vertex_size();
        let mut edges = Vec::new();
        for (e, upper) in graph.iter_edges() {
            let f = flow
                .get(&e.id)
                .copied()
                .unwrap_or(<G::Edge as FlowEdge>::Capacity::zero());
            if upper.capacity() > f {
                edges.push((e.source.to_raw(), e.sink.to_raw(), upper.cost()));
            }
            if !f.is_zero() {
                edges.push((e.sink.to_raw(), e.source.to_raw(), -upper.cost()));
            }
        }
        let mut dist = vec![<G::Edge as FlowEdge>::Cost::zero(); n];
        for _ in 0..n {
            let mut updated = false;
            for &(u, v, w) in &edges {
                if dist[u] + w < dist[v] {
                    dist[v] = dist[u] + w;
                    updated = true;
                }
            }
            if !updated {
                return false;
            }
        }
        true
    }

    #[quickcheck]
    fn min_cost_max_flow_properties(graph: McmfTestGraph) {
        let vids: Vec<VertexId> = graph.graph.iter_vertices().map(|(v, _)| v).collect();
        if vids.len() < 2 {
            return;
        }
        let src = vids[0];
        let snk = vids[1];
        let res = graph.graph.min_cost_max_flow(
            graph.graph.vertex_by_id(&src).unwrap(),
            graph.graph.vertex_by_id(&snk).unwrap(),
        );
        let flow = &res.flows;

        // The flow must respect capacities.
        for (e, upper) in graph.graph.iter_edges() {
            let f = flow.get(&e.id).copied().unwrap_or(0);
            assert!(
                f <= upper.capacity(),
                "flow {f} exceeds capacity {}",
                upper.capacity()
            );
        }

        // The flow must be conserved at every vertex except the source and the sink.
        for (v, _) in graph.graph.iter_vertices() {
            if v == src || v == snk {
                continue;
            }
            let out: i64 = graph
                .graph
                .out_edges(&v)
                .map(|(e, _)| flow.get(&e.id).copied().unwrap_or(0))
                .sum();
            let inn: i64 = graph
                .graph
                .in_edges(&v)
                .map(|(e, _)| flow.get(&e.id).copied().unwrap_or(0))
                .sum();
            assert_eq!(out, inn, "flow not conserved at {:?}", v);
        }

        // The reported cost must match the flow.
        let cost: i64 = graph
            .graph
            .iter_edges()
            .map(|(e, upper)| flow.get(&e.id).copied().unwrap_or(0) * upper.cost())
            .sum();
        assert_eq!(res.min_cost, cost);

        // The flow must be a maximum flow.
        assert_eq!(res.max_flow, max_flow_oracle(&graph.graph, src, snk));

        // The flow must be minimum-cost: no negative cycle in the residual network.
        assert!(!has_negative_cycle(&graph.graph, flow));
    }
}
