use pumpkin_data::packet::clientbound::PLAY_RECIPE_BOOK_ADD;
use pumpkin_macros::client_packet;
use pumpkin_util::resource_location::ResourceLocation;
use serde::Serialize;

use crate::codec::var_int::VarInt;

/// Sent by the server to unlock recipes for the player.
#[derive(Serialize)]
#[client_packet(PLAY_RECIPE_BOOK_ADD)]
pub struct CRecipeBookAdd {
    pub entries: Vec<RecipeBookEntry>,
    pub replace: bool,
}

impl CRecipeBookAdd {
    pub fn new(entries: Vec<RecipeBookEntry>, replace: bool) -> Self {
        Self { entries, replace }
    }

    /// Creates a packet to unlock new recipes (add to existing)
    pub fn add_recipes(recipe_ids: Vec<ResourceLocation>) -> Self {
        let entries = recipe_ids
            .into_iter()
            .map(|id| RecipeBookEntry {
                recipe_id: VarInt(0), // Recipe display ID from registry
                display_id: id,
                group_id: VarInt(0),
                category: VarInt(0),
                has_ingredients: false,
                ingredients: vec![],
                flags: RecipeBookEntryFlags::default(),
            })
            .collect();
        Self::new(entries, false)
    }

    /// Creates a packet to replace all recipes (used on login)
    pub fn init_recipes(recipe_ids: Vec<ResourceLocation>) -> Self {
        let entries = recipe_ids
            .into_iter()
            .map(|id| RecipeBookEntry {
                recipe_id: VarInt(0),
                display_id: id,
                group_id: VarInt(0),
                category: VarInt(0),
                has_ingredients: false,
                ingredients: vec![],
                flags: RecipeBookEntryFlags::default(),
            })
            .collect();
        Self::new(entries, true)
    }
}

#[derive(Serialize)]
pub struct RecipeBookEntry {
    pub recipe_id: VarInt,
    pub display_id: ResourceLocation,
    pub group_id: VarInt,
    pub category: VarInt,
    pub has_ingredients: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ingredients: Vec<RecipeIngredient>,
    pub flags: RecipeBookEntryFlags,
}

#[derive(Serialize)]
pub struct RecipeIngredient {
    pub items: Vec<VarInt>,
}

#[derive(Serialize, Default)]
pub struct RecipeBookEntryFlags {
    flags: u8,
}

impl RecipeBookEntryFlags {
    pub fn new(unlocked: bool, highlighted: bool) -> Self {
        let mut flags = 0u8;
        if unlocked {
            flags |= 0x01;
        }
        if highlighted {
            flags |= 0x02;
        }
        Self { flags }
    }
}
