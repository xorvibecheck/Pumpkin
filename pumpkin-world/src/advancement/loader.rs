//! Advancement loader for loading vanilla advancements from JSON data.

use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;
use serde::Deserialize;
use std::collections::HashMap;

use super::{
    Advancement, AdvancementCriterion, AdvancementDisplay, AdvancementEntry, AdvancementFrame,
    AdvancementIcon, AdvancementRequirements, AdvancementRewards, CriterionConditions,
};

/// Raw JSON structure for advancement files
#[derive(Debug, Clone, Deserialize)]
pub struct RawAdvancement {
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub display: Option<RawDisplay>,
    #[serde(default)]
    pub criteria: HashMap<String, RawCriterion>,
    #[serde(default)]
    pub requirements: Option<Vec<Vec<String>>>,
    #[serde(default)]
    pub rewards: Option<RawRewards>,
    #[serde(default)]
    pub sends_telemetry_event: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawDisplay {
    pub icon: RawIcon,
    pub title: serde_json::Value,
    pub description: serde_json::Value,
    #[serde(default)]
    pub frame: Option<String>,
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default = "default_true")]
    pub show_toast: bool,
    #[serde(default = "default_true")]
    pub announce_to_chat: bool,
    #[serde(default)]
    pub hidden: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawIcon {
    pub id: String,
    #[serde(default)]
    pub count: Option<i32>,
    #[serde(default)]
    pub components: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawCriterion {
    pub trigger: String,
    #[serde(default)]
    pub conditions: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawRewards {
    #[serde(default)]
    pub experience: i32,
    #[serde(default)]
    pub recipes: Option<Vec<String>>,
    #[serde(default)]
    pub loot: Option<Vec<String>>,
    #[serde(default)]
    pub function: Option<String>,
}

/// Converts a raw advancement JSON into an AdvancementEntry
pub fn parse_advancement(id: &str, raw: RawAdvancement) -> AdvancementEntry {
    // Advancement IDs should be minecraft:category/name format
    let adv_id = if id.contains(':') {
        ResourceLocation::from(id)
    } else {
        ResourceLocation::vanilla(id)
    };
    
    // Parse parent - parents in JSON are already in minecraft:category/name format
    let parent = raw.parent.map(|p| {
        if p.contains(':') {
            ResourceLocation::from(&p)
        } else {
            ResourceLocation::vanilla(&p)
        }
    });
    
    // Parse display
    let display = raw.display.map(parse_display);
    
    // Get criteria keys for requirements before consuming criteria
    let criteria_keys: Vec<String> = raw.criteria.keys().cloned().collect();
    
    // Parse criteria
    let criteria: HashMap<String, AdvancementCriterion> = raw.criteria.into_iter()
        .map(|(name, crit)| {
            let trigger = parse_resource_location(&crit.trigger);
            let conditions = crit.conditions.map(parse_conditions);
            (name, AdvancementCriterion {
                trigger,
                conditions,
            })
        })
        .collect();
    
    // Parse requirements
    let requirements = if let Some(reqs) = raw.requirements {
        AdvancementRequirements::new(reqs)
    } else {
        AdvancementRequirements::all_of(criteria_keys)
    };
    
    // Parse rewards
    let rewards = raw.rewards.map(|r| AdvancementRewards {
        experience: r.experience,
        recipes: r.recipes.unwrap_or_default().into_iter()
            .map(|s| parse_resource_location(&s))
            .collect(),
        loot: r.loot.and_then(|v| v.first().map(|s| parse_resource_location(s))),
        function: r.function.map(|s| parse_resource_location(&s)),
    }).unwrap_or_default();
    
    let advancement = Advancement {
        parent,
        display,
        rewards,
        criteria,
        requirements,
        sends_telemetry_event: raw.sends_telemetry_event,
    };
    
    AdvancementEntry::new(adv_id, advancement)
}

fn parse_resource_location(s: &str) -> ResourceLocation {
    ResourceLocation::from(s)
}

fn parse_display(raw: RawDisplay) -> AdvancementDisplay {
    let icon = AdvancementIcon {
        item: parse_resource_location(&raw.icon.id),
        count: raw.icon.count.unwrap_or(1),
        components: raw.icon.components,
    };
    
    let title = parse_text_component(raw.title);
    let description = parse_text_component(raw.description);
    
    let frame = match raw.frame.as_deref() {
        Some("goal") => AdvancementFrame::Goal,
        Some("challenge") => AdvancementFrame::Challenge,
        _ => AdvancementFrame::Task,
    };
    
    let background = raw.background.map(|b| parse_resource_location(&b));
    
    AdvancementDisplay {
        icon,
        title,
        description,
        frame,
        background,
        show_toast: raw.show_toast,
        announce_to_chat: raw.announce_to_chat,
        hidden: raw.hidden,
        x: 0.0,
        y: 0.0,
    }
}

fn parse_text_component(value: serde_json::Value) -> TextComponent {
    if let Some(s) = value.as_str() {
        TextComponent::text(s.to_string())
    } else if let Some(obj) = value.as_object() {
        if let Some(translate) = obj.get("translate").and_then(|v| v.as_str()) {
            TextComponent::translate(translate.to_string(), Vec::<TextComponent>::new())
        } else if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
            TextComponent::text(text.to_string())
        } else {
            TextComponent::text(value.to_string())
        }
    } else {
        TextComponent::text(value.to_string())
    }
}

fn parse_conditions(value: serde_json::Value) -> CriterionConditions {
    if let Some(obj) = value.as_object() {
        let player = obj.get("player").cloned();
        let other: HashMap<String, serde_json::Value> = obj.iter()
            .filter(|(k, _)| *k != "player")
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        CriterionConditions { player, other }
    } else {
        CriterionConditions::empty()
    }
}

/// Load a single advancement from JSON string
pub fn load_advancement_from_json(id: &str, json: &str) -> Result<AdvancementEntry, serde_json::Error> {
    let raw: RawAdvancement = serde_json::from_str(json)?;
    Ok(parse_advancement(id, raw))
}

/// All vanilla advancement definitions embedded at compile time
pub fn get_vanilla_advancements() -> Vec<AdvancementEntry> {
    let mut advancements = Vec::with_capacity(125);
    
    log::info!("Loading vanilla advancements from embedded JSON...");
    log::info!("Story advancements count: {}", STORY_ADVANCEMENTS.len());
    log::info!("Nether advancements count: {}", NETHER_ADVANCEMENTS.len());
    log::info!("End advancements count: {}", END_ADVANCEMENTS.len());
    log::info!("Adventure advancements count: {}", ADVENTURE_ADVANCEMENTS.len());
    log::info!("Husbandry advancements count: {}", HUSBANDRY_ADVANCEMENTS.len());
    
    // Story advancements
    load_category(&mut advancements, "story", STORY_ADVANCEMENTS);
    // Nether advancements  
    load_category(&mut advancements, "nether", NETHER_ADVANCEMENTS);
    // End advancements
    load_category(&mut advancements, "end", END_ADVANCEMENTS);
    // Adventure advancements
    load_category(&mut advancements, "adventure", ADVENTURE_ADVANCEMENTS);
    // Husbandry advancements
    load_category(&mut advancements, "husbandry", HUSBANDRY_ADVANCEMENTS);
    
    log::info!("Total advancements loaded: {}", advancements.len());
    
    advancements
}

fn load_category(advancements: &mut Vec<AdvancementEntry>, category: &str, data: &[(&str, &str)]) {
    let mut loaded = 0;
    let mut failed = 0;
    for (name, json) in data {
        let id = format!("{}/{}", category, name);
        match load_advancement_from_json(&id, json) {
            Ok(entry) => {
                advancements.push(entry);
                loaded += 1;
            }
            Err(e) => {
                log::warn!("Failed to parse advancement {}: {}", id, e);
                failed += 1;
            }
        }
    }
    log::info!("Loaded {} advancements from {} ({} failed)", loaded, category, failed);
}

// ============================================================================
// EMBEDDED VANILLA ADVANCEMENT DATA
// ============================================================================

const STORY_ADVANCEMENTS: &[(&str, &str)] = &[
    ("root", include_str!("../../../assets/advancements/story/root.json")),
    ("mine_stone", include_str!("../../../assets/advancements/story/mine_stone.json")),
    ("upgrade_tools", include_str!("../../../assets/advancements/story/upgrade_tools.json")),
    ("smelt_iron", include_str!("../../../assets/advancements/story/smelt_iron.json")),
    ("obtain_armor", include_str!("../../../assets/advancements/story/obtain_armor.json")),
    ("lava_bucket", include_str!("../../../assets/advancements/story/lava_bucket.json")),
    ("iron_tools", include_str!("../../../assets/advancements/story/iron_tools.json")),
    ("deflect_arrow", include_str!("../../../assets/advancements/story/deflect_arrow.json")),
    ("form_obsidian", include_str!("../../../assets/advancements/story/form_obsidian.json")),
    ("mine_diamond", include_str!("../../../assets/advancements/story/mine_diamond.json")),
    ("enter_the_nether", include_str!("../../../assets/advancements/story/enter_the_nether.json")),
    ("shiny_gear", include_str!("../../../assets/advancements/story/shiny_gear.json")),
    ("enchant_item", include_str!("../../../assets/advancements/story/enchant_item.json")),
    ("cure_zombie_villager", include_str!("../../../assets/advancements/story/cure_zombie_villager.json")),
    ("follow_ender_eye", include_str!("../../../assets/advancements/story/follow_ender_eye.json")),
    ("enter_the_end", include_str!("../../../assets/advancements/story/enter_the_end.json")),
];

const NETHER_ADVANCEMENTS: &[(&str, &str)] = &[
    ("root", include_str!("../../../assets/advancements/nether/root.json")),
    ("return_to_sender", include_str!("../../../assets/advancements/nether/return_to_sender.json")),
    ("find_bastion", include_str!("../../../assets/advancements/nether/find_bastion.json")),
    ("obtain_ancient_debris", include_str!("../../../assets/advancements/nether/obtain_ancient_debris.json")),
    ("fast_travel", include_str!("../../../assets/advancements/nether/fast_travel.json")),
    ("find_fortress", include_str!("../../../assets/advancements/nether/find_fortress.json")),
    ("obtain_crying_obsidian", include_str!("../../../assets/advancements/nether/obtain_crying_obsidian.json")),
    ("distract_piglin", include_str!("../../../assets/advancements/nether/distract_piglin.json")),
    ("ride_strider", include_str!("../../../assets/advancements/nether/ride_strider.json")),
    ("uneasy_alliance", include_str!("../../../assets/advancements/nether/uneasy_alliance.json")),
    ("loot_bastion", include_str!("../../../assets/advancements/nether/loot_bastion.json")),
    ("netherite_armor", include_str!("../../../assets/advancements/nether/netherite_armor.json")),
    ("get_wither_skull", include_str!("../../../assets/advancements/nether/get_wither_skull.json")),
    ("obtain_blaze_rod", include_str!("../../../assets/advancements/nether/obtain_blaze_rod.json")),
    ("charge_respawn_anchor", include_str!("../../../assets/advancements/nether/charge_respawn_anchor.json")),
    ("ride_strider_in_overworld_lava", include_str!("../../../assets/advancements/nether/ride_strider_in_overworld_lava.json")),
    ("summon_wither", include_str!("../../../assets/advancements/nether/summon_wither.json")),
    ("brew_potion", include_str!("../../../assets/advancements/nether/brew_potion.json")),
    ("create_beacon", include_str!("../../../assets/advancements/nether/create_beacon.json")),
    ("all_potions", include_str!("../../../assets/advancements/nether/all_potions.json")),
    ("create_full_beacon", include_str!("../../../assets/advancements/nether/create_full_beacon.json")),
    ("all_effects", include_str!("../../../assets/advancements/nether/all_effects.json")),
    ("explore_nether", include_str!("../../../assets/advancements/nether/explore_nether.json")),
];

const END_ADVANCEMENTS: &[(&str, &str)] = &[
    ("root", include_str!("../../../assets/advancements/end/root.json")),
    ("kill_dragon", include_str!("../../../assets/advancements/end/kill_dragon.json")),
    ("dragon_egg", include_str!("../../../assets/advancements/end/dragon_egg.json")),
    ("enter_end_gateway", include_str!("../../../assets/advancements/end/enter_end_gateway.json")),
    ("respawn_dragon", include_str!("../../../assets/advancements/end/respawn_dragon.json")),
    ("dragon_breath", include_str!("../../../assets/advancements/end/dragon_breath.json")),
    ("find_end_city", include_str!("../../../assets/advancements/end/find_end_city.json")),
    ("elytra", include_str!("../../../assets/advancements/end/elytra.json")),
    ("levitate", include_str!("../../../assets/advancements/end/levitate.json")),
];

const ADVENTURE_ADVANCEMENTS: &[(&str, &str)] = &[
    ("root", include_str!("../../../assets/advancements/adventure/root.json")),
    ("voluntary_exile", include_str!("../../../assets/advancements/adventure/voluntary_exile.json")),
    ("spyglass_at_parrot", include_str!("../../../assets/advancements/adventure/spyglass_at_parrot.json")),
    ("kill_a_mob", include_str!("../../../assets/advancements/adventure/kill_a_mob.json")),
    ("read_power_of_chiseled_bookshelf", include_str!("../../../assets/advancements/adventure/read_power_of_chiseled_bookshelf.json")),
    ("trade", include_str!("../../../assets/advancements/adventure/trade.json")),
    ("trim_with_any_armor_pattern", include_str!("../../../assets/advancements/adventure/trim_with_any_armor_pattern.json")),
    ("honey_block_slide", include_str!("../../../assets/advancements/adventure/honey_block_slide.json")),
    ("sleep_in_bed", include_str!("../../../assets/advancements/adventure/sleep_in_bed.json")),
    ("hero_of_the_village", include_str!("../../../assets/advancements/adventure/hero_of_the_village.json")),
    ("spyglass_at_ghast", include_str!("../../../assets/advancements/adventure/spyglass_at_ghast.json")),
    ("throw_trident", include_str!("../../../assets/advancements/adventure/throw_trident.json")),
    ("kill_mob_near_sculk_catalyst", include_str!("../../../assets/advancements/adventure/kill_mob_near_sculk_catalyst.json")),
    ("shoot_arrow", include_str!("../../../assets/advancements/adventure/shoot_arrow.json")),
    ("kill_all_mobs", include_str!("../../../assets/advancements/adventure/kill_all_mobs.json")),
    ("totem_of_undying", include_str!("../../../assets/advancements/adventure/totem_of_undying.json")),
    ("summon_iron_golem", include_str!("../../../assets/advancements/adventure/summon_iron_golem.json")),
    ("trade_at_world_height", include_str!("../../../assets/advancements/adventure/trade_at_world_height.json")),
    ("trim_with_all_exclusive_armor_patterns", include_str!("../../../assets/advancements/adventure/trim_with_all_exclusive_armor_patterns.json")),
    ("spyglass_at_dragon", include_str!("../../../assets/advancements/adventure/spyglass_at_dragon.json")),
    ("very_very_frightening", include_str!("../../../assets/advancements/adventure/very_very_frightening.json")),
    ("sniper_duel", include_str!("../../../assets/advancements/adventure/sniper_duel.json")),
    ("bullseye", include_str!("../../../assets/advancements/adventure/bullseye.json")),
    ("fall_from_world_height", include_str!("../../../assets/advancements/adventure/fall_from_world_height.json")),
    ("avoid_vibration", include_str!("../../../assets/advancements/adventure/avoid_vibration.json")),
    ("two_birds_one_arrow", include_str!("../../../assets/advancements/adventure/two_birds_one_arrow.json")),
    ("whos_the_pillager_now", include_str!("../../../assets/advancements/adventure/whos_the_pillager_now.json")),
    ("arbalistic", include_str!("../../../assets/advancements/adventure/arbalistic.json")),
    ("craft_decorated_pot_using_only_sherds", include_str!("../../../assets/advancements/adventure/craft_decorated_pot_using_only_sherds.json")),
    ("adventuring_time", include_str!("../../../assets/advancements/adventure/adventuring_time.json")),
    ("play_jukebox_in_meadows", include_str!("../../../assets/advancements/adventure/play_jukebox_in_meadows.json")),
    ("walk_on_powder_snow_with_leather_boots", include_str!("../../../assets/advancements/adventure/walk_on_powder_snow_with_leather_boots.json")),
    ("lightning_rod_with_villager_no_fire", include_str!("../../../assets/advancements/adventure/lightning_rod_with_villager_no_fire.json")),
    ("salvage_sherd", include_str!("../../../assets/advancements/adventure/salvage_sherd.json")),
    ("revaulting", include_str!("../../../assets/advancements/adventure/revaulting.json")),
    ("blowback", include_str!("../../../assets/advancements/adventure/blowback.json")),
    ("who_needs_rockets", include_str!("../../../assets/advancements/adventure/who_needs_rockets.json")),
    ("minecraft_trials_edition", include_str!("../../../assets/advancements/adventure/minecraft_trials_edition.json")),
    ("ol_betsy", include_str!("../../../assets/advancements/adventure/ol_betsy.json")),
    ("overoverkill", include_str!("../../../assets/advancements/adventure/overoverkill.json")),
    ("crafters_crafting_crafters", include_str!("../../../assets/advancements/adventure/crafters_crafting_crafters.json")),
    ("lighten_up", include_str!("../../../assets/advancements/adventure/lighten_up.json")),
    ("brush_armadillo", include_str!("../../../assets/advancements/adventure/brush_armadillo.json")),
    ("heart_transplanter", include_str!("../../../assets/advancements/adventure/heart_transplanter.json")),
    ("under_lock_and_key", include_str!("../../../assets/advancements/adventure/under_lock_and_key.json")),
    ("use_lodestone", include_str!("../../../assets/advancements/adventure/use_lodestone.json")),
];

const HUSBANDRY_ADVANCEMENTS: &[(&str, &str)] = &[
    ("root", include_str!("../../../assets/advancements/husbandry/root.json")),
    ("safely_harvest_honey", include_str!("../../../assets/advancements/husbandry/safely_harvest_honey.json")),
    ("breed_an_animal", include_str!("../../../assets/advancements/husbandry/breed_an_animal.json")),
    ("allay_deliver_item_to_player", include_str!("../../../assets/advancements/husbandry/allay_deliver_item_to_player.json")),
    ("ride_a_boat_with_a_goat", include_str!("../../../assets/advancements/husbandry/ride_a_boat_with_a_goat.json")),
    ("tame_an_animal", include_str!("../../../assets/advancements/husbandry/tame_an_animal.json")),
    ("make_a_sign_glow", include_str!("../../../assets/advancements/husbandry/make_a_sign_glow.json")),
    ("fishy_business", include_str!("../../../assets/advancements/husbandry/fishy_business.json")),
    ("silk_touch_nest", include_str!("../../../assets/advancements/husbandry/silk_touch_nest.json")),
    ("wax_on", include_str!("../../../assets/advancements/husbandry/wax_on.json")),
    ("bred_all_animals", include_str!("../../../assets/advancements/husbandry/bred_all_animals.json")),
    ("allay_deliver_cake_to_note_block", include_str!("../../../assets/advancements/husbandry/allay_deliver_cake_to_note_block.json")),
    ("complete_catalogue", include_str!("../../../assets/advancements/husbandry/complete_catalogue.json")),
    ("tactical_fishing", include_str!("../../../assets/advancements/husbandry/tactical_fishing.json")),
    ("leash_all_frog_variants", include_str!("../../../assets/advancements/husbandry/leash_all_frog_variants.json")),
    ("feed_snifflet", include_str!("../../../assets/advancements/husbandry/feed_snifflet.json")),
    ("wax_off", include_str!("../../../assets/advancements/husbandry/wax_off.json")),
    ("axolotl_in_a_bucket", include_str!("../../../assets/advancements/husbandry/axolotl_in_a_bucket.json")),
    ("froglights", include_str!("../../../assets/advancements/husbandry/froglights.json")),
    ("plant_any_sniffer_seed", include_str!("../../../assets/advancements/husbandry/plant_any_sniffer_seed.json")),
    ("kill_axolotl_target", include_str!("../../../assets/advancements/husbandry/kill_axolotl_target.json")),
    ("balanced_diet", include_str!("../../../assets/advancements/husbandry/balanced_diet.json")),
    ("obtain_netherite_hoe", include_str!("../../../assets/advancements/husbandry/obtain_netherite_hoe.json")),
    ("plant_seed", include_str!("../../../assets/advancements/husbandry/plant_seed.json")),
    ("repair_wolf_armor", include_str!("../../../assets/advancements/husbandry/repair_wolf_armor.json")),
    ("whole_pack", include_str!("../../../assets/advancements/husbandry/whole_pack.json")),
    ("remove_wolf_armor", include_str!("../../../assets/advancements/husbandry/remove_wolf_armor.json")),
    ("obtain_sniffer_egg", include_str!("../../../assets/advancements/husbandry/obtain_sniffer_egg.json")),
    ("tadpole_in_a_bucket", include_str!("../../../assets/advancements/husbandry/tadpole_in_a_bucket.json")),
    ("place_dried_ghast_in_water", include_str!("../../../assets/advancements/husbandry/place_dried_ghast_in_water.json")),
];
