use std::io::Read;

use pumpkin_data::packet::serverbound::PLAY_PLACE_RECIPE;
use pumpkin_macros::packet;
use pumpkin_util::resource_location::ResourceLocation;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

/// Sent by the client when the player clicks on a recipe in the recipe book.
/// This requests the server to place the recipe ingredients into the crafting grid.
#[packet(PLAY_PLACE_RECIPE)]
pub struct SPlaceRecipe {
    /// The window ID of the crafting table/inventory
    pub window_id: i8,
    /// The recipe the player wants to craft
    pub recipe: ResourceLocation,
    /// Whether to place all available items (shift-click)
    pub make_all: bool,
}

impl ServerPacket for SPlaceRecipe {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;
        let window_id = read.get_i8()?;
        let recipe = read.get_resource_location()?;
        let make_all = read.get_bool()?;
        Ok(Self {
            window_id,
            recipe,
            make_all,
        })
    }
}
