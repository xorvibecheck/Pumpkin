use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;
use serde::{Deserialize, Serialize};

use super::AdvancementFrame;

/// Display information for an advancement shown in the advancement GUI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancementDisplay {
    /// The icon item to display (as item ID or resource location).
    pub icon: AdvancementIcon,
    /// The title of the advancement.
    pub title: TextComponent,
    /// The description of the advancement.
    pub description: TextComponent,
    /// The frame type (task, goal, or challenge).
    #[serde(default)]
    pub frame: AdvancementFrame,
    /// The background texture for root advancements (only used for root advancements).
    #[serde(default)]
    pub background: Option<ResourceLocation>,
    /// Whether to show a toast notification when this advancement is completed.
    #[serde(default = "default_true")]
    pub show_toast: bool,
    /// Whether to announce this advancement in chat when completed.
    #[serde(default = "default_true")]
    pub announce_to_chat: bool,
    /// Whether this advancement is hidden until its parent is completed.
    #[serde(default)]
    pub hidden: bool,
    /// The X position in the advancement tab (set by the positioner).
    #[serde(skip)]
    pub x: f32,
    /// The Y position in the advancement tab (set by the positioner).
    #[serde(skip)]
    pub y: f32,
}

fn default_true() -> bool {
    true
}

impl AdvancementDisplay {
    /// Creates a new advancement display with the given properties.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        icon: AdvancementIcon,
        title: TextComponent,
        description: TextComponent,
        frame: AdvancementFrame,
        background: Option<ResourceLocation>,
        show_toast: bool,
        announce_to_chat: bool,
        hidden: bool,
    ) -> Self {
        Self {
            icon,
            title,
            description,
            frame,
            background,
            show_toast,
            announce_to_chat,
            hidden,
            x: 0.0,
            y: 0.0,
        }
    }

    /// Sets the position of this display in the advancement tab.
    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
}

/// The icon displayed for an advancement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancementIcon {
    /// The item identifier (e.g., "minecraft:diamond").
    #[serde(rename = "id")]
    pub item: ResourceLocation,
    /// The item count (defaults to 1).
    #[serde(default = "default_count", skip_serializing_if = "is_default_count")]
    pub count: i32,
    /// Optional NBT/component data for the item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub components: Option<serde_json::Value>,
}

fn default_count() -> i32 {
    1
}

fn is_default_count(count: &i32) -> bool {
    *count == 1
}

impl AdvancementIcon {
    /// Creates a simple icon from an item identifier.
    #[must_use]
    pub fn simple(item: ResourceLocation) -> Self {
        Self {
            item,
            count: 1,
            components: None,
        }
    }

    /// Creates an icon with item components/NBT data.
    #[must_use]
    pub fn with_components(item: ResourceLocation, components: serde_json::Value) -> Self {
        Self {
            item,
            count: 1,
            components: Some(components),
        }
    }

    /// Creates an icon with a specific count.
    #[must_use]
    pub fn with_count(item: ResourceLocation, count: i32) -> Self {
        Self {
            item,
            count,
            components: None,
        }
    }
}
