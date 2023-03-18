mod adjacent_list;
pub use self::adjacent_list::*;
mod tree_backed;
pub use self::tree_backed::*;

#[cfg(test)]
pub use self::tests::*;

#[cfg(test)]
mod tests {
    use crate::graph::*;
    use rs_quickcheck_util::*;
    use std::collections::BTreeSet;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Op {
        AddVertex(VertexId),
        RemoveVertex(VertexId),
        AddEdge((VertexId, VertexId, EdgeId)),
        RemoveEdge(EdgeId),
    }

    #[derive(Clone)]
    pub struct Ops {
        pub ops: Vec<Op>,
    }

    impl std::fmt::Debug for Ops {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.ops)
        }
    }

    impl Ops {
        pub fn iter(&self) -> impl Iterator<Item = &Op> + '_ {
            self.ops.iter()
        }
    }

    impl quickcheck::Arbitrary for Ops {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut vid_factory = VertexIdFactory::new();
            let mut eid_factory = EdgeIdFactory::new();
            let mut known_vid = BTreeSet::new();
            let mut known_eid = BTreeSet::new();
            let ops = gen_bytes(g, b"abcd.", b'.', 0..)
                .iter()
                .filter_map(|_| match u8::arbitrary(g) % 4 {
                    0 => {
                        let vid = vid_factory.one_more();
                        known_vid.insert(vid);
                        Some(Op::AddVertex(vid))
                    }
                    1 => {
                        if known_vid.is_empty() {
                            None
                        } else {
                            let vid = {
                                let idx = usize::arbitrary(g) % known_vid.len();
                                *known_vid.iter().nth(idx).unwrap()
                            };
                            known_vid.remove(&vid);
                            Some(Op::RemoveVertex(vid))
                        }
                    }
                    2 => {
                        if known_vid.is_empty() {
                            None
                        } else {
                            let src_vid = {
                                let idx = usize::arbitrary(g) % known_vid.len();
                                *known_vid.iter().nth(idx).unwrap()
                            };
                            let sink_vid = {
                                let idx = usize::arbitrary(g) % known_vid.len();
                                *known_vid.iter().nth(idx).unwrap()
                            };
                            let eid = eid_factory.one_more();
                            known_eid.insert(eid);
                            Some(Op::AddEdge((src_vid, sink_vid, eid)))
                        }
                    }
                    3 => {
                        if known_eid.is_empty() {
                            None
                        } else {
                            let eid = {
                                let idx = usize::arbitrary(g) % known_eid.len();
                                *known_eid.iter().nth(idx).unwrap()
                            };
                            known_eid.remove(&eid);
                            Some(Op::RemoveEdge(eid))
                        }
                    }
                    _ => unreachable!(),
                })
                .collect();
            Self { ops }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            let l = self.ops.len();
            let me = self.clone();
            let it = std::iter::successors(Some(l / 2), move |n| {
                let nxt = (n + l) / 2 + 1;
                if nxt >= l {
                    None
                } else {
                    Some(nxt)
                }
            })
            .map(move |n| {
                let mut res = me.clone();
                res.ops = me.ops[0..n].iter().map(|x| x.clone()).collect();
                res
            });
            Box::new(it)
        }
    }
}
