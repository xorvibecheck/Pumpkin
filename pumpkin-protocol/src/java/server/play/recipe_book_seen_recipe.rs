use std::io::Read;

use pumpkin_data::packet::serverbound::PLAY_RECIPE_BOOK_SEEN_RECIPE;
use pumpkin_macros::packet;
use pumpkin_util::resource_location::ResourceLocation;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

/// Sent by the client when the player clicks a recipe in the recipe book.
#[packet(PLAY_RECIPE_BOOK_SEEN_RECIPE)]
pub struct SRecipeBookSeenRecipe {
    /// The recipe identifier that was viewed/clicked.
    pub recipe_id: ResourceLocation,
}

impl ServerPacket for SRecipeBookSeenRecipe {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;
        let recipe_id = read.get_resource_location()?;
        Ok(Self { recipe_id })
    }
}
