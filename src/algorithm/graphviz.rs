//! Visualize tagged graphs in the graphviz format.
use crate::{graph::*, tagged::Edge as _Edge};
use ahash::RandomState;
use std::collections::HashMap;

/**
 * Provides graphviz labels for vertices.
 *
 * See [DumpInGraphviz] for details.
 */
pub trait GraphvizLabelForVertex {
    /**
     * Returns a string for graphviz name and an optional label.
     */
    fn label(&self) -> (String, Option<String>);
}

/**
 * Provides graphviz labels for edges.
 *
 * See [DumpInGraphviz] for details.
 */
pub trait GraphvizLabelForEdge: crate::tagged::Edge {
    /**
     * Returns an optional label.
     */
    fn label(&self) -> Option<String>;
}

/**
 * Dumps a directed/undirected graph into graphviz format.
 *
 * # Examples
 *
 * ```rust
 * use algograph::{
 *     algorithm::graphviz::*,
 *     graph::*,
 *     tagged::*,
 * };
 *
 * #[derive(Debug, Clone, Hash, PartialEq, Eq)]
 * enum Shape {
 *     Default,
 *     Rectangle,
 * }
 *
 * #[derive(Debug, Clone, Hash, PartialEq, Eq)]
 * struct ShapedVertex {
 *     key: usize,
 *     shape: Shape,
 * }
 *
 * impl GraphvizLabelForVertex for ShapedVertex {
 *     fn label(&self) -> (String, Option<String>) {
 *         let name = format!("{}", self.key);
 *         let label = match self.shape {
 *             Shape::Rectangle => Some("shape=rectangle".to_owned()),
 *             _ => None,
 *         };
 *         (name, label)
 *     }
 * }
 *
 * #[derive(Debug, Clone, Hash, PartialEq, Eq)]
 * enum Color {
 *     Default,
 *     Red,
 * }
 *
 * #[derive(Debug, Clone, Hash, PartialEq, Eq)]
 * struct ColoredEdge {
 *     src: VertexId,
 *     snk: VertexId,
 *     color: Color,
 * }
 *
 * impl algograph::tagged::Edge for ColoredEdge {
 *     fn source(&self) -> VertexId {
 *         self.src
 *     }
 *     fn sink(&self) -> VertexId {
 *         self.snk
 *     }
 * }
 *
 * impl GraphvizLabelForEdge for ColoredEdge {
 *     fn label(&self) -> Option<String> {
 *         match self.color {
 *             Color::Red => Some("color=red".to_owned()),
 *             _ => None,
 *         }
 *     }
 * }
 *
 * // for a directed graph
 * let mut dg = NaiveTaggedGraph::<ShapedVertex, ColoredEdge>::new();
 * let v0 = dg.overwrite_vertex(ShapedVertex {
 *     key: 0,
 *     shape: Shape::Default,
 * });
 * let v1 = dg.overwrite_vertex(ShapedVertex {
 *     key: 1,
 *     shape: Shape::Rectangle,
 * });
 * let _ = dg.add_edge(ColoredEdge {
 *     src: v0,
 *     snk: v1,
 *     color: Color::Red,
 * });
 * let _ = dg.add_edge(ColoredEdge {
 *     src: v0,
 *     snk: v0,
 *     color: Color::Default,
 * });
 * let trial = {
 *     let mut buf = vec![];
 *     dg.dump_in_graphviz(&mut buf, "trial").unwrap();
 *     String::from_utf8(buf).unwrap()
 * };
 * assert_eq!(
 *      trial,
 *      r#"digraph trial {
 *   0 ;
 *   1 [shape=rectangle] ;
 *   0 -> 1 [color=red] ;
 *   0 -> 0 ;
 * }
 * "#);
 *
 * // for an undirected graph
 * let mut udg =
 *     NaiveTaggedGraph::<ShapedVertex, ColoredEdge, undirected::TreeBackedGraph>::new();
 * let v0 = udg.overwrite_vertex(ShapedVertex {
 *     key: 0,
 *     shape: Shape::Default,
 * });
 * let v1 = udg.overwrite_vertex(ShapedVertex {
 *     key: 1,
 *     shape: Shape::Rectangle,
 * });
 * let _ = udg.add_edge(ColoredEdge {
 *     src: v0,
 *     snk: v1,
 *     color: Color::Red,
 * });
 * let _ = udg.add_edge(ColoredEdge {
 *     src: v0,
 *     snk: v0,
 *     color: Color::Default,
 * });
 * let trial = {
 *     let mut buf = vec![];
 *     udg.dump_in_graphviz(&mut buf, "trial").unwrap();
 *     String::from_utf8(buf).unwrap()
 * };
 * println!("{}", trial);
 * assert_eq!(
 *     trial,
 *     r#"graph trial {
 *   0 ;
 *   1 [shape=rectangle] ;
 *   0 -- 1 [color=red] ;
 *   0 -- 0 ;
 * }
 * "#
 * );
 * ```
 */
pub trait DumpInGraphviz
where
    Self: crate::tagged::QueryableTaggedGraph + DirectedOrNot,
    Self::LowerGraph: QueryableGraph,
    Self::Vertex: GraphvizLabelForVertex,
    Self::Edge: GraphvizLabelForEdge,
{
    /**
     * Dumps a directed/undirected graph to a `std::io::Write` object in the graphviz format.
     *
     */
    fn dump_in_graphviz<W>(&self, out: &mut W, graph_name: &str) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        if Self::DIRECTED_OR_NOT {
            writeln!(out, "digraph {} {{", graph_name)?;
        } else {
            writeln!(out, "graph {} {{", graph_name)?;
        }
        let mut vkey = HashMap::with_hasher(RandomState::new());
        for (vid, vert) in self.iter_vertices() {
            let (key, label) = vert.label();
            if let Some(label) = label {
                writeln!(out, "  {} [{}] ;", key, label)?;
            } else {
                writeln!(out, "  {} ;", key)?;
            }
            vkey.insert(vid, key);
        }
        let dir = if Self::DIRECTED_OR_NOT { "->" } else { "--" };
        for (_, e) in self.iter_edges() {
            let src = vkey.get(&e.source()).unwrap();
            let snk = vkey.get(&e.sink()).unwrap();
            if let Some(label) = e.label() {
                writeln!(out, "  {} {} {} [{}] ;", src, dir, snk, label)?;
            } else {
                writeln!(out, "  {} {} {} ;", src, dir, snk)?;
            }
        }
        writeln!(out, "}}")?;
        Ok(())
    }
}

impl<G> DumpInGraphviz for G
where
    G: crate::tagged::QueryableTaggedGraph + DirectedOrNot,
    G::LowerGraph: QueryableGraph,
    G::Vertex: GraphvizLabelForVertex,
    G::Edge: GraphvizLabelForEdge,
{
}
