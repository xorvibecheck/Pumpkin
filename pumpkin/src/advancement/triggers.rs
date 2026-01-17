#![allow(clippy::collapsible_if)]

use std::collections::HashSet;
use std::sync::Arc;

use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_world::advancement::triggers;

use crate::entity::player::Player;
use crate::server::Server;

#[derive(Debug, Clone)]
pub struct TriggerContext {
    pub trigger: ResourceLocation,
    pub data: std::collections::HashMap<String, TriggerValue>,
}

#[derive(Debug, Clone)]
pub enum TriggerValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    ResourceLocation(ResourceLocation),
    StringSet(HashSet<String>),
    ItemStack { item: String, count: i32 },
}

impl TriggerContext {
    #[must_use]
    pub fn new(trigger: ResourceLocation) -> Self {
        Self {
            trigger,
            data: std::collections::HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_string(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), TriggerValue::String(value.into()));
        self
    }

    #[must_use]
    pub fn with_int(mut self, key: impl Into<String>, value: i64) -> Self {
        self.data.insert(key.into(), TriggerValue::Int(value));
        self
    }

    #[must_use]
    pub fn with_resource(mut self, key: impl Into<String>, value: ResourceLocation) -> Self {
        self.data.insert(key.into(), TriggerValue::ResourceLocation(value));
        self
    }

    #[must_use]
    pub fn with_bool(mut self, key: impl Into<String>, value: bool) -> Self {
        self.data.insert(key.into(), TriggerValue::Bool(value));
        self
    }
}

pub struct AdvancementTriggers;

impl AdvancementTriggers {
    pub async fn trigger(player: &Arc<Player>, server: &Server, context: TriggerContext) {
        let registry = server.advancement_registry.read().await;
        
        for (advancement_id, advancement_entry) in registry.iter() {
            let advancement = &advancement_entry.advancement;
            
            for (criterion_name, criterion) in &advancement.criteria {
                if criterion.trigger == context.trigger {
                    let conditions_met = Self::check_conditions(criterion, &context);
                    
                    if conditions_met {
                        let mut tracker = player.advancement_tracker.lock().await;
                        let was_granted = tracker.grant_criterion(
                            advancement_id,
                            criterion_name,
                            &advancement.requirements,
                        );
                        
                        if was_granted {
                            let is_done = tracker.is_completed(advancement_id);
                            drop(tracker);
                            
                            if is_done {
                                Self::grant_rewards(player, server, advancement_entry).await;
                                Self::announce_completion(player, server, advancement_entry).await;
                            }
                            
                            Self::send_update(player, server).await;
                        }
                    }
                }
            }
        }
    }

    fn check_conditions(
        criterion: &pumpkin_world::advancement::AdvancementCriterion,
        context: &TriggerContext,
    ) -> bool {
        let conditions = match &criterion.conditions {
            Some(c) => c,
            None => return true,
        };

        // Check item conditions (for inventory_changed, consume_item, filled_bucket, etc.)
        if let Some(items_cond) = conditions.get_items() {
            if !Self::check_item_condition(items_cond, context) {
                return false;
            }
        }

        // Check entity conditions (for player_killed_entity, bred_animals, tame_animal, etc.)
        if let Some(entity_cond) = conditions.get_entity() {
            if !Self::check_entity_condition(entity_cond, context) {
                return false;
            }
        }

        // Check dimension conditions (for changed_dimension)
        if let Some(from) = conditions.get_from_dimension() {
            if let Some(TriggerValue::ResourceLocation(ctx_from)) = context.data.get("from") {
                if ctx_from.path.as_str() != from && format!("minecraft:{}", ctx_from.path) != from {
                    return false;
                }
            }
        }
        if let Some(to) = conditions.get_to_dimension() {
            if let Some(TriggerValue::ResourceLocation(ctx_to)) = context.data.get("to") {
                if ctx_to.path.as_str() != to && format!("minecraft:{}", ctx_to.path) != to {
                    return false;
                }
            }
        }

        // Check recipe_id for recipe_crafted
        if let Some(recipe_id) = conditions.get_recipe_id() {
            if let Some(TriggerValue::ResourceLocation(ctx_recipe)) = context.data.get("recipe_id") {
                let full_id = format!("{}:{}", ctx_recipe.namespace, ctx_recipe.path);
                if full_id != recipe_id && ctx_recipe.path.as_str() != recipe_id {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check effects for effects_changed
        if let Some(effects_cond) = conditions.get_effects() {
            if !Self::check_effects_condition(effects_cond, context) {
                return false;
            }
        }

        // Check beacon level for construct_beacon
        if let Some(min_level) = conditions.get_level() {
            if let Some(TriggerValue::Int(level)) = context.data.get("level") {
                if *level < min_level {
                    return false;
                }
            }
        }

        // Check block conditions
        if let Some(block_cond) = conditions.get_block() {
            if !Self::check_block_condition(block_cond, context) {
                return false;
            }
        }

        // Check distance for nether_travel, levitation, fall_from_height
        if let Some(distance_cond) = conditions.get_distance() {
            if !Self::check_distance_condition(distance_cond, context) {
                return false;
            }
        }

        true
    }

    fn check_item_condition(items_cond: &serde_json::Value, context: &TriggerContext) -> bool {
        // Get the item from context
        let ctx_item = match context.data.get("item") {
            Some(TriggerValue::String(s)) => s.clone(),
            Some(TriggerValue::ResourceLocation(r)) => format!("minecraft:{}", r.path),
            Some(TriggerValue::ItemStack { item, .. }) => item.clone(),
            _ => return true, // No item in context, pass through
        };

        // Handle array of item conditions
        if let Some(arr) = items_cond.as_array() {
            for item_cond in arr {
                if Self::check_single_item_condition(item_cond, &ctx_item) {
                    return true;
                }
            }
            return false;
        }

        // Handle single item condition object
        Self::check_single_item_condition(items_cond, &ctx_item)
    }

    fn check_single_item_condition(cond: &serde_json::Value, ctx_item: &str) -> bool {
        if let Some(obj) = cond.as_object() {
            // Check "items" field (can be string or array)
            if let Some(items) = obj.get("items") {
                if let Some(item_str) = items.as_str() {
                    let normalized_ctx = if ctx_item.starts_with("minecraft:") {
                        ctx_item.to_string()
                    } else {
                        format!("minecraft:{}", ctx_item)
                    };
                    let normalized_cond = if item_str.starts_with("minecraft:") {
                        item_str.to_string()
                    } else {
                        format!("minecraft:{}", item_str)
                    };
                    return normalized_ctx == normalized_cond;
                }
                if let Some(arr) = items.as_array() {
                    for item in arr {
                        if let Some(item_str) = item.as_str() {
                            let normalized_ctx = if ctx_item.starts_with("minecraft:") {
                                ctx_item.to_string()
                            } else {
                                format!("minecraft:{}", ctx_item)
                            };
                            let normalized_cond = if item_str.starts_with("minecraft:") {
                                item_str.to_string()
                            } else {
                                format!("minecraft:{}", item_str)
                            };
                            if normalized_ctx == normalized_cond {
                                return true;
                            }
                        }
                    }
                    return false;
                }
            }
            // Check "tag" field for item tags
            if let Some(_tag) = obj.get("tag").and_then(|v| v.as_str()) {
                // For now, we'd need tag lookup - just pass through
                // TODO: Implement proper tag checking
                return true;
            }
        }
        true
    }

    fn check_entity_condition(entity_cond: &serde_json::Value, context: &TriggerContext) -> bool {
        let ctx_entity = match context.data.get("entity") {
            Some(TriggerValue::String(s)) => s.clone(),
            Some(TriggerValue::ResourceLocation(r)) => format!("minecraft:{}", r.path),
            _ => return true,
        };

        // Handle loot table style conditions array
        if let Some(arr) = entity_cond.as_array() {
            for cond in arr {
                if let Some(obj) = cond.as_object() {
                    if let Some(predicate) = obj.get("predicate") {
                        if let Some(entity_type) = predicate.get("type").and_then(|v| v.as_str()) {
                            let normalized_ctx = if ctx_entity.starts_with("minecraft:") {
                                ctx_entity.clone()
                            } else {
                                format!("minecraft:{}", ctx_entity)
                            };
                            let normalized_cond = if entity_type.starts_with("minecraft:") {
                                entity_type.to_string()
                            } else {
                                format!("minecraft:{}", entity_type)
                            };
                            if normalized_ctx == normalized_cond {
                                return true;
                            }
                        }
                    }
                }
            }
            return false;
        }

        // Handle direct type check
        if let Some(obj) = entity_cond.as_object() {
            if let Some(entity_type) = obj.get("type").and_then(|v| v.as_str()) {
                let normalized_ctx = if ctx_entity.starts_with("minecraft:") {
                    ctx_entity.clone()
                } else {
                    format!("minecraft:{}", ctx_entity)
                };
                let normalized_cond = if entity_type.starts_with("minecraft:") {
                    entity_type.to_string()
                } else {
                    format!("minecraft:{}", entity_type)
                };
                return normalized_ctx == normalized_cond;
            }
        }

        true
    }

    fn check_effects_condition(effects_cond: &serde_json::Value, context: &TriggerContext) -> bool {
        if let Some(TriggerValue::StringSet(active_effects)) = context.data.get("effects") {
            if let Some(obj) = effects_cond.as_object() {
                // All specified effects must be present
                for (effect_id, _) in obj {
                    let normalized = if effect_id.starts_with("minecraft:") {
                        effect_id.clone()
                    } else {
                        format!("minecraft:{}", effect_id)
                    };
                    if !active_effects.contains(&normalized) && !active_effects.contains(effect_id) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn check_block_condition(block_cond: &serde_json::Value, context: &TriggerContext) -> bool {
        let ctx_block = match context.data.get("block") {
            Some(TriggerValue::String(s)) => s.clone(),
            Some(TriggerValue::ResourceLocation(r)) => format!("minecraft:{}", r.path),
            _ => return true,
        };

        if let Some(obj) = block_cond.as_object() {
            if let Some(blocks) = obj.get("blocks") {
                if let Some(block_str) = blocks.as_str() {
                    let normalized_ctx = if ctx_block.starts_with("minecraft:") {
                        ctx_block.clone()
                    } else {
                        format!("minecraft:{}", ctx_block)
                    };
                    let normalized_cond = if block_str.starts_with("minecraft:") {
                        block_str.to_string()
                    } else {
                        format!("minecraft:{}", block_str)
                    };
                    return normalized_ctx == normalized_cond;
                }
            }
        }
        true
    }

    fn check_distance_condition(distance_cond: &serde_json::Value, context: &TriggerContext) -> bool {
        if let Some(obj) = distance_cond.as_object() {
            if let Some(TriggerValue::Float(distance)) = context.data.get("distance") {
                // Check horizontal distance
                if let Some(horizontal) = obj.get("horizontal") {
                    if let Some(h_obj) = horizontal.as_object() {
                        if let Some(min) = h_obj.get("min").and_then(|v| v.as_f64()) {
                            if *distance < min {
                                return false;
                            }
                        }
                    }
                }
                // Check absolute distance
                if let Some(absolute) = obj.get("absolute") {
                    if let Some(a_obj) = absolute.as_object() {
                        if let Some(min) = a_obj.get("min").and_then(|v| v.as_f64()) {
                            if *distance < min {
                                return false;
                            }
                        }
                    }
                }
                // Check vertical distance
                if let Some(TriggerValue::Float(v_distance)) = context.data.get("vertical_distance") {
                    if let Some(vertical) = obj.get("y") {
                        if let Some(v_obj) = vertical.as_object() {
                            if let Some(min) = v_obj.get("min").and_then(|v| v.as_f64()) {
                                if *v_distance < min {
                                    return false;
                                }
                            }
                        }
                    }
                }
            }
        }
        true
    }

    async fn grant_rewards(
        player: &Arc<Player>,
        _server: &Server,
        advancement: &Arc<pumpkin_world::advancement::AdvancementEntry>,
    ) {
        let rewards = &advancement.advancement.rewards;
        
        if rewards.experience > 0 {
            player.add_experience_points(rewards.experience).await;
        }
    }

    async fn announce_completion(
        player: &Arc<Player>,
        _server: &Server,
        advancement: &Arc<pumpkin_world::advancement::AdvancementEntry>,
    ) {
        if let Some(ref display) = advancement.advancement.display {
            if display.announce_to_chat {
                let frame = display.frame;
                let key = match frame {
                    pumpkin_world::advancement::AdvancementFrame::Task => "chat.type.advancement.task",
                    pumpkin_world::advancement::AdvancementFrame::Goal => "chat.type.advancement.goal",
                    pumpkin_world::advancement::AdvancementFrame::Challenge => "chat.type.advancement.challenge",
                };
                
                let player_name = player.gameprofile.name.clone();
                let message = pumpkin_util::text::TextComponent::translate(
                    key,
                    [
                        pumpkin_util::text::TextComponent::text(player_name.clone()),
                        display.title.clone(),
                    ],
                );
                
                let sender = pumpkin_util::text::TextComponent::text(player_name);
                player.world().broadcast_message(&message, &sender, 0, None).await;
            }
        }
    }

    pub async fn send_update(player: &Arc<Player>, server: &Server) {
        let tracker = player.advancement_tracker.lock().await;
        
        if !tracker.has_pending_updates() {
            return;
        }
        
        let registry = server.advancement_registry.read().await;
        let reset = tracker.needs_reset();
        
        let advancements: Vec<pumpkin_protocol::java::client::play::AdvancementMapping> = if reset {
            registry
                .iter()
                .filter_map(|(id, entry)| {
                    entry.advancement.display.as_ref().map(|display| {
                        Self::create_advancement_mapping(id, entry, display)
                    })
                })
                .collect()
        } else {
            Vec::new()
        };
        
        // On reset, send progress for all advancements that player has interacted with
        let progress: Vec<pumpkin_protocol::java::client::play::AdvancementProgressMapping> = 
            tracker.to_progress_mappings();
        
        drop(tracker);
        drop(registry);
        
        log::debug!("Sending advancement update: reset={}, advancements={}, progress={}", 
            reset, advancements.len(), progress.len());
        
        let packet = pumpkin_protocol::java::client::play::CUpdateAdvancements::new(
            reset,
            &advancements,
            &[],
            &progress,
            !reset, // Don't show toast on initial reset, show on subsequent updates
        );
        
        player.client.enqueue_packet(&packet).await;
        
        let mut tracker = player.advancement_tracker.lock().await;
        tracker.clear_dirty();
    }

    fn create_advancement_mapping(
        id: &ResourceLocation,
        entry: &Arc<pumpkin_world::advancement::AdvancementEntry>,
        display: &pumpkin_world::advancement::AdvancementDisplay,
    ) -> pumpkin_protocol::java::client::play::AdvancementMapping {
        let advancement = &entry.advancement;
        
        let mut flags = 0i32;
        if display.background.is_some() {
            flags |= 0x01;
        }
        if display.show_toast {
            flags |= 0x02;
        }
        if display.hidden {
            flags |= 0x04;
        }
        
        let item_key = display.icon.item.path.as_str();
        let item_id = pumpkin_data::item::Item::from_registry_key(item_key)
            .map(|i| i.id as i32)
            .unwrap_or(1);
        
        let protocol_display = pumpkin_protocol::java::client::play::AdvancementDisplay {
            title: display.title.clone(),
            description: display.description.clone(),
            icon: pumpkin_protocol::java::client::play::AdvancementIcon::item(item_id),
            frame: pumpkin_protocol::java::client::play::AdvancementFrame::from_ordinal(display.frame.ordinal())
                .unwrap_or(pumpkin_protocol::java::client::play::AdvancementFrame::Task),
            flags,
            background: display.background.clone(),
            x: display.x,
            y: display.y,
        };
        
        let protocol_advancement = pumpkin_protocol::java::client::play::Advancement {
            parent: advancement.parent.clone(),
            display: Some(protocol_display),
            requirements: advancement.requirements.requirements.clone(),
            sends_telemetry_event: advancement.sends_telemetry_event,
        };
        
        pumpkin_protocol::java::client::play::AdvancementMapping::new(
            id.clone(),
            protocol_advancement,
        )
    }

    pub async fn trigger_inventory_changed(player: &Arc<Player>, server: &Server) {
        let context = TriggerContext::new(triggers::inventory_changed());
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_location(player: &Arc<Player>, server: &Server) {
        let context = TriggerContext::new(triggers::location());
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_changed_dimension(
        player: &Arc<Player>,
        server: &Server,
        from: ResourceLocation,
        to: ResourceLocation,
    ) {
        let context = TriggerContext::new(triggers::changed_dimension())
            .with_resource("from", from)
            .with_resource("to", to);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_player_killed_entity(
        player: &Arc<Player>,
        server: &Server,
        entity_type: ResourceLocation,
    ) {
        let context = TriggerContext::new(triggers::player_killed_entity())
            .with_resource("entity", entity_type);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_consume_item(
        player: &Arc<Player>,
        server: &Server,
        item: ResourceLocation,
    ) {
        let context = TriggerContext::new(triggers::consume_item())
            .with_resource("item", item);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_recipe_crafted(
        player: &Arc<Player>,
        server: &Server,
        recipe: ResourceLocation,
    ) {
        let context = TriggerContext::new(triggers::recipe_crafted())
            .with_resource("recipe_id", recipe);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_tick(player: &Arc<Player>, server: &Server) {
        let context = TriggerContext::new(triggers::tick());
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_bred_animals(
        player: &Arc<Player>,
        server: &Server,
        entity_type: ResourceLocation,
    ) {
        let context = TriggerContext::new(triggers::bred_animals())
            .with_resource("entity", entity_type);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_effects_changed(player: &Arc<Player>, server: &Server, effects: HashSet<String>) {
        let context = TriggerContext::new(triggers::effects_changed())
            .with_effects(effects);
        Self::trigger(player, server, context).await;
    }

    // ===== Additional Trigger Methods =====

    pub async fn trigger_enter_block(player: &Arc<Player>, server: &Server, block: ResourceLocation) {
        let context = TriggerContext::new(triggers::enter_block())
            .with_resource("block", block);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_slept_in_bed(player: &Arc<Player>, server: &Server) {
        let context = TriggerContext::new(triggers::slept_in_bed());
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_hero_of_the_village(player: &Arc<Player>, server: &Server) {
        let context = TriggerContext::new(triggers::hero_of_the_village());
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_voluntary_exile(player: &Arc<Player>, server: &Server) {
        let context = TriggerContext::new(triggers::voluntary_exile());
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_villager_trade(player: &Arc<Player>, server: &Server, villager_type: Option<ResourceLocation>) {
        let mut context = TriggerContext::new(triggers::villager_trade());
        if let Some(vt) = villager_type {
            context = context.with_resource("villager", vt);
        }
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_tame_animal(player: &Arc<Player>, server: &Server, entity_type: ResourceLocation) {
        let context = TriggerContext::new(triggers::tame_animal())
            .with_resource("entity", entity_type);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_summoned_entity(player: &Arc<Player>, server: &Server, entity_type: ResourceLocation) {
        let context = TriggerContext::new(triggers::summoned_entity())
            .with_resource("entity", entity_type);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_player_hurt_entity(player: &Arc<Player>, server: &Server, entity_type: ResourceLocation) {
        let context = TriggerContext::new(triggers::player_hurt_entity())
            .with_resource("entity", entity_type);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_entity_hurt_player(player: &Arc<Player>, server: &Server, entity_type: ResourceLocation) {
        let context = TriggerContext::new(triggers::entity_hurt_player())
            .with_resource("entity", entity_type);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_item_used_on_block(player: &Arc<Player>, server: &Server, item: ResourceLocation, block: ResourceLocation) {
        let context = TriggerContext::new(triggers::item_used_on_block())
            .with_resource("item", item)
            .with_resource("block", block);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_filled_bucket(player: &Arc<Player>, server: &Server, item: ResourceLocation) {
        let context = TriggerContext::new(triggers::filled_bucket())
            .with_resource("item", item);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_fishing_rod_hooked(player: &Arc<Player>, server: &Server, item: Option<ResourceLocation>) {
        let mut context = TriggerContext::new(triggers::fishing_rod_hooked());
        if let Some(i) = item {
            context = context.with_resource("item", i);
        }
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_enchanted_item(player: &Arc<Player>, server: &Server, item: ResourceLocation) {
        let context = TriggerContext::new(triggers::enchanted_item())
            .with_resource("item", item);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_brewed_potion(player: &Arc<Player>, server: &Server, potion: Option<String>) {
        let mut context = TriggerContext::new(triggers::brewed_potion());
        if let Some(p) = potion {
            context = context.with_string("potion", p);
        }
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_construct_beacon(player: &Arc<Player>, server: &Server, level: i32) {
        let context = TriggerContext::new(triggers::construct_beacon())
            .with_int("level", level as i64);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_used_totem(player: &Arc<Player>, server: &Server) {
        let context = TriggerContext::new(triggers::used_totem());
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_nether_travel(player: &Arc<Player>, server: &Server, distance: f64) {
        let context = TriggerContext::new(triggers::nether_travel())
            .with_float("distance", distance);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_levitation(player: &Arc<Player>, server: &Server, distance: f64) {
        let context = TriggerContext::new(triggers::levitation())
            .with_float("vertical_distance", distance);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_fall_from_height(player: &Arc<Player>, server: &Server, distance: f64) {
        let context = TriggerContext::new(triggers::fall_from_height())
            .with_float("distance", distance);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_started_riding(player: &Arc<Player>, server: &Server, entity_type: ResourceLocation) {
        let context = TriggerContext::new(triggers::started_riding())
            .with_resource("entity", entity_type);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_shot_crossbow(player: &Arc<Player>, server: &Server, item: ResourceLocation) {
        let context = TriggerContext::new(triggers::shot_crossbow())
            .with_resource("item", item);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_killed_by_crossbow(player: &Arc<Player>, server: &Server, unique_kills: i32) {
        let context = TriggerContext::new(triggers::killed_by_crossbow())
            .with_int("unique_entity_types", unique_kills as i64);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_target_hit(player: &Arc<Player>, server: &Server, signal_strength: i32) {
        let context = TriggerContext::new(triggers::target_hit())
            .with_int("signal_strength", signal_strength as i64);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_channeled_lightning(player: &Arc<Player>, server: &Server, victims: Vec<ResourceLocation>) {
        let set: HashSet<String> = victims.iter().map(|v| format!("{}:{}", v.namespace, v.path)).collect();
        let context = TriggerContext::new(triggers::channeled_lightning())
            .with_entity_set(set);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_lightning_strike(player: &Arc<Player>, server: &Server) {
        let context = TriggerContext::new(triggers::lightning_strike());
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_using_item(player: &Arc<Player>, server: &Server, item: ResourceLocation) {
        let context = TriggerContext::new(triggers::using_item())
            .with_resource("item", item);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_thrown_item_picked_up_by_entity(player: &Arc<Player>, server: &Server, item: ResourceLocation, entity: ResourceLocation) {
        let context = TriggerContext::new(triggers::thrown_item_picked_up_by_entity())
            .with_resource("item", item)
            .with_resource("entity", entity);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_player_interacted_with_entity(player: &Arc<Player>, server: &Server, item: ResourceLocation, entity: ResourceLocation) {
        let context = TriggerContext::new(triggers::player_interacted_with_entity())
            .with_resource("item", item)
            .with_resource("entity", entity);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_bee_nest_destroyed(player: &Arc<Player>, server: &Server, block: ResourceLocation, bees_inside: i32) {
        let context = TriggerContext::new(triggers::bee_nest_destroyed())
            .with_resource("block", block)
            .with_int("num_bees_inside", bees_inside as i64);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_cured_zombie_villager(player: &Arc<Player>, server: &Server) {
        let context = TriggerContext::new(triggers::cured_zombie_villager());
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_slide_down_block(player: &Arc<Player>, server: &Server, block: ResourceLocation) {
        let context = TriggerContext::new(triggers::slide_down_block())
            .with_resource("block", block);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_fall_after_explosion(player: &Arc<Player>, server: &Server, distance: f64) {
        let context = TriggerContext::new(triggers::fall_after_explosion())
            .with_float("distance", distance);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_avoid_vibration(player: &Arc<Player>, server: &Server) {
        let context = TriggerContext::new(triggers::avoid_vibration());
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_allay_drop_item_on_block(player: &Arc<Player>, server: &Server, item: ResourceLocation) {
        let context = TriggerContext::new(triggers::allay_drop_item_on_block())
            .with_resource("item", item);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_kill_mob_near_sculk_catalyst(player: &Arc<Player>, server: &Server, entity_type: ResourceLocation) {
        let context = TriggerContext::new(triggers::kill_mob_near_sculk_catalyst())
            .with_resource("entity", entity_type);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_placed_block(player: &Arc<Player>, server: &Server, block: ResourceLocation) {
        let context = TriggerContext::new(triggers::placed_block())
            .with_resource("block", block);
        Self::trigger(player, server, context).await;
    }

    pub async fn trigger_used_ender_eye(player: &Arc<Player>, server: &Server, distance: f64) {
        let context = TriggerContext::new(triggers::used_ender_eye())
            .with_float("distance", distance);
        Self::trigger(player, server, context).await;
    }

    pub async fn send_initial_advancements(player: &Arc<Player>, server: &Server) {
        let mut tracker = player.advancement_tracker.lock().await;
        tracker.mark_needs_reset();
        drop(tracker);
        Self::send_update(player, server).await;
    }
}

impl TriggerContext {
    #[must_use]
    pub fn with_float(mut self, key: impl Into<String>, value: f64) -> Self {
        self.data.insert(key.into(), TriggerValue::Float(value));
        self
    }

    #[must_use]
    pub fn with_effects(mut self, effects: HashSet<String>) -> Self {
        self.data.insert("effects".to_string(), TriggerValue::StringSet(effects));
        self
    }

    #[must_use]
    pub fn with_entity_set(mut self, entities: HashSet<String>) -> Self {
        self.data.insert("entities".to_string(), TriggerValue::StringSet(entities));
        self
    }

    #[must_use]
    pub fn with_item_stack(mut self, item: String, count: i32) -> Self {
        self.data.insert("item".to_string(), TriggerValue::ItemStack { item, count });
        self
    }
}
