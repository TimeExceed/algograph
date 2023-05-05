//! Implementations of low-level undirected graph

mod tree_backed;
pub use self::tree_backed::*;
mod adjacent_list;
pub use self::adjacent_list::*;

#[cfg(test)]
mod tests {
    use crate::graph::*;

    #[test]
    fn to_graphviz() {
        let mut g = undirected::AdjacentListGraph::new();
        let v = g.add_vertex();
        g.add_edge(v, v);
        let trial = {
            let mut trial = vec![];
            g.dump_in_graphviz(&mut trial, "trial").unwrap();
            String::from_utf8(trial).unwrap()
        };
        assert_eq!(
            trial,
            r#"graph trial {
  0 ;
  0 -- 0 ;
}
"#
        );
    }
}
