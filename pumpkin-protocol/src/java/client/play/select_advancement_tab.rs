use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_SELECT_ADVANCEMENTS_TAB;
use pumpkin_macros::packet;
use pumpkin_util::resource_location::ResourceLocation;

use crate::ser::NetworkWriteExt;
use crate::{ClientPacket, WritingError};

/// Sent by the server to select a specific advancement tab for the client.
#[packet(PLAY_SELECT_ADVANCEMENTS_TAB)]
pub struct CSelectAdvancementTab {
    /// The identifier of the tab to select, or `None` to close the advancement screen.
    pub tab_id: Option<ResourceLocation>,
}

impl CSelectAdvancementTab {
    /// Creates a new packet to select an advancement tab.
    #[must_use]
    pub fn new(tab_id: Option<ResourceLocation>) -> Self {
        Self { tab_id }
    }

    /// Creates a packet to select a specific tab.
    #[must_use]
    pub fn select(tab_id: ResourceLocation) -> Self {
        Self {
            tab_id: Some(tab_id),
        }
    }

    /// Creates a packet to deselect all tabs (close screen).
    #[must_use]
    pub fn deselect() -> Self {
        Self { tab_id: None }
    }
}

impl ClientPacket for CSelectAdvancementTab {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;
        write.write_option(&self.tab_id, |w, tab| w.write_resource_location(tab))
    }
}
