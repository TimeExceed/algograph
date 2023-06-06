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
    fn iter_edges(&self) -> Box<dyn Iterator<Item = (EdgeId, &Self::Edge)> + '_>;
    /// Iterates edges connecting two specified endpoints.
    fn edges_connecting(
        &self,
        source: &VertexId,
        sink: &VertexId,
    ) -> Box<dyn Iterator<Item = (EdgeId, &Self::Edge)> + '_>;
    /// Iterates over in-edges of a specified vertex.
    fn in_edges(&self, vid: &VertexId) -> Box<dyn Iterator<Item = (EdgeId, &Self::Edge)> + '_>;
    /// Iterates over out-edges of a specified vertex.
    fn out_edges(&self, vid: &VertexId) -> Box<dyn Iterator<Item = (EdgeId, &Self::Edge)> + '_>;
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
    fn debug(&self, init: usize, indent: usize) -> GraphDebug<'_, Self> {
        GraphDebug {
            graph: self,
            indent: Indention {
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
    indent: Indention,
}

impl<'a, G> std::fmt::Debug for GraphDebug<'a, G>
where
    G: QueryableTaggedGraph + DirectedOrNot,
    G::LowerGraph: QueryableGraph + DirectedOrNot,
    G::Vertex: std::fmt::Debug,
    G::Edge: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = self.indent.one_more_level();
        for (vid, vert) in self.graph.iter_vertices() {
            writeln!(f, "{}{:?}", self.indent, vert)?;
            for (_, edge) in self.graph.out_edges(&vid) {
                writeln!(
                    f,
                    "{}--{:?}-> {:?}",
                    indent,
                    edge,
                    self.graph.vertex_by_id(&edge.sink()).unwrap()
                )?;
            }
        }
        Ok(())
    }
}

struct Indention {
    spaces: usize,
    step: usize,
}

impl std::fmt::Display for Indention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.spaces {
            write!(f, " ")?;
        }
        Ok(())
    }
}

impl Indention {
    fn one_more_level(&self) -> Self {
        Self {
            spaces: self.spaces + self.step,
            step: self.step,
        }
    }
}
