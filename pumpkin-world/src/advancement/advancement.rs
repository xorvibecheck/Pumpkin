use pumpkin_util::resource_location::ResourceLocation;
use serde::{Deserialize, Serialize};

use super::{
    AdvancementCriterion, AdvancementDisplay, AdvancementRequirements, AdvancementRewards,
    CriteriaMap,
};

/// A complete advancement definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advancement {
    /// The parent advancement ID. `None` for root advancements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<ResourceLocation>,
    /// Display information. `None` for hidden advancements (used for recipes, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<AdvancementDisplay>,
    /// Rewards granted when the advancement is completed.
    #[serde(default, skip_serializing_if = "AdvancementRewards::is_empty")]
    pub rewards: AdvancementRewards,
    /// The criteria that can be completed for this advancement.
    pub criteria: CriteriaMap,
    /// The requirements for completing this advancement.
    /// If not specified, defaults to requiring all criteria (AND).
    #[serde(default)]
    pub requirements: AdvancementRequirements,
    /// Whether completing this advancement sends telemetry data.
    #[serde(default)]
    pub sends_telemetry_event: bool,
}

impl Advancement {
    /// Creates a new advancement with the given criteria.
    #[must_use]
    pub fn new(criteria: CriteriaMap) -> Self {
        let requirements = AdvancementRequirements::all_of(criteria.keys().cloned());
        Self {
            parent: None,
            display: None,
            rewards: AdvancementRewards::empty(),
            criteria,
            requirements,
            sends_telemetry_event: false,
        }
    }

    /// Creates a new root advancement (no parent) with display information.
    #[must_use]
    pub fn root(display: AdvancementDisplay, criteria: CriteriaMap) -> Self {
        let requirements = AdvancementRequirements::all_of(criteria.keys().cloned());
        Self {
            parent: None,
            display: Some(display),
            rewards: AdvancementRewards::empty(),
            criteria,
            requirements,
            sends_telemetry_event: false,
        }
    }

    /// Creates a builder for constructing an advancement.
    #[must_use]
    pub fn builder() -> AdvancementBuilder {
        AdvancementBuilder::new()
    }

    /// Returns whether this is a root advancement (has no parent).
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    /// Returns whether this advancement has display information.
    #[must_use]
    pub fn has_display(&self) -> bool {
        self.display.is_some()
    }
}

/// An advancement entry with its identifier.
#[derive(Debug, Clone)]
pub struct AdvancementEntry {
    /// The unique identifier for this advancement.
    pub id: ResourceLocation,
    /// The advancement definition.
    pub advancement: Advancement,
}

impl AdvancementEntry {
    /// Creates a new advancement entry.
    #[must_use]
    pub fn new(id: ResourceLocation, advancement: Advancement) -> Self {
        Self { id, advancement }
    }
}

/// Builder for constructing advancements.
#[derive(Debug, Default)]
pub struct AdvancementBuilder {
    parent: Option<ResourceLocation>,
    display: Option<AdvancementDisplay>,
    rewards: AdvancementRewards,
    criteria: CriteriaMap,
    requirements: Option<AdvancementRequirements>,
    sends_telemetry_event: bool,
}

impl AdvancementBuilder {
    /// Creates a new advancement builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the parent advancement.
    #[must_use]
    pub fn parent(mut self, parent: ResourceLocation) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Sets the display information.
    #[must_use]
    pub fn display(mut self, display: AdvancementDisplay) -> Self {
        self.display = Some(display);
        self
    }

    /// Sets the rewards.
    #[must_use]
    pub fn rewards(mut self, rewards: AdvancementRewards) -> Self {
        self.rewards = rewards;
        self
    }

    /// Adds a criterion.
    #[must_use]
    pub fn criterion(mut self, name: impl Into<String>, criterion: AdvancementCriterion) -> Self {
        self.criteria.insert(name.into(), criterion);
        self
    }

    /// Sets the requirements.
    #[must_use]
    pub fn requirements(mut self, requirements: AdvancementRequirements) -> Self {
        self.requirements = Some(requirements);
        self
    }

    /// Sets whether this advancement sends telemetry events.
    #[must_use]
    pub fn sends_telemetry_event(mut self) -> Self {
        self.sends_telemetry_event = true;
        self
    }

    /// Builds the advancement.
    #[must_use]
    pub fn build(self) -> Advancement {
        let requirements = self.requirements.unwrap_or_else(|| {
            AdvancementRequirements::all_of(self.criteria.keys().cloned())
        });

        Advancement {
            parent: self.parent,
            display: self.display,
            rewards: self.rewards,
            criteria: self.criteria,
            requirements,
            sends_telemetry_event: self.sends_telemetry_event,
        }
    }

    /// Builds the advancement with an ID.
    #[must_use]
    pub fn build_entry(self, id: ResourceLocation) -> AdvancementEntry {
        AdvancementEntry::new(id, self.build())
    }
}

/// A positioned advancement in the advancement tree.
/// This is used for calculating positions in the advancement GUI.
#[derive(Debug, Clone)]
pub struct PlacedAdvancement {
    /// The advancement entry.
    pub entry: AdvancementEntry,
    /// The parent placed advancement, if any.
    pub parent: Option<ResourceLocation>,
    /// Child advancements.
    pub children: Vec<ResourceLocation>,
}

impl PlacedAdvancement {
    /// Creates a new placed advancement.
    #[must_use]
    pub fn new(entry: AdvancementEntry) -> Self {
        let parent = entry.advancement.parent.clone();
        Self {
            entry,
            parent,
            children: Vec::new(),
        }
    }

    /// Adds a child advancement.
    pub fn add_child(&mut self, child_id: ResourceLocation) {
        self.children.push(child_id);
    }
}
