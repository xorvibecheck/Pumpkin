use pumpkin_util::resource_location::ResourceLocation;
use serde::{Deserialize, Serialize};

/// Rewards granted when an advancement is completed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdvancementRewards {
    /// Experience points to grant.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub experience: i32,
    /// Recipes to unlock.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recipes: Vec<ResourceLocation>,
    /// Loot table to use for item rewards.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loot: Option<ResourceLocation>,
    /// Function to run when the advancement is completed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: Option<ResourceLocation>,
}

fn is_zero(v: &i32) -> bool {
    *v == 0
}

impl AdvancementRewards {
    /// Creates empty rewards.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            experience: 0,
            recipes: Vec::new(),
            loot: None,
            function: None,
        }
    }

    /// Creates rewards with only experience.
    #[must_use]
    pub const fn with_experience(experience: i32) -> Self {
        Self {
            experience,
            recipes: Vec::new(),
            loot: None,
            function: None,
        }
    }

    /// Creates rewards with recipes to unlock.
    #[must_use]
    pub fn with_recipes(recipes: Vec<ResourceLocation>) -> Self {
        Self {
            experience: 0,
            recipes,
            loot: None,
            function: None,
        }
    }

    /// Returns whether these rewards are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.experience == 0
            && self.recipes.is_empty()
            && self.loot.is_none()
            && self.function.is_none()
    }
}
