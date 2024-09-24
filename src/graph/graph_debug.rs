use crate::graph::*;

/// A default implementation of inspecting into a graph with customized indentation.
pub struct GraphDebug<'a, G>
where
    G: QueryableGraph,
{
    graph: &'a G,
    indent: crate::util::Indention,
}

impl<'a, G> GraphDebug<'a, G>
where
    G: QueryableGraph,
{
    pub fn new(graph: &'a G, init_indent: usize, indent_step: usize) -> Self {
        Self {
            graph,
            indent: crate::util::Indention {
                spaces: init_indent,
                step: indent_step,
            },
        }
    }
}

impl<'a, G> std::fmt::Debug for GraphDebug<'a, G>
where
    G: QueryableGraph,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = &self.indent;
        let nxt_indent = self.indent.one_more_level();
        for v in self.graph.iter_vertices() {
            writeln!(f, "{indent}{:?}", v)?;
            for e in self.graph.out_edges(&v) {
                writeln!(f, "{nxt_indent}--{:?}-> {:?}", e.id, e.sink)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::*;

    #[test]
    fn undirected_graph_debug() {
        use std::fmt::Write;

        let mut g = undirected::TreeBackedGraph::new();
        let vertices: Vec<_> = (0..3).map(|_| g.add_vertex()).collect();
        let mut vert_iter0 = vertices.iter();
        while let Some(v0) = vert_iter0.next() {
            for v1 in vert_iter0.clone() {
                g.add_edge(*v0, *v1);
            }
        }
        println!("{:?}", g.debug());
        let mut out = String::new();
        write!(&mut out, "{:?}", g.debug()).unwrap();
        assert_eq!(
            out,
            "VertexId(0)\
            \n  --EdgeId(0)-> VertexId(1)\
            \n  --EdgeId(1)-> VertexId(2)\
            \nVertexId(1)\
            \n  --EdgeId(0)-> VertexId(0)\
            \n  --EdgeId(2)-> VertexId(2)\
            \nVertexId(2)\
            \n  --EdgeId(1)-> VertexId(0)\
            \n  --EdgeId(2)-> VertexId(1)\
            \n"
        );
    }
}
