use pumpkin_data::packet::clientbound::PLAY_RECIPE_BOOK_REMOVE;
use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::codec::var_int::VarInt;

/// Sent by the server to remove/lock recipes for the player.
#[derive(Serialize)]
#[client_packet(PLAY_RECIPE_BOOK_REMOVE)]
pub struct CRecipeBookRemove {
    pub recipe_ids: Vec<VarInt>,
}

impl CRecipeBookRemove {
    pub fn new(recipe_ids: Vec<VarInt>) -> Self {
        Self { recipe_ids }
    }
}
