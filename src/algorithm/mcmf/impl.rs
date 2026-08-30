use crate::graph::EdgeId;
use crate::*;
use ahash::AHashMap;
use num_traits::bounds::UpperBounded;
use num_traits::identities::Zero;
use std::cmp::Ord;
use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

/// A tagged edge carrying a capacity and a cost per unit of flow,
/// suitable for min-cost max-flow algorithms.
pub trait FlowEdge: tagged::Edge + Sized {
    type Capacity: Ord
        + Add<Output = Self::Capacity>
        + AddAssign
        + Sub<Output = Self::Capacity>
        + SubAssign
        + Zero
        + UpperBounded
        + Copy
        + Clone;
    type Cost: Ord
        + Add<Output = Self::Cost>
        + AddAssign
        + Sub<Output = Self::Cost>
        + SubAssign
        + Neg<Output = Self::Cost>
        + Zero
        + Mul<Self::Capacity, Output = Self::Cost>
        + Copy
        + Clone;

    /// The capacity of the edge.
    fn capacity(&self) -> Self::Capacity;
    /// The cost of sending one unit of flow through the edge.
    fn cost(&self) -> Self::Cost;
}

/// The result of the minimum-cost maximum flow computation.
#[derive(Debug, Clone)]
pub struct McmfResult<Capacity, Cost> {
    /// The value of the maximum flow.
    pub max_flow: Capacity,
    /// The minimal total cost of the maximum flow.
    pub min_cost: Cost,
    /// The amount of flow pushed through each edge of the input graph.
    ///
    /// Edges with zero flow are not contained in this map.
    pub flows: AHashMap<EdgeId, Capacity>,
}
