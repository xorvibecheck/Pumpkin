use pumpkin_data::packet::clientbound::PLAY_PLACE_GHOST_RECIPE;
use pumpkin_macros::client_packet;
use pumpkin_util::resource_location::ResourceLocation;
use serde::Serialize;

use crate::codec::var_int::VarInt;

/// Sent by the server to place a ghost recipe in the crafting grid.
/// This shows the recipe ingredients as ghost items in the crafting UI.
#[derive(Serialize)]
#[client_packet(PLAY_PLACE_GHOST_RECIPE)]
pub struct CPlaceGhostRecipe {
    /// The window ID of the crafting table/inventory
    pub window_id: VarInt,
    /// The recipe to display
    pub recipe: ResourceLocation,
}

impl CPlaceGhostRecipe {
    pub fn new(window_id: i32, recipe: ResourceLocation) -> Self {
        Self {
            window_id: VarInt(window_id),
            recipe,
        }
    }
}
