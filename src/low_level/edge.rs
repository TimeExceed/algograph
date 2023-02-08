use super::VertexId;

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct EdgeId(usize);

#[derive(Clone)]
pub struct EdgeIdFactory(usize);

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Edge {
    pub id: EdgeId,
    pub source: VertexId,
    pub sink: VertexId,
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
