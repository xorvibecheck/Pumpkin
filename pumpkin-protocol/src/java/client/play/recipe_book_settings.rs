use pumpkin_data::packet::clientbound::PLAY_RECIPE_BOOK_SETTINGS;
use pumpkin_macros::client_packet;
use serde::Serialize;

/// Sent by the server to set recipe book settings for the player.
#[derive(Serialize)]
#[client_packet(PLAY_RECIPE_BOOK_SETTINGS)]
pub struct CRecipeBookSettings {
    pub crafting_recipe_book_open: bool,
    pub crafting_recipe_book_filter_active: bool,
    pub furnace_recipe_book_open: bool,
    pub furnace_recipe_book_filter_active: bool,
    pub blast_furnace_recipe_book_open: bool,
    pub blast_furnace_recipe_book_filter_active: bool,
    pub smoker_recipe_book_open: bool,
    pub smoker_recipe_book_filter_active: bool,
}

impl CRecipeBookSettings {
    pub fn new() -> Self {
        Self {
            crafting_recipe_book_open: false,
            crafting_recipe_book_filter_active: false,
            furnace_recipe_book_open: false,
            furnace_recipe_book_filter_active: false,
            blast_furnace_recipe_book_open: false,
            blast_furnace_recipe_book_filter_active: false,
            smoker_recipe_book_open: false,
            smoker_recipe_book_filter_active: false,
        }
    }

    pub fn with_crafting(mut self, open: bool, filter: bool) -> Self {
        self.crafting_recipe_book_open = open;
        self.crafting_recipe_book_filter_active = filter;
        self
    }

    pub fn with_furnace(mut self, open: bool, filter: bool) -> Self {
        self.furnace_recipe_book_open = open;
        self.furnace_recipe_book_filter_active = filter;
        self
    }

    pub fn with_blast_furnace(mut self, open: bool, filter: bool) -> Self {
        self.blast_furnace_recipe_book_open = open;
        self.blast_furnace_recipe_book_filter_active = filter;
        self
    }

    pub fn with_smoker(mut self, open: bool, filter: bool) -> Self {
        self.smoker_recipe_book_open = open;
        self.smoker_recipe_book_filter_active = filter;
        self
    }
}

impl Default for CRecipeBookSettings {
    fn default() -> Self {
        Self::new()
    }
}
