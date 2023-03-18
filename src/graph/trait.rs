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
    init_indent: usize,
    indent_step: usize,
}

impl<'a, G> GraphDebug<'a, G>
where
    G: QueryableGraph,
{
    fn new(graph: &'a G) -> Self {
        Self {
            graph,
            init_indent: 0,
            indent_step: 2,
        }
    }

    pub fn indent(mut self, init: usize, step: usize) -> Self {
        self.init_indent = init;
        self.indent_step = step;
        self
    }

    fn display_indent(&self, f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result {
        let indention = self.init_indent + self.indent_step * level;
        for _ in 0..indention {
            write!(f, " ")?;
        }
        Ok(())
    }
}

impl<'a, G> std::fmt::Debug for GraphDebug<'a, G>
where
    G: QueryableGraph,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for v in self.graph.iter_vertices() {
            self.display_indent(f, 0)?;
            writeln!(f, "{:?}", v)?;
            for e in self.graph.out_edges(&v) {
                self.display_indent(f, 1)?;
                writeln!(f, "--{:?}-> {:?}", e.id, e.sink)?;
            }
        }
        Ok(())
    }
}

pub trait DirectedOrNot {
    const DIRECTED_OR_NOT: bool;
}
