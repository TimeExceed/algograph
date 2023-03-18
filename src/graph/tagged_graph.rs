use crate::graph::*;
use ahash::RandomState;
use bimap::BiHashMap;
use std::collections::HashMap;
use std::hash::Hash;

pub struct TaggedGraph<VKey, VTag, ETag, G = directed::TreeBackedGraph>
where
    VKey: Hash + Eq,
{
    lower_graph: G,
    vertex_keys: BiHashMap<VertexId, VKey, RandomState, RandomState>,
    vertex_tags: HashMap<VertexId, VTag, RandomState>,
    edge_tags: HashMap<EdgeId, ETag, RandomState>,
}

#[derive(Clone)]
pub struct TaggedVertex<VKey, VTag> {
    pub id: VertexId,
    pub key: VKey,
    pub tag: VTag,
}

#[derive(Clone)]
pub struct TaggedEdge<VKey, VTag, ETag> {
    pub id: EdgeId,
    pub tag: ETag,
    pub source: TaggedVertex<VKey, VTag>,
    pub sink: TaggedVertex<VKey, VTag>,
}

impl<VKey, VTag, ETag, G> TaggedGraph<VKey, VTag, ETag, G>
where
    VKey: Hash + Eq + Clone,
    G: GrowableGraph,
{
    pub fn new() -> Self {
        Self {
            lower_graph: G::new(),
            vertex_keys: BiHashMap::with_hashers(RandomState::new(), RandomState::new()),
            vertex_tags: HashMap::with_hasher(RandomState::new()),
            edge_tags: HashMap::with_hasher(RandomState::new()),
        }
    }

    pub fn overwrite_vertex(&mut self, vkey: &VKey, vtag: VTag) -> VertexId {
        if let Some(vid) = self.vertex_keys.get_by_right(vkey) {
            self.vertex_tags.insert(*vid, vtag);
            *vid
        } else {
            let vid = self.lower_graph.add_vertex();
            self.vertex_keys.insert(vid, vkey.clone());
            self.vertex_tags.insert(vid, vtag);
            vid
        }
    }

    pub fn add_edge(&mut self, v_src: &VKey, v_snk: &VKey, etag: ETag) -> EdgeId {
        let vid_src = self.vertex_id_by_key(v_src).unwrap();
        let vid_snk = self.vertex_id_by_key(v_snk).unwrap();
        let eid = self.lower_graph.add_edge(vid_src, vid_snk);
        self.edge_tags.insert(eid, etag);
        eid
    }

    pub fn update_etag(&mut self, eid: &EdgeId, etag: ETag) {
        let value = self.edge_tags.get_mut(eid).unwrap();
        *value = etag;
    }
}

impl<VKey, VTag, ETag, G> TaggedGraph<VKey, VTag, ETag, G>
where
    VKey: Hash + Eq + Clone,
    G: EdgeShrinkableGraph,
{
    pub fn remove_edge_by_id(&mut self, eid: &EdgeId) -> Option<TaggedEdge<&VKey, &VTag, ETag>> {
        self.lower_graph.remove_edge(eid).map(|e| {
            let etag = self.edge_tags.remove(eid).unwrap();
            TaggedEdge {
                id: *eid,
                tag: etag,
                source: self.vertex_by_id(&e.source).unwrap(),
                sink: self.vertex_by_id(&e.sink).unwrap(),
            }
        })
    }
}

impl<VKey, VTag, ETag, G> TaggedGraph<VKey, VTag, ETag, G>
where
    VKey: Hash + Eq + Clone + 'static,
    VTag: Clone + 'static,
    ETag: 'static,
    G: VertexShrinkableGraph,
{
    pub fn remove_vertex_by_id(
        &mut self,
        vid: &VertexId,
    ) -> Box<dyn Iterator<Item = TaggedEdge<VKey, VTag, ETag>> + 'static> {
        if let Some((_, vkey)) = self.vertex_keys.remove_by_left(vid) {
            let vtag = self.vertex_tags.remove(vid).unwrap();
            let v = TaggedVertex {
                id: *vid,
                key: vkey,
                tag: vtag,
            };
            let lower_edges: Vec<_> = self.lower_graph.remove_vertex(vid).collect();
            let etags: Vec<_> = lower_edges
                .iter()
                .map(|e| self.edge_tags.remove(&e.id).unwrap())
                .collect();
            let res: Vec<_> = lower_edges
                .into_iter()
                .zip(etags.into_iter())
                .map(|(e, etag)| {
                    let source = self
                        .vertex_by_id(&e.source)
                        .map(|v| TaggedVertex {
                            id: v.id,
                            key: v.key.clone(),
                            tag: v.tag.clone(),
                        })
                        .unwrap_or(v.clone());
                    let sink = self
                        .vertex_by_id(&e.sink)
                        .map(|v| TaggedVertex {
                            id: v.id,
                            key: v.key.clone(),
                            tag: v.tag.clone(),
                        })
                        .unwrap_or(v.clone());
                    TaggedEdge {
                        id: e.id,
                        tag: etag,
                        source,
                        sink,
                    }
                })
                .collect();
            Box::new(res.into_iter())
        } else {
            Box::new(std::iter::empty())
        }
    }

    pub fn remove_vertex_by_key(
        &mut self,
        vkey: &VKey,
    ) -> Box<dyn Iterator<Item = TaggedEdge<VKey, VTag, ETag>> + 'static> {
        if let Some(vid) = self.vertex_id_by_key(vkey) {
            self.remove_vertex_by_id(&vid)
        } else {
            Box::new(std::iter::empty())
        }
    }
}

impl<VKey, VTag, ETag, G> TaggedGraph<VKey, VTag, ETag, G>
where
    VKey: Hash + Eq,
    G: QueryableGraph,
{
    pub fn vertex_size(&self) -> usize {
        self.vertex_tags.len()
    }

    pub fn iter_vertices(&self) -> Box<dyn Iterator<Item = TaggedVertex<&VKey, &VTag>> + '_> {
        let it = self
            .lower_graph
            .iter_vertices()
            .map(|vid| self.vertex_by_id(&vid).unwrap());
        Box::new(it)
    }

    pub fn contains_vertex_by_id(&self, vid: &VertexId) -> bool {
        self.lower_graph.contains_vertex(vid)
    }

    pub fn contains_vertex_by_key(&self, vkey: &VKey) -> bool {
        self.vertex_keys.contains_right(vkey)
    }

    pub fn edge_size(&self) -> usize {
        self.edge_tags.len()
    }

    pub fn iter_edges(&self) -> Box<dyn Iterator<Item = TaggedEdge<&VKey, &VTag, &ETag>> + '_> {
        let it = self
            .lower_graph
            .iter_edges()
            .map(|e| self.edge_by_lower_edge(&e).unwrap());
        Box::new(it)
    }

    pub fn find_edge(&self, eid: &EdgeId) -> Option<TaggedEdge<&VKey, &VTag, &ETag>> {
        self.lower_graph
            .find_edge(eid)
            .and_then(|e| self.edge_by_lower_edge(&e))
    }

    pub fn contains_edge(&self, eid: &EdgeId) -> bool {
        self.edge_tags.contains_key(eid)
    }

    pub fn edges_connecting(
        &self,
        source: &VertexId,
        sink: &VertexId,
    ) -> Box<dyn Iterator<Item = TaggedEdge<&VKey, &VTag, &ETag>> + '_> {
        let it = self
            .lower_graph
            .edges_connecting(source, sink)
            .map(|e| self.edge_by_lower_edge(&e).unwrap());
        Box::new(it)
    }

    pub fn in_edges_by_id(
        &self,
        vid: &VertexId,
    ) -> Box<dyn Iterator<Item = TaggedEdge<&VKey, &VTag, &ETag>> + '_> {
        let it = self
            .lower_graph
            .in_edges(vid)
            .map(|e| self.edge_by_lower_edge(&e).unwrap());
        Box::new(it)
    }

    pub fn in_edges_by_key(
        &self,
        vkey: &VKey,
    ) -> Box<dyn Iterator<Item = TaggedEdge<&VKey, &VTag, &ETag>> + '_> {
        if let Some(vid) = self.vertex_keys.get_by_right(vkey) {
            self.in_edges_by_id(vid)
        } else {
            Box::new(std::iter::empty())
        }
    }

    pub fn out_edges_by_id(
        &self,
        vid: &VertexId,
    ) -> Box<dyn Iterator<Item = TaggedEdge<&VKey, &VTag, &ETag>> + '_> {
        let it = self
            .lower_graph
            .out_edges(vid)
            .map(|e| self.edge_by_lower_edge(&e).unwrap());
        Box::new(it)
    }

    pub fn out_edges_by_key(
        &self,
        vkey: &VKey,
    ) -> Box<dyn Iterator<Item = TaggedEdge<&VKey, &VTag, &ETag>> + '_> {
        if let Some(vid) = self.vertex_keys.get_by_right(vkey) {
            self.out_edges_by_id(vid)
        } else {
            Box::new(std::iter::empty())
        }
    }
}

impl<VKey, VTag, ETag, G> TaggedGraph<VKey, VTag, ETag, G>
where
    VKey: Hash + Eq,
{
    pub fn vertex_by_id(&self, vid: &VertexId) -> Option<TaggedVertex<&VKey, &VTag>> {
        if let Some(key) = self.vertex_key_by_id(vid) {
            let tag = self.vertex_tag_by_id(vid).unwrap();
            Some(TaggedVertex { id: *vid, key, tag })
        } else {
            None
        }
    }

    pub fn vertex_by_key<'a, 'b, 'c>(
        &'a self,
        vkey: &'b VKey,
    ) -> Option<TaggedVertex<&'c VKey, &'c VTag>>
    where
        'a: 'c,
        'b: 'c,
    {
        if let Some(id) = self.vertex_id_by_key(vkey) {
            let tag = self.vertex_tag_by_id(&id).unwrap();
            Some(TaggedVertex { id, key: vkey, tag })
        } else {
            None
        }
    }

    pub fn vertex_key_by_id(&self, vid: &VertexId) -> Option<&VKey> {
        self.vertex_keys.get_by_left(vid)
    }
    pub fn vertex_id_by_key(&self, vkey: &VKey) -> Option<VertexId> {
        self.vertex_keys.get_by_right(vkey).copied()
    }

    pub fn vertex_tag_by_key(&self, vkey: &VKey) -> Option<&VTag> {
        self.vertex_keys
            .get_by_right(vkey)
            .and_then(|vid| self.vertex_tags.get(vid))
    }

    pub fn vertex_tag_by_id(&self, vid: &VertexId) -> Option<&VTag> {
        self.vertex_tags.get(vid)
    }

    pub fn edge_tag(&self, eid: &EdgeId) -> Option<&ETag> {
        self.edge_tags.get(eid)
    }

    pub fn edge_by_lower_edge(&self, e: &Edge) -> Option<TaggedEdge<&VKey, &VTag, &ETag>> {
        match (
            self.vertex_by_id(&e.source),
            self.vertex_by_id(&e.sink),
            self.edge_tag(&e.id),
        ) {
            (Some(src), Some(snk), Some(etag)) => Some(TaggedEdge {
                id: e.id,
                tag: etag,
                source: src,
                sink: snk,
            }),
            _ => None,
        }
    }
}

impl<VKey, VTag> std::fmt::Debug for TaggedVertex<VKey, VTag>
where
    VKey: std::fmt::Debug,
    VTag: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}({:?}, {:?})", self.id, self.key, self.tag)
    }
}

impl<VKey, VTag, ETag> std::fmt::Debug for TaggedEdge<VKey, VTag, ETag>
where
    VKey: std::fmt::Debug,
    VTag: std::fmt::Debug,
    ETag: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} --{:?}({:?})-> {:?}",
            self.source, self.id, self.tag, self.sink
        )
    }
}

pub struct TaggedGraphDebug<'a, VKey, VTag, ETag, G>
where
    VKey: Hash + Eq + std::fmt::Debug,
    VTag: std::fmt::Debug,
    ETag: std::fmt::Debug,
    G: QueryableGraph,
{
    graph: &'a TaggedGraph<VKey, VTag, ETag, G>,
    init_indent: usize,
    indent_step: usize,
}

impl<'a, VKey, VTag, ETag, G> TaggedGraphDebug<'a, VKey, VTag, ETag, G>
where
    VKey: Hash + Eq + std::fmt::Debug,
    VTag: std::fmt::Debug,
    ETag: std::fmt::Debug,
    G: QueryableGraph,
{
    pub fn indent(mut self, init: usize, step: usize) -> Self {
        self.init_indent = init;
        self.indent_step = step;
        self
    }

    fn display_indent(&self, f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result {
        let indention = self.init_indent + self.indent_step * level;
        for _ in 0..indention {
            write!(f, " ")?;
        }
        Ok(())
    }
}

impl<'a, VKey, VTag, ETag, G> std::fmt::Debug for TaggedGraphDebug<'a, VKey, VTag, ETag, G>
where
    VKey: Hash + Eq + std::fmt::Debug,
    VTag: std::fmt::Debug,
    ETag: std::fmt::Debug,
    G: QueryableGraph,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for v in self.graph.iter_vertices() {
            self.display_indent(f, 0)?;
            writeln!(f, "{:?}", v)?;
            for e in self.graph.out_edges_by_id(&v.id) {
                self.display_indent(f, 1)?;
                writeln!(f, "--{:?}({:?})-> {:?}", e.id, e.tag, e.sink)?;
            }
        }
        Ok(())
    }
}
