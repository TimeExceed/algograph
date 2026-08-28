use algograph::graph::{directed::*, *};
use criterion::{Criterion, criterion_group, criterion_main};
use rand::seq::IndexedRandom;
use std::hint::black_box;

criterion_group!(benches, tree_backed, adjacent_list);
criterion_main!(benches);

fn tree_backed(c: &mut Criterion) {
    cases::<TreeBackedGraph>(c, "tree_backed");
}

fn adjacent_list(c: &mut Criterion) {
    cases::<AdjacentListGraph>(c, "adjacent_list");
}

fn cases<G>(c: &mut Criterion, prefix: &str)
where
    G: GrowableGraph + QueryableGraph + EdgeShrinkableGraph + VertexShrinkableGraph + Clone,
{
    let vertex_size: usize = std::env::var("VERTEX_SIZE")
        .unwrap()
        .parse()
        .unwrap_or(10000);
    println!("VERTEX_SIZE: {}", vertex_size);
    let edge_size: usize = std::env::var("EDGE_SIZE")
        .unwrap()
        .parse()
        .unwrap_or(100000);
    println!("EDGE_SIZE: {}", edge_size);
    c.bench_function(&(prefix.to_string() + "/add_vertex"), |b| {
        b.iter(|| add_vertices::<G>(vertex_size))
    });
    c.bench_function(&(prefix.to_string() + "/add_vertex and add_edge"), |b| {
        b.iter(|| add_vertices_and_edges::<G>(vertex_size, edge_size))
    });

    let mut g = G::new();
    let mut vertices = vec![];
    let mut edges = vec![];
    for _ in 0..vertex_size {
        let vid = g.add_vertex();
        vertices.push(vid);
    }
    let rng = &mut rand::rng();
    for _ in 0..edge_size {
        let v0 = *vertices.choose(rng).unwrap();
        let v1 = *vertices.choose(rng).unwrap();
        let eid = g.add_edge(v0, v1);
        edges.push(eid);
    }
    c.bench_function(&(prefix.to_string() + "/iter_vertices"), |b| {
        b.iter(|| iter_vertices(&g))
    });
    c.bench_function(&(prefix.to_string() + "/iter_edges"), |b| {
        b.iter(|| iter_edges(&g))
    });
    c.bench_function(&(prefix.to_string() + "/contains_vertex"), |b| {
        b.iter(|| contains_vertex(&g, &vertices))
    });
    c.bench_function(&(prefix.to_string() + "/contains_edge"), |b| {
        b.iter(|| contains_edge(&g, &edges))
    });
    c.bench_function(&(prefix.to_string() + "/remove_edges"), |b| {
        let mut g = g.clone();
        b.iter(|| remove_edges(&mut g, &edges))
    });
    c.bench_function(&(prefix.to_string() + "/remove_vertices"), |b| {
        let mut g = g.clone();
        b.iter(|| remove_vertices(&mut g, &vertices))
    });
}

fn add_vertices<G>(vertex_size: usize)
where
    G: GrowableGraph,
{
    let mut g = G::new();
    for _ in 0..vertex_size {
        let _ = g.add_vertex();
    }
}

fn add_vertices_and_edges<G>(vertex_size: usize, edge_size: usize)
where
    G: GrowableGraph,
{
    let mut g = G::new();
    let mut vertices = vec![];
    for _ in 0..vertex_size {
        let vid = g.add_vertex();
        vertices.push(vid);
    }
    let rng = &mut rand::rng();
    for _ in 0..edge_size {
        let v0 = *vertices.choose(rng).unwrap();
        let v1 = *vertices.choose(rng).unwrap();
        let _ = g.add_edge(v0, v1);
    }
}

fn contains_vertex<G>(g: &G, vertices: &[VertexId])
where
    G: QueryableGraph,
{
    let vid = *vertices.choose(&mut rand::rng()).unwrap();
    g.contains_vertex(&vid);
}

fn contains_edge<G>(g: &G, edges: &[EdgeId])
where
    G: QueryableGraph,
{
    let eid = *edges.choose(&mut rand::rng()).unwrap();
    g.contains_edge(&eid);
}

fn iter_vertices<G>(g: &G)
where
    G: QueryableGraph,
{
    for x in g.iter_vertices() {
        black_box(x.to_raw());
    }
}

fn iter_edges<G>(g: &G)
where
    G: QueryableGraph,
{
    for x in g.iter_edges() {
        black_box(x.id.to_raw());
    }
}

fn remove_edges<G>(g: &mut G, edges: &[EdgeId])
where
    G: EdgeShrinkableGraph,
{
    for e in edges {
        g.remove_edge(e);
    }
}

fn remove_vertices<G>(g: &mut G, vertices: &[VertexId])
where
    G: VertexShrinkableGraph,
{
    for v in vertices {
        let _ = black_box(g.remove_vertex(v));
    }
}
