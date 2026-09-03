// AD-1 single-ownership: the engine's `range_compare` is the ONE comparison
// contract used by every custom_type! `compare`. It is subtype-agnostic (operates
// on the fixed-width flag+ordinal byte layout). Subtypes must NOT supply their own.
// Re-exported here so the spine's named `engine/compare.rs` is the single owner.

pub use crate::engine::canonical::compare_bytes as range_compare;
pub use crate::engine::total_cmp;
