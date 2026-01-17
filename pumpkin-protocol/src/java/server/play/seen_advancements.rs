use std::io::Read;

use pumpkin_data::packet::serverbound::PLAY_SEEN_ADVANCEMENTS;
use pumpkin_macros::packet;
use pumpkin_util::resource_location::ResourceLocation;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

/// Sent by the client when the player interacts with the advancement screen.
#[packet(PLAY_SEEN_ADVANCEMENTS)]
pub struct SSeenAdvancements {
    /// The action being performed.
    pub action: SeenAdvancementsAction,
    /// The tab identifier (only present if action is OpenedTab).
    pub tab_id: Option<ResourceLocation>,
}

/// Actions for the seen advancements packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeenAdvancementsAction {
    /// The player opened an advancement tab.
    OpenedTab,
    /// The player closed the advancement screen.
    ClosedScreen,
}

impl SeenAdvancementsAction {
    /// Returns the action from its protocol value.
    #[must_use]
    pub fn from_varint(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::OpenedTab),
            1 => Some(Self::ClosedScreen),
            _ => None,
        }
    }

    /// Returns the protocol value for this action.
    #[must_use]
    pub const fn to_varint(self) -> i32 {
        match self {
            Self::OpenedTab => 0,
            Self::ClosedScreen => 1,
        }
    }
}

impl ServerPacket for SSeenAdvancements {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;

        let action_id = read.get_var_int()?;
        let action = SeenAdvancementsAction::from_varint(action_id.0).ok_or_else(|| {
            ReadingError::Message(format!("Invalid seen advancements action: {}", action_id.0))
        })?;

        let tab_id = if action == SeenAdvancementsAction::OpenedTab {
            Some(read.get_resource_location()?)
        } else {
            None
        };

        Ok(Self { action, tab_id })
    }
}
