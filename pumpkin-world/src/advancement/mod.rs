//! Advancement system for Minecraft.
//!
//! This module contains all the data structures and logic for the advancement system,
//! including advancement definitions, display information, progress tracking, and criteria.

#[allow(clippy::module_inception)]
mod advancement;
mod criterion;
mod display;
mod frame;
mod loader;
mod progress;
mod requirements;
mod rewards;

pub use advancement::*;
pub use criterion::*;
pub use display::*;
pub use frame::*;
pub use loader::*;
pub use progress::*;
pub use requirements::*;
pub use rewards::*;
