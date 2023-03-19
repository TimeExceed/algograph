use crate::graph::*;

/// A default implementation of inspecting into a graph with customized indentation.
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
    pub fn new(graph: &'a G) -> Self {
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
