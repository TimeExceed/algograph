//! Traits and a naive implementation to support graphs with customzied vertex types and edge types.
//!
//! ```plain
//! Queryable    Growable   VertexShrinkable
//!     |            |             |
//!     |            |             v
//!     |            |       EdgeShrinkable
//!     |            |             |
//!     |            v             |
//!     +-------->  Base <---------+
//! ```
mod traits;
pub use self::traits::*;

mod naive_impl;
pub use self::naive_impl::*;
