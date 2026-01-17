mod player_tracker;
mod registry;
mod triggers;

pub use player_tracker::*;
pub use registry::*;
pub use triggers::*;

/// Type alias for backward compatibility
pub type ServerAdvancementRegistry = AdvancementRegistry;
