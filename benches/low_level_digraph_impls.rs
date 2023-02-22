use algograph::graph::{directed::*, *};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use static_init::dynamic;

#[dynamic]
static VERTEX_SIZE: usize = std::env::var("VERTEX_SIZE")
    .unwrap_or("10000".to_string())
    .parse()
    .unwrap();
#[dynamic]
static EDGE_SIZE: usize = std::env::var("EDGE_SIZE")
    .unwrap_or("100000".to_string())
    .parse()
    .unwrap();

criterion_group!(benches, tree_backed, petgraph_backed);
criterion_main!(benches);

fn tree_backed(c: &mut Criterion) {
    cases::<TreeBackedGraph>(c, "tree_backed");
}

fn petgraph_backed(c: &mut Criterion) {
    cases::<PetgraphBackedGraph>(c, "petgraph_backed");
}

fn cases<G>(c: &mut Criterion, prefix: &str)
where
    G: GrowableGraph + QueryableGraph + EdgeShrinkableGraph + VertexShrinkableGraph + Clone,
{
    let vertex_size = *VERTEX_SIZE;
    println!("VERTEX_SIZE: {}", vertex_size);
    let edge_size = *EDGE_SIZE;
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
    for _ in 0..edge_size {
        let v0 = vertices[rand::thread_rng().gen::<usize>() % vertices.len()];
        let v1 = vertices[rand::thread_rng().gen::<usize>() % vertices.len()];
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
    for _ in 0..edge_size {
        let v0 = vertices[rand::thread_rng().gen::<usize>() % vertices.len()];
        let v1 = vertices[rand::thread_rng().gen::<usize>() % vertices.len()];
        let _ = g.add_edge(v0, v1);
    }
}

fn contains_vertex<G>(g: &G, vertices: &[VertexId])
where
    G: QueryableGraph,
{
    let vid = vertices[rand::thread_rng().gen::<usize>() % vertices.len()];
    g.contains_vertex(&vid);
}

fn contains_edge<G>(g: &G, edges: &[EdgeId])
where
    G: QueryableGraph,
{
    let eid = edges[rand::thread_rng().gen::<usize>() % edges.len()];
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
