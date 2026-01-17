use std::io::Read;

use pumpkin_data::packet::serverbound::PLAY_RECIPE_BOOK_CHANGE_SETTINGS;
use pumpkin_macros::packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

/// Sent by the client when the player changes recipe book settings.
#[packet(PLAY_RECIPE_BOOK_CHANGE_SETTINGS)]
pub struct SRecipeBookChangeSettings {
    /// The recipe book type being modified.
    pub book_type: RecipeBookType,
    /// Whether the recipe book is open.
    pub book_open: bool,
    /// Whether filtering is active.
    pub filter_active: bool,
}

/// Types of recipe books.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeBookType {
    Crafting,
    Furnace,
    BlastFurnace,
    Smoker,
}

impl RecipeBookType {
    /// Returns the book type from its protocol value.
    #[must_use]
    pub fn from_varint(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Crafting),
            1 => Some(Self::Furnace),
            2 => Some(Self::BlastFurnace),
            3 => Some(Self::Smoker),
            _ => None,
        }
    }
}

impl ServerPacket for SRecipeBookChangeSettings {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;

        let book_type_id = read.get_var_int()?;
        let book_type = RecipeBookType::from_varint(book_type_id.0).ok_or_else(|| {
            ReadingError::Message(format!("Invalid recipe book type: {}", book_type_id.0))
        })?;

        let book_open = read.get_bool()?;
        let filter_active = read.get_bool()?;

        Ok(Self {
            book_type,
            book_open,
            filter_active,
        })
    }
}
