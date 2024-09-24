pub(crate) struct Indention {
    pub(crate) spaces: usize,
    pub(crate) step: usize,
}

impl std::fmt::Display for Indention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.spaces {
            write!(f, " ")?;
        }
        Ok(())
    }
}

impl Indention {
    pub(crate) fn one_more_level(&self) -> Self {
        Self {
            spaces: self.spaces + self.step,
            step: self.step,
        }
    }
}
