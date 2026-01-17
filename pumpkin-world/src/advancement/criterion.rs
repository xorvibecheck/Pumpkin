use pumpkin_util::resource_location::ResourceLocation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A criterion that must be met to complete an advancement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancementCriterion {
    /// The trigger type (e.g., "minecraft:inventory_changed").
    pub trigger: ResourceLocation,
    /// The conditions for this criterion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conditions: Option<CriterionConditions>,
}

impl AdvancementCriterion {
    /// Creates a new criterion with the given trigger and no conditions.
    #[must_use]
    pub fn new(trigger: ResourceLocation) -> Self {
        Self {
            trigger,
            conditions: None,
        }
    }

    /// Creates a new criterion with the given trigger and conditions.
    #[must_use]
    pub fn with_conditions(trigger: ResourceLocation, conditions: CriterionConditions) -> Self {
        Self {
            trigger,
            conditions: Some(conditions),
        }
    }
}

/// Conditions for a criterion trigger.
///
/// The structure of conditions varies based on the trigger type.
/// This is a flexible JSON-like structure to accommodate all trigger types.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CriterionConditions {
    /// The player conditions (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player: Option<serde_json::Value>,
    /// Additional condition fields depending on trigger type.
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

impl CriterionConditions {
    /// Creates empty conditions.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            player: None,
            other: HashMap::new(),
        }
    }

    /// Creates conditions with only player conditions.
    #[must_use]
    pub fn with_player(player: serde_json::Value) -> Self {
        Self {
            player: Some(player),
            other: HashMap::new(),
        }
    }

    /// Get a condition field by key
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.other.get(key)
    }

    /// Get items condition (for inventory_changed, consume_item, etc.)
    #[must_use]
    pub fn get_items(&self) -> Option<&serde_json::Value> {
        self.other.get("items").or_else(|| self.other.get("item"))
    }

    /// Get entity condition (for player_killed_entity, etc.)
    #[must_use]
    pub fn get_entity(&self) -> Option<&serde_json::Value> {
        self.other.get("entity")
    }

    /// Get effects condition (for effects_changed)
    #[must_use]
    pub fn get_effects(&self) -> Option<&serde_json::Value> {
        self.other.get("effects")
    }

    /// Get block condition
    #[must_use]
    pub fn get_block(&self) -> Option<&serde_json::Value> {
        self.other.get("block")
    }

    /// Get location condition
    #[must_use]
    pub fn get_location(&self) -> Option<&serde_json::Value> {
        self.other.get("location")
    }

    /// Get distance condition
    #[must_use]
    pub fn get_distance(&self) -> Option<&serde_json::Value> {
        self.other.get("distance")
    }

    /// Get from/to dimension conditions
    #[must_use]
    pub fn get_from_dimension(&self) -> Option<&str> {
        self.other.get("from").and_then(|v| v.as_str())
    }

    #[must_use]
    pub fn get_to_dimension(&self) -> Option<&str> {
        self.other.get("to").and_then(|v| v.as_str())
    }

    /// Get recipe_id for recipe_crafted
    #[must_use]
    pub fn get_recipe_id(&self) -> Option<&str> {
        self.other.get("recipe_id").and_then(|v| v.as_str())
    }

    /// Get potion for brewed_potion
    #[must_use]
    pub fn get_potion(&self) -> Option<&str> {
        self.other.get("potion").and_then(|v| v.as_str())
    }

    /// Get level for construct_beacon
    #[must_use]
    pub fn get_level(&self) -> Option<i64> {
        self.other.get("level").and_then(|v| {
            if let Some(obj) = v.as_object() {
                obj.get("min").and_then(|m| m.as_i64())
            } else {
                v.as_i64()
            }
        })
    }
}

/// A map of criterion names to their definitions.
pub type CriteriaMap = HashMap<String, AdvancementCriterion>;

/// All 53 vanilla criterion trigger types as lazy constants.
/// These correspond exactly to the triggers registered in Minecraft's Criteria.java
pub mod triggers {
    use pumpkin_util::resource_location::ResourceLocation;
    use std::sync::LazyLock;

    // ===== Core Triggers =====
    pub static IMPOSSIBLE: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("impossible"));
    pub static TICK: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("tick"));

    // ===== Kill/Death Triggers =====
    pub static PLAYER_KILLED_ENTITY: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("player_killed_entity"));
    pub static ENTITY_KILLED_PLAYER: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("entity_killed_player"));
    pub static KILL_MOB_NEAR_SCULK_CATALYST: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("kill_mob_near_sculk_catalyst"));
    pub static KILLED_BY_ARROW: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("killed_by_crossbow")); // Note: named killed_by_crossbow in game

    // ===== Block/Location Triggers =====
    pub static ENTER_BLOCK: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("enter_block"));
    pub static LOCATION: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("location"));
    pub static SLEPT_IN_BED: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("slept_in_bed"));
    pub static HERO_OF_THE_VILLAGE: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("hero_of_the_village"));
    pub static VOLUNTARY_EXILE: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("voluntary_exile"));
    pub static SLIDE_DOWN_BLOCK: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("slide_down_block"));
    pub static PLACED_BLOCK: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("placed_block"));
    pub static ANY_BLOCK_USE: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("any_block_use"));
    pub static DEFAULT_BLOCK_USE: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("default_block_use"));

    // ===== Inventory/Item Triggers =====
    pub static INVENTORY_CHANGED: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("inventory_changed"));
    pub static CONSUME_ITEM: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("consume_item"));
    pub static USING_ITEM: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("using_item"));
    pub static ITEM_USED_ON_BLOCK: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("item_used_on_block"));
    pub static FILLED_BUCKET: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("filled_bucket"));
    pub static ENCHANTED_ITEM: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("enchanted_item"));
    pub static ITEM_DURABILITY_CHANGED: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("item_durability_changed"));
    pub static THROWN_ITEM_PICKED_UP_BY_ENTITY: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("thrown_item_picked_up_by_entity"));
    pub static THROWN_ITEM_PICKED_UP_BY_PLAYER: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("thrown_item_picked_up_by_player"));

    // ===== Recipe Triggers =====
    pub static RECIPE_UNLOCKED: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("recipe_unlocked"));
    pub static RECIPE_CRAFTED: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("recipe_crafted"));
    pub static CRAFTER_RECIPE_CRAFTED: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("crafter_recipe_crafted"));

    // ===== Combat/Damage Triggers =====
    pub static PLAYER_HURT_ENTITY: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("player_hurt_entity"));
    pub static ENTITY_HURT_PLAYER: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("entity_hurt_player"));
    pub static USED_TOTEM: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("used_totem"));
    pub static SHOT_CROSSBOW: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("shot_crossbow"));
    pub static KILLED_BY_CROSSBOW: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("killed_by_crossbow"));
    pub static TARGET_HIT: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("target_hit"));
    pub static CHANNELED_LIGHTNING: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("channeled_lightning"));
    pub static LIGHTNING_STRIKE: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("lightning_strike"));
    pub static SPEAR_MOBS: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("spear_mobs"));

    // ===== Travel/Movement Triggers =====
    pub static CHANGED_DIMENSION: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("changed_dimension"));
    pub static NETHER_TRAVEL: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("nether_travel"));
    pub static FALL_FROM_HEIGHT: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("fall_from_height"));
    pub static RIDE_ENTITY_IN_LAVA: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("ride_entity_in_lava"));
    pub static STARTED_RIDING: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("started_riding"));
    pub static LEVITATION: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("levitation"));
    pub static FALL_AFTER_EXPLOSION: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("fall_after_explosion"));
    pub static AVOID_VIBRATION: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("avoid_vibration"));

    // ===== Entity Interaction Triggers =====
    pub static PLAYER_INTERACTED_WITH_ENTITY: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("player_interacted_with_entity"));
    pub static PLAYER_SHEARED_EQUIPMENT: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("player_sheared_equipment"));
    pub static BRED_ANIMALS: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("bred_animals"));
    pub static TAME_ANIMAL: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("tame_animal"));
    pub static SUMMONED_ENTITY: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("summoned_entity"));
    pub static CURED_ZOMBIE_VILLAGER: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("cured_zombie_villager"));
    pub static VILLAGER_TRADE: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("villager_trade"));
    pub static FISHING_ROD_HOOKED: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("fishing_rod_hooked"));
    pub static ALLAY_DROP_ITEM_ON_BLOCK: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("allay_drop_item_on_block"));

    // ===== Brewing/Beacon Triggers =====
    pub static BREWED_POTION: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("brewed_potion"));
    pub static EFFECTS_CHANGED: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("effects_changed"));
    pub static CONSTRUCT_BEACON: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("construct_beacon"));

    // ===== Special Triggers =====
    pub static USED_ENDER_EYE: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("used_ender_eye"));
    pub static PLAYER_GENERATES_CONTAINER_LOOT: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("player_generates_container_loot"));
    pub static BEE_NEST_DESTROYED: LazyLock<ResourceLocation> = LazyLock::new(|| ResourceLocation::vanilla("bee_nest_destroyed"));

    // ===== Function-style triggers for backward compatibility =====
    pub fn impossible() -> ResourceLocation { IMPOSSIBLE.clone() }
    pub fn tick() -> ResourceLocation { TICK.clone() }
    pub fn player_killed_entity() -> ResourceLocation { PLAYER_KILLED_ENTITY.clone() }
    pub fn entity_killed_player() -> ResourceLocation { ENTITY_KILLED_PLAYER.clone() }
    pub fn kill_mob_near_sculk_catalyst() -> ResourceLocation { KILL_MOB_NEAR_SCULK_CATALYST.clone() }
    pub fn enter_block() -> ResourceLocation { ENTER_BLOCK.clone() }
    pub fn location() -> ResourceLocation { LOCATION.clone() }
    pub fn slept_in_bed() -> ResourceLocation { SLEPT_IN_BED.clone() }
    pub fn hero_of_the_village() -> ResourceLocation { HERO_OF_THE_VILLAGE.clone() }
    pub fn voluntary_exile() -> ResourceLocation { VOLUNTARY_EXILE.clone() }
    pub fn slide_down_block() -> ResourceLocation { SLIDE_DOWN_BLOCK.clone() }
    pub fn placed_block() -> ResourceLocation { PLACED_BLOCK.clone() }
    pub fn any_block_use() -> ResourceLocation { ANY_BLOCK_USE.clone() }
    pub fn default_block_use() -> ResourceLocation { DEFAULT_BLOCK_USE.clone() }
    pub fn inventory_changed() -> ResourceLocation { INVENTORY_CHANGED.clone() }
    pub fn consume_item() -> ResourceLocation { CONSUME_ITEM.clone() }
    pub fn using_item() -> ResourceLocation { USING_ITEM.clone() }
    pub fn item_used_on_block() -> ResourceLocation { ITEM_USED_ON_BLOCK.clone() }
    pub fn filled_bucket() -> ResourceLocation { FILLED_BUCKET.clone() }
    pub fn enchanted_item() -> ResourceLocation { ENCHANTED_ITEM.clone() }
    pub fn item_durability_changed() -> ResourceLocation { ITEM_DURABILITY_CHANGED.clone() }
    pub fn thrown_item_picked_up_by_entity() -> ResourceLocation { THROWN_ITEM_PICKED_UP_BY_ENTITY.clone() }
    pub fn thrown_item_picked_up_by_player() -> ResourceLocation { THROWN_ITEM_PICKED_UP_BY_PLAYER.clone() }
    pub fn recipe_unlocked() -> ResourceLocation { RECIPE_UNLOCKED.clone() }
    pub fn recipe_crafted() -> ResourceLocation { RECIPE_CRAFTED.clone() }
    pub fn crafter_recipe_crafted() -> ResourceLocation { CRAFTER_RECIPE_CRAFTED.clone() }
    pub fn player_hurt_entity() -> ResourceLocation { PLAYER_HURT_ENTITY.clone() }
    pub fn entity_hurt_player() -> ResourceLocation { ENTITY_HURT_PLAYER.clone() }
    pub fn used_totem() -> ResourceLocation { USED_TOTEM.clone() }
    pub fn shot_crossbow() -> ResourceLocation { SHOT_CROSSBOW.clone() }
    pub fn killed_by_crossbow() -> ResourceLocation { KILLED_BY_CROSSBOW.clone() }
    pub fn target_hit() -> ResourceLocation { TARGET_HIT.clone() }
    pub fn channeled_lightning() -> ResourceLocation { CHANNELED_LIGHTNING.clone() }
    pub fn lightning_strike() -> ResourceLocation { LIGHTNING_STRIKE.clone() }
    pub fn spear_mobs() -> ResourceLocation { SPEAR_MOBS.clone() }
    pub fn changed_dimension() -> ResourceLocation { CHANGED_DIMENSION.clone() }
    pub fn nether_travel() -> ResourceLocation { NETHER_TRAVEL.clone() }
    pub fn fall_from_height() -> ResourceLocation { FALL_FROM_HEIGHT.clone() }
    pub fn ride_entity_in_lava() -> ResourceLocation { RIDE_ENTITY_IN_LAVA.clone() }
    pub fn started_riding() -> ResourceLocation { STARTED_RIDING.clone() }
    pub fn levitation() -> ResourceLocation { LEVITATION.clone() }
    pub fn fall_after_explosion() -> ResourceLocation { FALL_AFTER_EXPLOSION.clone() }
    pub fn avoid_vibration() -> ResourceLocation { AVOID_VIBRATION.clone() }
    pub fn player_interacted_with_entity() -> ResourceLocation { PLAYER_INTERACTED_WITH_ENTITY.clone() }
    pub fn bred_animals() -> ResourceLocation { BRED_ANIMALS.clone() }
    pub fn tame_animal() -> ResourceLocation { TAME_ANIMAL.clone() }
    pub fn summoned_entity() -> ResourceLocation { SUMMONED_ENTITY.clone() }
    pub fn cured_zombie_villager() -> ResourceLocation { CURED_ZOMBIE_VILLAGER.clone() }
    pub fn villager_trade() -> ResourceLocation { VILLAGER_TRADE.clone() }
    pub fn fishing_rod_hooked() -> ResourceLocation { FISHING_ROD_HOOKED.clone() }
    pub fn allay_drop_item_on_block() -> ResourceLocation { ALLAY_DROP_ITEM_ON_BLOCK.clone() }
    pub fn brewed_potion() -> ResourceLocation { BREWED_POTION.clone() }
    pub fn effects_changed() -> ResourceLocation { EFFECTS_CHANGED.clone() }
    pub fn construct_beacon() -> ResourceLocation { CONSTRUCT_BEACON.clone() }
    pub fn used_ender_eye() -> ResourceLocation { USED_ENDER_EYE.clone() }
    pub fn player_generates_container_loot() -> ResourceLocation { PLAYER_GENERATES_CONTAINER_LOOT.clone() }
    pub fn bee_nest_destroyed() -> ResourceLocation { BEE_NEST_DESTROYED.clone() }
}
