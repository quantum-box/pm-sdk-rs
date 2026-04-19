//! Platform-agnostic traits and domain models for project management SDKs.
//!
//! This crate defines the [`PmAdapter`] trait and shared types used by
//! adapter crates such as `pm-linear` and `pm-asana`.

mod adapter;
mod error;
mod model;

pub use adapter::PmAdapter;
pub use error::{PmError, PmResult};
pub use model::*;
