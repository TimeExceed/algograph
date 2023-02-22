use crate::graph::*;

pub trait GrowableGraph {
    fn new() -> Self;
    fn add_vertex(&mut self) -> VertexId;
    fn add_edge(&mut self, source: VertexId, sink: VertexId) -> EdgeId;
}

pub trait EdgeShrinkableGraph {
    fn remove_edge(&mut self, edge: &EdgeId) -> Option<Edge>;
}

pub trait VertexShrinkableGraph: EdgeShrinkableGraph {
    fn remove_vertex(&mut self, vertex: &VertexId) -> Box<dyn Iterator<Item = Edge> + 'static>;
}

pub trait QueryableGraph {
    fn vertex_size(&self) -> usize;
    fn iter_vertices(&self) -> Box<dyn Iterator<Item = VertexId> + '_>;
    fn contains_vertex(&self, v: &VertexId) -> bool;

    fn edge_size(&self) -> usize;
    fn iter_edges(&self) -> Box<dyn Iterator<Item = Edge> + '_>;
    fn contains_edge(&self, e: &EdgeId) -> bool;
    fn find_edge(&self, e: &EdgeId) -> Option<Edge>;
    fn edges_connecting(
        &self,
        source: &VertexId,
        sink: &VertexId,
    ) -> Box<dyn Iterator<Item = Edge> + '_>;
    fn in_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_>;
    fn out_edges(&self, v: &VertexId) -> Box<dyn Iterator<Item = Edge> + '_>;

    fn debug<'a>(&'a self) -> GraphDebug<'a, Self>
    where
        Self: Sized,
    {
        GraphDebug::new(self)
    }
}

pub struct GraphDebug<'a, G>
where
    G: QueryableGraph,
{
    graph: &'a G,
    preamble: Option<&'a str>,
    epilogue: Option<&'a str>,
}

impl<'a, G> GraphDebug<'a, G>
where
    G: QueryableGraph,
{
    fn new(graph: &'a G) -> Self {
        Self {
            graph,
            preamble: None,
            epilogue: None,
        }
    }

    pub fn preamble(mut self, pre: &'a str) -> Self {
        self.preamble.replace(pre);
        self
    }

    pub fn epilogue(mut self, epi: &'a str) -> Self {
        self.epilogue.replace(epi);
        self
    }
}

impl<'a, G> std::fmt::Debug for GraphDebug<'a, G>
where
    G: QueryableGraph,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(pre) = self.preamble {
            writeln!(f, "{}", pre)?;
        }
        for v in self.graph.iter_vertices() {
            writeln!(f, "{:?}", v)?;
            for e in self.graph.out_edges(&v) {
                writeln!(f, "  --{:?}-> {:?}", e.id, e.sink)?;
            }
        }
        if let Some(epi) = self.epilogue {
            writeln!(f, "{}", epi)?;
        }
        Ok(())
    }
}

pub trait DirectedOrNot {
    const DIRECTED_OR_NOT: bool;
}
