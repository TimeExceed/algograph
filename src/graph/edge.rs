use super::VertexId;

/// ID for edges, which are essentially `usize`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct EdgeId(pub usize);

/// A factory to generate `EdgeId` uniquely.
#[derive(Clone)]
pub struct EdgeIdFactory(usize);

/// Information about a low-level edge.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Edge {
    pub id: EdgeId,
    pub source: VertexId,
    pub sink: VertexId,
}

impl Default for EdgeIdFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgeIdFactory {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn one_more(&mut self) -> EdgeId {
        let cur = self.0;
        self.0 += 1;
        EdgeId(cur)
    }
}

impl EdgeId {
    pub const MIN: EdgeId = EdgeId(0);
    pub const MAX: EdgeId = EdgeId(usize::MAX);

    pub fn new(x: usize) -> Self {
        Self(x)
    }

    pub fn to_raw(&self) -> usize {
        self.0
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}
