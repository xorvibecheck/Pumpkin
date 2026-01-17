use std::collections::HashMap;
use std::sync::Arc;

use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;
use pumpkin_world::advancement::{
    Advancement, AdvancementCriterion, AdvancementDisplay, AdvancementEntry, AdvancementFrame,
    AdvancementIcon, PlacedAdvancement, triggers,
};

#[derive(Debug, Default)]
pub struct AdvancementRegistry {
    advancements: HashMap<ResourceLocation, Arc<AdvancementEntry>>,
    roots: Vec<ResourceLocation>,
    placed: HashMap<ResourceLocation, PlacedAdvancement>,
}

impl AdvancementRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, entry: AdvancementEntry) {
        let id = entry.id.clone();
        let is_root = entry.advancement.is_root();
        if let Some(ref parent_id) = entry.advancement.parent {
            if let Some(placed_parent) = self.placed.get_mut(parent_id) {
                placed_parent.add_child(id.clone());
            }
        }
        let placed = PlacedAdvancement::new(entry.clone());
        self.placed.insert(id.clone(), placed);
        if is_root {
            self.roots.push(id.clone());
        }
        self.advancements.insert(id, Arc::new(entry));
    }

    #[must_use]
    pub fn get(&self, id: &ResourceLocation) -> Option<&Arc<AdvancementEntry>> {
        self.advancements.get(id)
    }

    #[must_use]
    pub fn roots(&self) -> &[ResourceLocation] {
        &self.roots
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ResourceLocation, &Arc<AdvancementEntry>)> {
        self.advancements.iter()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.advancements.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.advancements.is_empty()
    }

    pub fn clear(&mut self) {
        self.advancements.clear();
        self.roots.clear();
        self.placed.clear();
    }

    #[must_use]
    pub fn get_placed(&self, id: &ResourceLocation) -> Option<&PlacedAdvancement> {
        self.placed.get(id)
    }

    #[must_use]
    pub fn all_ids(&self) -> Vec<ResourceLocation> {
        self.advancements.keys().cloned().collect()
    }

    pub fn load_vanilla_advancements(&mut self) {
        // Load advancements from embedded JSON data
        let advancements = pumpkin_world::advancement::get_vanilla_advancements();
        for entry in advancements {
            self.register(entry);
        }
        log::info!("Loaded {} vanilla advancements from JSON", self.advancements.len());
    }
}

// Keep unused helper methods for potential future use
#[allow(dead_code)]
impl AdvancementRegistry {
    fn adv(&mut self, id: &str, parent: Option<&str>, icon: &str, title: &str, desc: &str, frame: AdvancementFrame, bg: Option<&str>, x: f32, y: f32, trigger: &ResourceLocation) {
        let adv_id = ResourceLocation::vanilla(id);
        let mut display = AdvancementDisplay::new(
            AdvancementIcon::simple(ResourceLocation::vanilla(icon)),
            TextComponent::text(title.to_string()),
            TextComponent::text(desc.to_string()),
            frame,
            bg.map(|b| ResourceLocation::vanilla(b)),
            bg.is_none(),
            bg.is_none(),
            false,
        );
        display.set_pos(x, y);
        let advancement = if let Some(p) = parent {
            Advancement::builder()
                .parent(ResourceLocation::vanilla(p))
                .display(display)
                .criterion("trigger", AdvancementCriterion::new(trigger.clone()))
                .build()
        } else {
            let mut criteria = HashMap::new();
            criteria.insert("trigger".to_string(), AdvancementCriterion::new(trigger.clone()));
            Advancement::root(display, criteria)
        };
        self.register(AdvancementEntry::new(adv_id, advancement));
    }

    fn adv_hidden(&mut self, id: &str, parent: &str, icon: &str, title: &str, desc: &str, frame: AdvancementFrame, x: f32, y: f32, trigger: &ResourceLocation) {
        let adv_id = ResourceLocation::vanilla(id);
        let mut display = AdvancementDisplay::new(
            AdvancementIcon::simple(ResourceLocation::vanilla(icon)),
            TextComponent::text(title.to_string()),
            TextComponent::text(desc.to_string()),
            frame,
            None,
            true,
            true,
            true,
        );
        display.set_pos(x, y);
        let advancement = Advancement::builder()
            .parent(ResourceLocation::vanilla(parent))
            .display(display)
            .criterion("trigger", AdvancementCriterion::new(trigger.clone()))
            .build();
        self.register(AdvancementEntry::new(adv_id, advancement));
    }

    fn register_story_advancements(&mut self) {
        use AdvancementFrame::*;
        let bg = Some("textures/gui/advancements/backgrounds/stone.png");
        // Root: Minecraft
        self.adv("story/root", None, "grass_block", "Minecraft", "The heart and story of the game", Task, bg, 0.0, 0.0, &triggers::INVENTORY_CHANGED);
        // Stone Age
        self.adv("story/mine_stone", Some("story/root"), "wooden_pickaxe", "Stone Age", "Mine Stone with your new Pickaxe", Task, None, 2.0, 3.0, &triggers::INVENTORY_CHANGED);
        // Getting an Upgrade
        self.adv("story/upgrade_tools", Some("story/mine_stone"), "stone_pickaxe", "Getting an Upgrade", "Construct a better Pickaxe", Task, None, 4.0, 3.0, &triggers::INVENTORY_CHANGED);
        // Acquire Hardware
        self.adv("story/smelt_iron", Some("story/upgrade_tools"), "iron_ingot", "Acquire Hardware", "Smelt an Iron Ingot", Task, None, 6.0, 3.0, &triggers::INVENTORY_CHANGED);
        // Suit Up
        self.adv("story/obtain_armor", Some("story/smelt_iron"), "iron_chestplate", "Suit Up", "Protect yourself with a piece of iron armor", Task, None, 6.0, 5.0, &triggers::INVENTORY_CHANGED);
        // Hot Stuff
        self.adv("story/lava_bucket", Some("story/smelt_iron"), "lava_bucket", "Hot Stuff", "Fill a Bucket with lava", Task, None, 7.0, 4.0, &triggers::INVENTORY_CHANGED);
        // Isn't It Iron Pick
        self.adv("story/iron_tools", Some("story/smelt_iron"), "iron_pickaxe", "Isn't It Iron Pick", "Upgrade your Pickaxe", Task, None, 6.0, 1.0, &triggers::INVENTORY_CHANGED);
        // Not Today, Thank You
        self.adv("story/deflect_arrow", Some("story/obtain_armor"), "shield", "Not Today, Thank You", "Deflect a projectile with a Shield", Task, None, 6.0, 7.0, &triggers::INVENTORY_CHANGED);
        // Ice Bucket Challenge
        self.adv("story/form_obsidian", Some("story/lava_bucket"), "obsidian", "Ice Bucket Challenge", "Obtain a block of Obsidian", Task, None, 8.0, 4.0, &triggers::INVENTORY_CHANGED);
        // Diamonds!
        self.adv("story/mine_diamond", Some("story/iron_tools"), "diamond", "Diamonds!", "Acquire diamonds", Task, None, 7.0, 1.0, &triggers::INVENTORY_CHANGED);
        // We Need to Go Deeper
        self.adv("story/enter_the_nether", Some("story/form_obsidian"), "flint_and_steel", "We Need to Go Deeper", "Build, light and enter a Nether Portal", Task, None, 10.0, 4.0, &triggers::CHANGED_DIMENSION);
        // Cover Me with Diamonds
        self.adv("story/shiny_gear", Some("story/mine_diamond"), "diamond_chestplate", "Cover Me with Diamonds", "Diamond armor saves lives", Task, None, 9.0, 1.0, &triggers::INVENTORY_CHANGED);
        // Enchanter
        self.adv("story/enchant_item", Some("story/mine_diamond"), "enchanting_table", "Enchanter", "Enchant an item at an Enchanting Table", Task, None, 7.0, -1.0, &triggers::INVENTORY_CHANGED);
        // Zombie Doctor
        self.adv("story/cure_zombie_villager", Some("story/enter_the_nether"), "golden_apple", "Zombie Doctor", "Weaken and then cure a Zombie Villager", Goal, None, 12.0, 4.0, &triggers::VILLAGER_TRADE);
        // Eye Spy
        self.adv("story/follow_ender_eye", Some("story/enter_the_nether"), "ender_eye", "Eye Spy", "Follow an Eye of Ender", Task, None, 10.0, 2.0, &triggers::LOCATION);
        // The End?
        self.adv("story/enter_the_end", Some("story/follow_ender_eye"), "end_stone", "The End?", "Enter the End Portal", Task, None, 12.0, 2.0, &triggers::CHANGED_DIMENSION);
    }

    fn register_nether_advancements(&mut self) {
        use AdvancementFrame::*;
        let bg = Some("textures/gui/advancements/backgrounds/nether.png");
        // Root: Nether
        self.adv("nether/root", None, "red_nether_bricks", "Nether", "Bring summer clothes", Task, bg, 0.0, 0.0, &triggers::CHANGED_DIMENSION);
        // Return to Sender
        self.adv("nether/return_to_sender", Some("nether/root"), "fire_charge", "Return to Sender", "Destroy a Ghast with a fireball", Challenge, None, 4.0, 0.0, &triggers::PLAYER_KILLED_ENTITY);
        // Those Were the Days
        self.adv("nether/find_bastion", Some("nether/root"), "polished_blackstone_bricks", "Those Were the Days", "Enter a Bastion Remnant", Task, None, 2.0, 2.0, &triggers::LOCATION);
        // Hidden in the Depths
        self.adv("nether/obtain_ancient_debris", Some("nether/root"), "ancient_debris", "Hidden in the Depths", "Obtain Ancient Debris", Task, None, 2.0, -2.0, &triggers::INVENTORY_CHANGED);
        // Subspace Bubble
        self.adv("nether/fast_travel", Some("nether/root"), "map", "Subspace Bubble", "Use the Nether to travel 7 km in the Overworld", Challenge, None, 2.0, 0.0, &triggers::NETHER_TRAVEL);
        // A Terrible Fortress
        self.adv("nether/find_fortress", Some("nether/root"), "nether_bricks", "A Terrible Fortress", "Break your way into a Nether Fortress", Task, None, 4.0, 2.0, &triggers::LOCATION);
        // Who is Cutting Onions?
        self.adv("nether/obtain_crying_obsidian", Some("nether/root"), "crying_obsidian", "Who is Cutting Onions?", "Obtain Crying Obsidian", Task, None, 0.0, 2.0, &triggers::INVENTORY_CHANGED);
        // Oh Shiny
        self.adv("nether/distract_piglin", Some("nether/root"), "gold_ingot", "Oh Shiny", "Distract Piglins with gold", Task, None, 0.0, -2.0, &triggers::THROWN_ITEM_PICKED_UP_BY_ENTITY);
        // This Boat Has Legs
        self.adv("nether/ride_strider", Some("nether/root"), "warped_fungus_on_a_stick", "This Boat Has Legs", "Ride a Strider with a Warped Fungus on a Stick", Task, None, -2.0, 0.0, &triggers::STARTED_RIDING);
        // Uneasy Alliance
        self.adv("nether/uneasy_alliance", Some("nether/return_to_sender"), "ghast_tear", "Uneasy Alliance", "Rescue a Ghast from the Nether, bring it safely home to the Overworld... and then kill it", Challenge, None, 6.0, 0.0, &triggers::PLAYER_KILLED_ENTITY);
        // War Pigs
        self.adv("nether/loot_bastion", Some("nether/find_bastion"), "chest", "War Pigs", "Loot a chest in a Bastion Remnant", Task, None, 4.0, 4.0, &triggers::INVENTORY_CHANGED);
        // Country Lode, Take Me Home
        self.adv("nether/use_lodestone", Some("nether/obtain_ancient_debris"), "lodestone", "Country Lode, Take Me Home", "Use a Compass on a Lodestone", Task, None, 4.0, -2.0, &triggers::ITEM_USED_ON_BLOCK);
        // Cover Me in Debris
        self.adv("nether/netherite_armor", Some("nether/obtain_ancient_debris"), "netherite_chestplate", "Cover Me in Debris", "Get a full suit of Netherite armor", Challenge, None, 2.0, -4.0, &triggers::INVENTORY_CHANGED);
        // Spooky Scary Skeleton
        self.adv("nether/get_wither_skull", Some("nether/find_fortress"), "wither_skeleton_skull", "Spooky Scary Skeleton", "Obtain a Wither Skeleton's skull", Task, None, 6.0, 2.0, &triggers::INVENTORY_CHANGED);
        // Into Fire
        self.adv("nether/obtain_blaze_rod", Some("nether/find_fortress"), "blaze_rod", "Into Fire", "Relieve a Blaze of its rod", Task, None, 4.0, 6.0, &triggers::INVENTORY_CHANGED);
        // Not Quite \"Nine\" Lives
        self.adv("nether/charge_respawn_anchor", Some("nether/obtain_crying_obsidian"), "respawn_anchor", "Not Quite \"Nine\" Lives", "Charge a Respawn Anchor to the maximum", Task, None, 0.0, 4.0, &triggers::ITEM_USED_ON_BLOCK);
        // Feel the Burn (renamed from Feels like home)
        self.adv("nether/ride_strider_in_overworld_lava", Some("nether/ride_strider"), "strider_spawn_egg", "Feels Like Home", "Take a Strider for a loooong ride on a lava lake in the Overworld", Task, None, -4.0, 0.0, &triggers::STARTED_RIDING);
        // Withering Heights
        self.adv("nether/summon_wither", Some("nether/get_wither_skull"), "nether_star", "Withering Heights", "Summon the Wither", Task, None, 8.0, 2.0, &triggers::SUMMONED_ENTITY);
        // Local Brewery
        self.adv("nether/brew_potion", Some("nether/obtain_blaze_rod"), "potion", "Local Brewery", "Brew a Potion", Task, None, 4.0, 8.0, &triggers::BREWED_POTION);
        // Bring Home the Beacon
        self.adv("nether/create_beacon", Some("nether/summon_wither"), "beacon", "Bring Home the Beacon", "Construct and place a Beacon", Task, None, 10.0, 2.0, &triggers::CONSTRUCT_BEACON);
        // A Furious Cocktail
        self.adv("nether/all_potions", Some("nether/brew_potion"), "milk_bucket", "A Furious Cocktail", "Have every potion effect applied at the same time", Challenge, None, 4.0, 10.0, &triggers::EFFECTS_CHANGED);
        // Beaconator
        self.adv("nether/create_full_beacon", Some("nether/create_beacon"), "beacon", "Beaconator", "Bring a Beacon to full power", Goal, None, 12.0, 2.0, &triggers::CONSTRUCT_BEACON);
        // How Did We Get Here?
        self.adv_hidden("nether/all_effects", "nether/all_potions", "bucket", "How Did We Get Here?", "Have every effect applied at the same time", Challenge, 4.0, 12.0, &triggers::EFFECTS_CHANGED);
        // Hot Tourist Destinations
        self.adv("nether/explore_nether", Some("nether/fast_travel"), "netherite_boots", "Hot Tourist Destinations", "Explore all Nether biomes", Challenge, None, 2.0, -4.0, &triggers::LOCATION);
    }

    fn register_end_advancements(&mut self) {
        use AdvancementFrame::*;
        let bg = Some("textures/gui/advancements/backgrounds/end.png");
        // Root: The End
        self.adv("end/root", None, "end_stone", "The End", "Or the beginning?", Task, bg, 0.0, 0.0, &triggers::CHANGED_DIMENSION);
        // Free the End
        self.adv("end/kill_dragon", Some("end/root"), "dragon_head", "Free the End", "Good luck", Task, None, 4.0, 0.0, &triggers::PLAYER_KILLED_ENTITY);
        // The Next Generation
        self.adv("end/dragon_egg", Some("end/kill_dragon"), "dragon_egg", "The Next Generation", "Hold the Dragon Egg", Goal, None, 6.0, 0.0, &triggers::INVENTORY_CHANGED);
        // Remote Getaway
        self.adv("end/enter_end_gateway", Some("end/kill_dragon"), "ender_pearl", "Remote Getaway", "Escape the island", Task, None, 4.0, 2.0, &triggers::LOCATION);
        // The End... Again...
        self.adv("end/respawn_dragon", Some("end/kill_dragon"), "end_crystal", "The End... Again...", "Respawn the Ender Dragon", Goal, None, 4.0, -2.0, &triggers::SUMMONED_ENTITY);
        // You Need a Mint
        self.adv("end/dragon_breath", Some("end/kill_dragon"), "dragon_breath", "You Need a Mint", "Collect Dragon's Breath in a Glass Bottle", Goal, None, 6.0, 2.0, &triggers::INVENTORY_CHANGED);
        // The City at the End of the Game
        self.adv("end/find_end_city", Some("end/enter_end_gateway"), "purpur_block", "The City at the End of the Game", "Go on in, what could happen?", Task, None, 4.0, 4.0, &triggers::LOCATION);
        // Sky's the Limit
        self.adv("end/elytra", Some("end/find_end_city"), "elytra", "Sky's the Limit", "Find Elytra", Goal, None, 6.0, 4.0, &triggers::INVENTORY_CHANGED);
        // Great View From Up Here
        self.adv("end/levitate", Some("end/find_end_city"), "shulker_shell", "Great View From Up Here", "Levitate up 50 blocks from the attacks of a Shulker", Challenge, None, 2.0, 4.0, &triggers::LEVITATION);
    }

    fn register_adventure_advancements(&mut self) {
        use AdvancementFrame::*;
        let bg = Some("textures/gui/advancements/backgrounds/adventure.png");
        // Root: Adventure
        self.adv("adventure/root", None, "map", "Adventure", "Adventure, exploration and combat", Task, bg, 0.0, 0.0, &triggers::LOCATION);
        // Voluntary Exile
        self.adv("adventure/voluntary_exile", Some("adventure/root"), "ominous_banner", "Voluntary Exile", "Kill a raid captain. Maybe consider staying away from villages for the time being...", Task, None, 2.0, -4.0, &triggers::PLAYER_KILLED_ENTITY);
        // Is It a Bird?
        self.adv("adventure/spyglass_at_parrot", Some("adventure/root"), "spyglass", "Is It a Bird?", "Look at a Parrot through a Spyglass", Task, None, -2.0, -4.0, &triggers::USING_ITEM);
        // Monster Hunter
        self.adv("adventure/kill_a_mob", Some("adventure/root"), "iron_sword", "Monster Hunter", "Kill any hostile monster", Task, None, 2.0, 2.0, &triggers::PLAYER_KILLED_ENTITY);
        // The Power of Books
        self.adv("adventure/read_power_of_chiseled_bookshelf", Some("adventure/root"), "chiseled_bookshelf", "The Power of Books", "Read the power signal of a Chiseled Bookshelf using a Comparator", Task, None, -2.0, 2.0, &triggers::ITEM_USED_ON_BLOCK);
        // What a Deal!
        self.adv("adventure/trade", Some("adventure/root"), "emerald", "What a Deal!", "Successfully trade with a Villager", Task, None, 0.0, 2.0, &triggers::VILLAGER_TRADE);
        // Crafting a New Look
        self.adv("adventure/trim_with_any_armor_pattern", Some("adventure/root"), "dune_armor_trim_smithing_template", "Crafting a New Look", "Craft a trimmed armor at a Smithing Table", Task, None, -2.0, 0.0, &triggers::RECIPE_CRAFTED);
        // Sticky Situation
        self.adv("adventure/honey_block_slide", Some("adventure/root"), "honey_block", "Sticky Situation", "Jump into a Honey Block to break your fall", Task, None, 0.0, -2.0, &triggers::LOCATION);
        // Sweet Dreams
        self.adv("adventure/sleep_in_bed", Some("adventure/root"), "red_bed", "Sweet Dreams", "Sleep in a Bed to change your respawn point", Task, None, 2.0, 0.0, &triggers::SLEPT_IN_BED);
        // Hero of the Village
        self.adv("adventure/hero_of_the_village", Some("adventure/voluntary_exile"), "ominous_banner", "Hero of the Village", "Successfully defend a village from a raid", Challenge, None, 2.0, -6.0, &triggers::HERO_OF_THE_VILLAGE);
        // Is It a Balloon?
        self.adv("adventure/spyglass_at_ghast", Some("adventure/spyglass_at_parrot"), "spyglass", "Is It a Balloon?", "Look at a Ghast through a Spyglass", Task, None, -4.0, -4.0, &triggers::USING_ITEM);
        // A Throwaway Joke
        self.adv("adventure/throw_trident", Some("adventure/kill_a_mob"), "trident", "A Throwaway Joke", "Throw a Trident at something. Note: Throwing away your only weapon is not a good idea.", Task, None, 6.0, 2.0, &triggers::PLAYER_HURT_ENTITY);
        // It Spreads
        self.adv("adventure/kill_mob_near_sculk_catalyst", Some("adventure/kill_a_mob"), "sculk_catalyst", "It Spreads", "Kill a mob near a Sculk Catalyst", Task, None, 2.0, 4.0, &triggers::PLAYER_KILLED_ENTITY);
        // Take Aim
        self.adv("adventure/shoot_arrow", Some("adventure/kill_a_mob"), "bow", "Take Aim", "Shoot something with an Arrow", Task, None, 4.0, 0.0, &triggers::PLAYER_HURT_ENTITY);
        // Monsters Hunted
        self.adv("adventure/kill_all_mobs", Some("adventure/kill_a_mob"), "diamond_sword", "Monsters Hunted", "Kill one of every hostile monster", Challenge, None, 2.0, 6.0, &triggers::PLAYER_KILLED_ENTITY);
        // Postmortal
        self.adv("adventure/totem_of_undying", Some("adventure/kill_a_mob"), "totem_of_undying", "Postmortal", "Use a Totem of Undying to cheat death", Goal, None, 4.0, 4.0, &triggers::USED_TOTEM);
        // Hired Help
        self.adv("adventure/summon_iron_golem", Some("adventure/trade"), "carved_pumpkin", "Hired Help", "Summon an Iron Golem to help defend a village", Goal, None, 0.0, 4.0, &triggers::SUMMONED_ENTITY);
        // Star Trader
        self.adv("adventure/trade_at_world_height", Some("adventure/trade"), "emerald", "Star Trader", "Trade with a Villager at the build height limit", Task, None, -2.0, 4.0, &triggers::VILLAGER_TRADE);
        // Smithing with Style
        self.adv("adventure/trim_with_all_exclusive_armor_patterns", Some("adventure/trim_with_any_armor_pattern"), "silence_armor_trim_smithing_template", "Smithing with Style", "Apply these smithing templates at least once: Spire, Snout, Rib, Ward, Silence, Vex, Tide, Wayfinder", Challenge, None, -4.0, 0.0, &triggers::RECIPE_CRAFTED);
        self.register_adventure_advancements_part2();
    }

    fn register_adventure_advancements_part2(&mut self) {
        use AdvancementFrame::*;
        // Is It a Plane?
        self.adv("adventure/spyglass_at_dragon", Some("adventure/spyglass_at_ghast"), "spyglass", "Is It a Plane?", "Look at the Ender Dragon through a Spyglass", Task, None, -6.0, -4.0, &triggers::USING_ITEM);
        // Very Very Frightening
        self.adv("adventure/very_very_frightening", Some("adventure/throw_trident"), "trident", "Very Very Frightening", "Strike a Villager with lightning", Task, None, 8.0, 2.0, &triggers::CHANNELED_LIGHTNING);
        // Sniper Duel
        self.adv("adventure/sniper_duel", Some("adventure/shoot_arrow"), "arrow", "Sniper Duel", "Kill a Skeleton from at least 50 meters away", Challenge, None, 4.0, -2.0, &triggers::PLAYER_KILLED_ENTITY);
        // Bullseye
        self.adv("adventure/bullseye", Some("adventure/shoot_arrow"), "target", "Bullseye", "Hit the bullseye of a Target block from at least 30 meters away", Challenge, None, 6.0, 0.0, &triggers::TARGET_HIT);
        // Caves & Cliffs
        self.adv("adventure/fall_from_world_height", Some("adventure/totem_of_undying"), "water_bucket", "Caves & Cliffs", "Free fall from the top of the world (build limit) to the bottom of the world and survive", Task, None, 4.0, 6.0, &triggers::FALL_FROM_HEIGHT);
        // Respecting the Remnants
        self.adv("adventure/avoid_vibration", Some("adventure/kill_mob_near_sculk_catalyst"), "sculk_sensor", "Respecting the Remnants", "Sneak near a Sculk Sensor or Warden to prevent it from detecting you", Task, None, 2.0, 8.0, &triggers::AVOID_VIBRATION);
        // Sneak 100
        self.adv("adventure/sneak_while_mining", Some("adventure/avoid_vibration"), "swift_sneak_smithing_template", "Sneak 100", "Sneak near a Sculk Sensor without triggering it", Task, None, 2.0, 10.0, &triggers::AVOID_VIBRATION);
        // Two Birds, One Arrow
        self.adv("adventure/two_birds_one_arrow", Some("adventure/sniper_duel"), "crossbow", "Two Birds, One Arrow", "Kill two Phantoms with a piercing Arrow", Challenge, None, 4.0, -4.0, &triggers::PLAYER_KILLED_ENTITY);
        // Who's the Pillager Now?
        self.adv("adventure/whos_the_pillager_now", Some("adventure/sniper_duel"), "crossbow", "Who's the Pillager Now?", "Give a Pillager a taste of their own medicine", Task, None, 6.0, -2.0, &triggers::PLAYER_KILLED_ENTITY);
        // Arbalistic
        self.adv("adventure/arbalistic", Some("adventure/sniper_duel"), "crossbow", "Arbalistic", "Kill five unique mobs with one crossbow shot", Challenge, None, 2.0, -4.0, &triggers::KILLED_BY_CROSSBOW);
        // Careful Restoration
        self.adv("adventure/craft_decorated_pot_using_only_sherds", Some("adventure/sneak_while_mining"), "decorated_pot", "Careful Restoration", "Make a Decorated Pot out of 4 Pottery Sherds", Task, None, 2.0, 12.0, &triggers::RECIPE_CRAFTED);
        // Adventuring Time
        self.adv("adventure/adventuring_time", Some("adventure/sleep_in_bed"), "diamond_boots", "Adventuring Time", "Discover every biome", Challenge, None, 4.0, 2.0, &triggers::LOCATION);
        // Sound of Music
        self.adv("adventure/play_jukebox_in_meadows", Some("adventure/sleep_in_bed"), "jukebox", "Sound of Music", "Make the Meadows come alive with the sound of music from a Jukebox", Task, None, 2.0, -2.0, &triggers::ITEM_USED_ON_BLOCK);
        // Light as a Rabbit
        self.adv("adventure/walk_on_powder_snow_with_leather_boots", Some("adventure/sleep_in_bed"), "leather_boots", "Light as a Rabbit", "Walk on Powder Snow without sinking in it", Task, None, 0.0, -4.0, &triggers::LOCATION);
        // Surge Protector
        self.adv("adventure/lightning_rod_with_villager_no_fire", Some("adventure/very_very_frightening"), "lightning_rod", "Surge Protector", "Protect a Villager from an undesired shock without starting a fire", Task, None, 10.0, 2.0, &triggers::LIGHTNING_STRIKE);
        // Under Lock and Key
        self.adv("adventure/salvage_sherd", Some("adventure/craft_decorated_pot_using_only_sherds"), "brush", "Under Lock and Key", "Unlock a Trial Chamber Vault with a Trial Key", Task, None, 4.0, 12.0, &triggers::ITEM_USED_ON_BLOCK);
        // Revaulting
        self.adv("adventure/ominous_trial_omen", Some("adventure/salvage_sherd"), "ominous_trial_key", "Revaulting", "Unlock an Ominous Trial Chamber Vault", Challenge, None, 6.0, 12.0, &triggers::ITEM_USED_ON_BLOCK);
        // Blowback
        self.adv("adventure/blowback", Some("adventure/salvage_sherd"), "wind_charge", "Blowback", "Kill a Breeze with a deflected Breeze-shot Wind Charge", Challenge, None, 4.0, 14.0, &triggers::PLAYER_KILLED_ENTITY);
        // Who Needs Rockets?
        self.adv("adventure/who_needs_rockets", Some("adventure/blowback"), "wind_charge", "Who Needs Rockets?", "Use a Wind Charge to launch yourself upward at least 8 blocks", Task, None, 4.0, 16.0, &triggers::FALL_AFTER_EXPLOSION);
        // Minecraft: Trial(s) Edition
        self.adv("adventure/minecraft_trials_edition", Some("adventure/ominous_trial_omen"), "trial_spawner", "Minecraft: Trial(s) Edition", "Defeat all unique mobs in a Trial Chamber", Challenge, None, 8.0, 12.0, &triggers::PLAYER_KILLED_ENTITY);
        // Ol' Betsy
        self.adv("adventure/ol_betsy", Some("adventure/kill_a_mob"), "crossbow", "Ol' Betsy", "Shoot a Crossbow", Task, None, 6.0, 4.0, &triggers::SHOT_CROSSBOW);
    }

    fn register_husbandry_advancements(&mut self) {
        use AdvancementFrame::*;
        let bg = Some("textures/gui/advancements/backgrounds/husbandry.png");
        // Root: Husbandry
        self.adv("husbandry/root", None, "hay_block", "Husbandry", "The world is full of friends and food", Task, bg, 0.0, 0.0, &triggers::CONSUME_ITEM);
        // Bee Our Guest
        self.adv("husbandry/safely_harvest_honey", Some("husbandry/root"), "bee_nest", "Bee Our Guest", "Use a Campfire to collect Honey from a Beehive using a Bottle without aggravating the bees", Task, None, 2.0, -4.0, &triggers::ITEM_USED_ON_BLOCK);
        // The Parrots and the Bats
        self.adv("husbandry/breed_an_animal", Some("husbandry/root"), "wheat", "The Parrots and the Bats", "Breed two animals together", Task, None, 2.0, 2.0, &triggers::BRED_ANIMALS);
        // You've Got a Friend in Me
        self.adv("husbandry/allay_deliver_item_to_player", Some("husbandry/root"), "allay_spawn_egg", "You've Got a Friend in Me", "Have an Allay deliver items to you", Task, None, -2.0, 0.0, &triggers::ALLAY_DROP_ITEM_ON_BLOCK);
        // Whatever Floats Your Goat!
        self.adv("husbandry/ride_a_boat_with_a_goat", Some("husbandry/root"), "oak_boat", "Whatever Floats Your Goat!", "Get in a Boat and float with a Goat", Task, None, 0.0, 2.0, &triggers::STARTED_RIDING);
        // Best Friends Forever
        self.adv("husbandry/tame_an_animal", Some("husbandry/root"), "lead", "Best Friends Forever", "Tame an animal", Task, None, -2.0, 2.0, &triggers::TAME_ANIMAL);
        // Glow and Behold!
        self.adv("husbandry/make_a_sign_glow", Some("husbandry/root"), "glow_ink_sac", "Glow and Behold!", "Make the text of any kind of sign glow", Task, None, 0.0, -2.0, &triggers::ITEM_USED_ON_BLOCK);
        // Fishy Business
        self.adv("husbandry/fishy_business", Some("husbandry/root"), "fishing_rod", "Fishy Business", "Catch a fish", Task, None, 2.0, 0.0, &triggers::FISHING_ROD_HOOKED);
        // Total Beelocation
        self.adv("husbandry/silk_touch_nest", Some("husbandry/safely_harvest_honey"), "bee_nest", "Total Beelocation", "Move a Bee Nest, with 3 bees inside, using Silk Touch", Task, None, 2.0, -6.0, &triggers::BEE_NEST_DESTROYED);
        // Wax On
        self.adv("husbandry/wax_on", Some("husbandry/safely_harvest_honey"), "honeycomb", "Wax On", "Apply Honeycomb to a Copper block!", Task, None, 4.0, -4.0, &triggers::ITEM_USED_ON_BLOCK);
        // Two by Two
        self.adv("husbandry/bred_all_animals", Some("husbandry/breed_an_animal"), "golden_carrot", "Two by Two", "Breed all the animals!", Challenge, None, 2.0, 4.0, &triggers::BRED_ANIMALS);
        // Birthday Song
        self.adv("husbandry/allay_deliver_cake_to_note_block", Some("husbandry/allay_deliver_item_to_player"), "note_block", "Birthday Song", "Have an Allay drop a Cake at a Note Block", Task, None, -4.0, 0.0, &triggers::ALLAY_DROP_ITEM_ON_BLOCK);
        // A Complete Catalogue
        self.adv("husbandry/complete_catalogue", Some("husbandry/tame_an_animal"), "cod", "A Complete Catalogue", "Tame all cat variants!", Challenge, None, -2.0, 4.0, &triggers::TAME_ANIMAL);
        // Tactical Fishing
        self.adv("husbandry/tactical_fishing", Some("husbandry/fishy_business"), "pufferfish_bucket", "Tactical Fishing", "Catch a Fish in a Bucket!", Task, None, 4.0, 0.0, &triggers::FILLED_BUCKET);
        // When the Squad Hops into Town
        self.adv("husbandry/leash_all_frog_variants", Some("husbandry/breed_an_animal"), "lead", "When the Squad Hops into Town", "Get each Frog variant on a Lead", Task, None, 0.0, 4.0, &triggers::PLAYER_INTERACTED_WITH_ENTITY);
        // Little Sniffs
        self.adv("husbandry/feed_snifflet", Some("husbandry/breed_an_animal"), "torchflower_seeds", "Little Sniffs", "Feed a Snifflet", Task, None, 4.0, 2.0, &triggers::PLAYER_INTERACTED_WITH_ENTITY);
        // Wax Off
        self.adv("husbandry/wax_off", Some("husbandry/wax_on"), "stone_axe", "Wax Off", "Scrape Wax off of a Copper block!", Task, None, 6.0, -4.0, &triggers::ITEM_USED_ON_BLOCK);
        // The Cutest Predator
        self.adv("husbandry/axolotl_in_a_bucket", Some("husbandry/tactical_fishing"), "axolotl_bucket", "The Cutest Predator", "Catch an Axolotl in a Bucket", Task, None, 6.0, 0.0, &triggers::FILLED_BUCKET);
        // With Our Powers Combined!
        self.adv("husbandry/froglights", Some("husbandry/leash_all_frog_variants"), "verdant_froglight", "With Our Powers Combined!", "Have all Froglights in your inventory", Task, None, 0.0, 6.0, &triggers::INVENTORY_CHANGED);
        // Planting the Past
        self.adv("husbandry/plant_any_sniffer_seed", Some("husbandry/feed_snifflet"), "torchflower_seeds", "Planting the Past", "Plant any Sniffer seed", Task, None, 6.0, 2.0, &triggers::ITEM_USED_ON_BLOCK);
        // The Healing Power of Friendship!
        self.adv("husbandry/kill_axolotl_target", Some("husbandry/axolotl_in_a_bucket"), "tropical_fish_bucket", "The Healing Power of Friendship!", "Team up with an axolotl and win a fight", Task, None, 8.0, 0.0, &triggers::PLAYER_KILLED_ENTITY);
        // A Balanced Diet
        self.adv("husbandry/balanced_diet", Some("husbandry/bred_all_animals"), "apple", "A Balanced Diet", "Eat everything that is edible, even if it's not good for you", Challenge, None, 2.0, 6.0, &triggers::CONSUME_ITEM);
        // Serious Dedication
        self.adv("husbandry/obtain_netherite_hoe", Some("husbandry/bred_all_animals"), "netherite_hoe", "Serious Dedication", "Use a Netherite Ingot to upgrade a Hoe, and then reevaluate your life choices", Challenge, None, 4.0, 4.0, &triggers::INVENTORY_CHANGED);
        // A Seedy Place
        self.adv("husbandry/plant_seed", Some("husbandry/root"), "wheat_seeds", "A Seedy Place", "Plant a seed and watch it grow", Task, None, -2.0, -2.0, &triggers::ITEM_USED_ON_BLOCK);
        // Good as New
        self.adv("husbandry/repair_wolf_armor", Some("husbandry/tame_an_animal"), "wolf_armor", "Good as New", "Repair a damaged Wolf Armor using Armadillo Scutes", Task, None, -4.0, 2.0, &triggers::PLAYER_INTERACTED_WITH_ENTITY);
        // The Whole Pack
        self.adv("husbandry/whole_pack", Some("husbandry/complete_catalogue"), "bone", "The Whole Pack", "Tame all dog variants!", Challenge, None, -2.0, 6.0, &triggers::TAME_ANIMAL);
        // Shear Brilliance
        self.adv("husbandry/remove_wolf_armor", Some("husbandry/repair_wolf_armor"), "shears", "Shear Brilliance", "Remove Wolf Armor from a Wolf using Shears", Task, None, -6.0, 2.0, &triggers::PLAYER_INTERACTED_WITH_ENTITY);
        // Smells Interesting
        self.adv("husbandry/obtain_sniffer_egg", Some("husbandry/plant_any_sniffer_seed"), "sniffer_egg", "Smells Interesting", "Obtain a Sniffer Egg", Task, None, 8.0, 2.0, &triggers::INVENTORY_CHANGED);
    }
}
