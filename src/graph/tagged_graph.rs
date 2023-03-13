use crate::graph::*;
use ahash::RandomState;
use bimap::BiHashMap;
use std::collections::HashMap;
use std::hash::Hash;

pub struct TaggedGraph<VKey, VTag, ETag, G = directed::TreeBackedGraph>
where
    VKey: Hash + Eq,
{
    low_graph: G,
    vertex_keys: BiHashMap<VertexId, VKey, RandomState, RandomState>,
    vertex_tags: HashMap<VertexId, VTag, RandomState>,
    edge_tags: HashMap<EdgeId, ETag, RandomState>,
}

impl<VKey, VTag, ETag, G> TaggedGraph<VKey, VTag, ETag, G>
where
    VKey: Hash + Eq,
    G: GrowableGraph,
{
    pub fn new() -> Self {
        Self {
            low_graph: G::new(),
            vertex_keys: BiHashMap::with_hashers(RandomState::new(), RandomState::new()),
            vertex_tags: HashMap::with_hasher(RandomState::new()),
            edge_tags: HashMap::with_hasher(RandomState::new()),
        }
    }
}
