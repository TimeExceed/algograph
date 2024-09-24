use crate::graph::*;

/// Links between low-level graphs and tagged graphs,
/// such as translation, in both directions,
/// between [VertexId] and customized vertices,
/// and between [EdgeId] and customized edges.
pub trait TaggedGraph {
    /// type of underlying low-level graph
    type LowerGraph;
    /// customized vertex type
    type Vertex;
    /// customized edge type
    type Edge: Edge;

    fn lower_graph(&self) -> &Self::LowerGraph;
    fn vertex_by_id(&self, vid: &VertexId) -> Option<&Self::Vertex>;
    fn id_by_vertex(&self, vert: &Self::Vertex) -> Option<VertexId>;
    fn edge_by_id(&self, eid: &EdgeId) -> Option<&Self::Edge>;
    fn id_by_edge(&self, edge: &Self::Edge) -> Option<EdgeId>;

    fn contains_vertex_by_id(&self, vid: &VertexId) -> bool {
        self.vertex_by_id(vid).is_some()
    }

    fn contains_vertex(&self, vert: &Self::Vertex) -> bool {
        self.id_by_vertex(vert).is_some()
    }

    fn contains_edge_by_id(&self, eid: &EdgeId) -> bool {
        self.edge_by_id(eid).is_some()
    }

    fn contains_edge(&self, edge: &Self::Edge) -> bool {
        self.id_by_edge(edge).is_some()
    }
}

/// Interfaces to query vertices and edges in tagged graphs.
pub trait QueryableTaggedGraph: TaggedGraph
where
    Self::LowerGraph: QueryableGraph,
{
    /// Total number of vertices.
    fn vertex_size(&self) -> usize;
    /// Iterates over vertices without any specific order.
    fn iter_vertices(&self) -> Box<dyn Iterator<Item = (VertexId, &Self::Vertex)> + '_>;

    /// Total number of edges.
    fn edge_size(&self) -> usize;
    /// Iterates edges without any specific order.
    fn iter_edges(&self) -> Box<dyn Iterator<Item = (crate::graph::Edge, &Self::Edge)> + '_>;
    /// Iterates edges connecting two specified endpoints.
    fn edges_connecting(
        &self,
        source: &VertexId,
        sink: &VertexId,
    ) -> Box<dyn Iterator<Item = (crate::graph::Edge, &Self::Edge)> + '_>;
    /// Iterates over in-edges of a specified vertex.
    fn in_edges(
        &self,
        vid: &VertexId,
    ) -> Box<dyn Iterator<Item = (crate::graph::Edge, &Self::Edge)> + '_>;
    /// Iterates over out-edges of a specified vertex.
    fn out_edges(
        &self,
        vid: &VertexId,
    ) -> Box<dyn Iterator<Item = (crate::graph::Edge, &Self::Edge)> + '_>;
}

/// Interfaces to add customized vertices and edges into tagged graphs.
pub trait GrowableTaggedGraph: TaggedGraph
where
    Self::LowerGraph: GrowableGraph,
{
    /// Creates an empty graph.
    fn new() -> Self;
    /// Inserts a vertex if it is unpresent or
    /// updates the one with a same [VertexId].
    fn overwrite_vertex(&mut self, vert: Self::Vertex) -> VertexId;
    /// Adds an edge and returns its [EdgeId].
    fn add_edge(&mut self, edge: Self::Edge) -> EdgeId;
    /// Updates an edge w.r.t. its [EdgeId].
    fn update_edge(&mut self, eid: EdgeId, edge: Self::Edge);
}

/// Interfaces to remove edges from tagged graphs.
pub trait EdgeShrinkableTaggedGraph: TaggedGraph
where
    Self::LowerGraph: EdgeShrinkableGraph,
{
    /// Removes an edge and returns it if it is present.
    ///
    /// Removing an edge will not remove its endpoints.
    fn remove_edge(&mut self, eid: &EdgeId) -> Option<Self::Edge>;
}

/// Interfaces to remove vertices from the graph.
pub trait VertexShrinkableTaggedGraph: TaggedGraph
where
    Self::LowerGraph: VertexShrinkableGraph,
{
    /// Removes a vertex and edges connecting to it and returns these edges.
    fn remove_vertex(
        &mut self,
        vid: &VertexId,
    ) -> Box<dyn Iterator<Item = (EdgeId, Self::Edge)> + '_>;
}

/// A trait for customized edges.
///
/// In order to implement algorithms without inspecting on details of customized
/// edges, the laters must provide their low-level information such as endpoints.
pub trait Edge {
    /// The source of an edge.
    ///
    /// *   For directed graphs, this must be the source.
    /// *   For undirected graphs, either is okay.
    ///     Just be the opposite of the sink.
    fn source(&self) -> VertexId;
    /// The sink of an edge.
    ///
    /// *   For directed graphs, this mut be the sink.
    /// *   For undirected graphs, either is okay.
    ///     Just be the opposite of the source.
    fn sink(&self) -> VertexId;
}

/// A `debug` function which allows users to inspect details of tagged graphs.
pub trait DebuggableTaggedGraph
where
    Self: QueryableTaggedGraph + DirectedOrNot + Sized,
    Self::LowerGraph: QueryableGraph + DirectedOrNot,
    Self::Vertex: std::fmt::Debug,
    Self::Edge: std::fmt::Debug,
{
    /// Inspects into a tagged graph.
    fn debug(&self) -> GraphDebug<'_, Self> {
        self.debug_with_indent(0, 2)
    }

    /// Inspects into a tagged graph with customized indentation.
    fn debug_with_indent(&self, init: usize, indent: usize) -> GraphDebug<'_, Self> {
        GraphDebug {
            graph: self,
            indent: crate::util::Indention {
                spaces: init,
                step: indent,
            },
        }
    }
}

impl<G> DebuggableTaggedGraph for G
where
    G: QueryableTaggedGraph + DirectedOrNot + Sized,
    G::LowerGraph: QueryableGraph + DirectedOrNot,
    G::Vertex: std::fmt::Debug,
    G::Edge: std::fmt::Debug,
{
}

/// Default implementation about how to inspect a tagged graph.
pub struct GraphDebug<'a, G>
where
    G: QueryableTaggedGraph + DirectedOrNot,
    G::LowerGraph: QueryableGraph + DirectedOrNot,
    G::Vertex: std::fmt::Debug,
    G::Edge: std::fmt::Debug,
{
    graph: &'a G,
    indent: crate::util::Indention,
}

impl<'a, G> std::fmt::Debug for GraphDebug<'a, G>
where
    G: QueryableTaggedGraph + DirectedOrNot,
    G::LowerGraph: QueryableGraph + DirectedOrNot,
    G::Vertex: std::fmt::Debug,
    G::Edge: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = &self.indent;
        let nxt_indent = self.indent.one_more_level();
        for (vid, vert) in self.graph.iter_vertices() {
            writeln!(f, "{indent}{:?}", vert)?;
            for (lower_edge, upper_edge) in self.graph.out_edges(&vid) {
                writeln!(
                    f,
                    "{nxt_indent}--{:?}-> {:?}",
                    upper_edge,
                    self.graph.vertex_by_id(&lower_edge.sink).unwrap()
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{graph::*, tagged::*};

    #[test]
    fn undirected_graph_debug() {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        struct E {
            source: VertexId,
            sink: VertexId,
        }
        impl crate::tagged::Edge for E {
            fn source(&self) -> VertexId {
                self.source
            }

            fn sink(&self) -> VertexId {
                self.sink
            }
        }
        let mut g = NaiveTaggedGraph::<String, E, undirected::TreeBackedGraph>::new();
        let vertices = ["a".to_string(), "b".to_string(), "c".to_string()];
        let mut vert_iter0 = vertices.iter();
        while let Some(v0) = vert_iter0.next() {
            let vid0 = g.overwrite_vertex(v0.clone());
            for v1 in vert_iter0.clone() {
                let vid1 = g.overwrite_vertex(v1.clone());
                g.add_edge(E {
                    source: vid0,
                    sink: vid1,
                });
            }
        }
        println!("{:?}", g.debug());

        use std::fmt::Write;
        let mut out = String::new();
        write!(&mut out, "{:?}", g.debug()).unwrap();
        assert_eq!(
            out,
            r#""a"
  --E { source: VertexId(0), sink: VertexId(1) }-> "b"
  --E { source: VertexId(0), sink: VertexId(2) }-> "c"
"b"
  --E { source: VertexId(0), sink: VertexId(1) }-> "a"
  --E { source: VertexId(1), sink: VertexId(2) }-> "c"
"c"
  --E { source: VertexId(0), sink: VertexId(2) }-> "a"
  --E { source: VertexId(1), sink: VertexId(2) }-> "b"
"#
        );
    }
}
