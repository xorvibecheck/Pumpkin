use serde::{Deserialize, Serialize};

/// The frame type of an advancement, which determines its appearance in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdvancementFrame {
    /// A regular task advancement (square frame, green title).
    #[default]
    Task,
    /// A goal advancement (rounded frame, green title).
    Goal,
    /// A challenge advancement (spiked frame, purple title).
    Challenge,
}

impl AdvancementFrame {
    /// Returns the ordinal value used in packet serialization.
    #[must_use]
    pub const fn ordinal(self) -> i32 {
        match self {
            Self::Task => 0,
            Self::Goal => 1,
            Self::Challenge => 2,
        }
    }

    /// Creates an `AdvancementFrame` from its ordinal value.
    #[must_use]
    pub const fn from_ordinal(ordinal: i32) -> Option<Self> {
        match ordinal {
            0 => Some(Self::Task),
            1 => Some(Self::Goal),
            2 => Some(Self::Challenge),
            _ => None,
        }
    }

    /// Returns the string identifier for this frame type.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Goal => "goal",
            Self::Challenge => "challenge",
        }
    }
}
