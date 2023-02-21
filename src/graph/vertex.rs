#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct VertexId(pub usize);

#[derive(Clone)]
pub struct VertexIdFactory(usize);

impl VertexIdFactory {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn one_more(&mut self) -> VertexId {
        let cur = self.0;
        self.0 += 1;
        VertexId(cur)
    }
}

impl VertexId {
    pub const MIN: VertexId = VertexId(0);
    pub const MAX: VertexId = VertexId(usize::MAX);

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
