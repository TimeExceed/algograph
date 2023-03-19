# algograph

Directed or undirected graphs and their algorithms implemented in Rust.

## Low-level graphs and `TaggedGraph`

Some graph libraries allow customized types of vertices and edges.
But for algorithm authors, these customized types are hard to deal with.
Can we copy a vertex?
What is the cost of doing that copying?

In this crate, there are traits and implements of low level graphs.
Vertices and edges in low level graphs are lightweight ID's.
They are essentially `usize`.
Algorithm authors may feel free to copy and store these ID's.

There is also `TaggedGraph` to let vertices and edges be tagged.
Users may usually experience `TaggedGraph` as easy as those with customized vertice types and edge types in other crates.

## `ShadowedSubgraph` and `SelectedSubgraph`

They can form subgraphs with shadowed/selected vertices and edges.
Futhermore, these subgraphs are shrinkable.
While they are shrinking, their underlying graphs are kept unchanged.
