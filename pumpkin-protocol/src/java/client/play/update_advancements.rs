use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_UPDATE_ADVANCEMENTS;
use pumpkin_macros::packet;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;

use crate::codec::var_int::VarInt;
use crate::ser::NetworkWriteExt;
use crate::{ClientPacket, WritingError};

/// Sent by the server to update the client's advancement data.
#[packet(PLAY_UPDATE_ADVANCEMENTS)]
pub struct CUpdateAdvancements<'a> {
    /// Whether to reset/clear existing advancements before applying this update.
    pub reset: bool,
    /// Advancements to add or update.
    pub advancements: &'a [AdvancementMapping],
    /// Identifiers of advancements to remove.
    pub identifiers_to_remove: &'a [ResourceLocation],
    /// Progress updates for advancements.
    pub progress: &'a [AdvancementProgressMapping],
}

impl<'a> CUpdateAdvancements<'a> {
    /// Creates a new advancement update packet.
    #[must_use]
    pub fn new(
        reset: bool,
        advancements: &'a [AdvancementMapping],
        identifiers_to_remove: &'a [ResourceLocation],
        progress: &'a [AdvancementProgressMapping],
    ) -> Self {
        Self {
            reset,
            advancements,
            identifiers_to_remove,
            progress,
        }
    }

    /// Creates a packet that resets all advancements and sends new ones.
    #[must_use]
    pub fn reset_with(
        advancements: &'a [AdvancementMapping],
        progress: &'a [AdvancementProgressMapping],
    ) -> Self {
        Self {
            reset: true,
            advancements,
            identifiers_to_remove: &[],
            progress,
        }
    }

    /// Creates a packet that adds/updates advancements without reset.
    #[must_use]
    pub fn update(
        advancements: &'a [AdvancementMapping],
        progress: &'a [AdvancementProgressMapping],
    ) -> Self {
        Self {
            reset: false,
            advancements,
            identifiers_to_remove: &[],
            progress,
        }
    }
}

impl ClientPacket for CUpdateAdvancements<'_> {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;

        // Reset flag
        write.write_bool(self.reset)?;

        // Advancements to add/update
        write.write_var_int(&VarInt(self.advancements.len() as i32))?;
        for advancement in self.advancements {
            advancement.write(&mut write)?;
        }

        // Advancements to remove
        write.write_var_int(&VarInt(self.identifiers_to_remove.len() as i32))?;
        for id in self.identifiers_to_remove {
            write.write_resource_location(id)?;
        }

        // Progress updates
        write.write_var_int(&VarInt(self.progress.len() as i32))?;
        for progress in self.progress {
            progress.write(&mut write)?;
        }

        Ok(())
    }
}

/// An advancement with its identifier.
#[derive(Debug, Clone)]
pub struct AdvancementMapping {
    /// The unique identifier for this advancement.
    pub id: ResourceLocation,
    /// The advancement data.
    pub advancement: Advancement,
}

impl AdvancementMapping {
    /// Creates a new advancement mapping.
    #[must_use]
    pub fn new(id: ResourceLocation, advancement: Advancement) -> Self {
        Self { id, advancement }
    }

    fn write(&self, write: &mut impl Write) -> Result<(), WritingError> {
        write.write_resource_location(&self.id)?;
        self.advancement.write(write)
    }
}

/// Advancement data sent to the client.
#[derive(Debug, Clone)]
pub struct Advancement {
    /// The parent advancement ID, if any.
    pub parent: Option<ResourceLocation>,
    /// Display information, if this advancement should be visible in the UI.
    pub display: Option<AdvancementDisplay>,
    /// The requirements for completing this advancement.
    pub requirements: Vec<Vec<String>>,
    /// Whether completing this advancement sends telemetry data.
    pub sends_telemetry_event: bool,
}

impl Advancement {
    /// Creates a new advancement with no parent or display.
    #[must_use]
    pub fn new(requirements: Vec<Vec<String>>, sends_telemetry_event: bool) -> Self {
        Self {
            parent: None,
            display: None,
            requirements,
            sends_telemetry_event,
        }
    }

    /// Creates a root advancement (no parent) with display.
    #[must_use]
    pub fn root(display: AdvancementDisplay, requirements: Vec<Vec<String>>) -> Self {
        Self {
            parent: None,
            display: Some(display),
            requirements,
            sends_telemetry_event: false,
        }
    }

    /// Creates a child advancement with display.
    #[must_use]
    pub fn child(
        parent: ResourceLocation,
        display: AdvancementDisplay,
        requirements: Vec<Vec<String>>,
    ) -> Self {
        Self {
            parent: Some(parent),
            display: Some(display),
            requirements,
            sends_telemetry_event: false,
        }
    }

    fn write(&self, write: &mut impl Write) -> Result<(), WritingError> {
        // Parent
        write.write_option(&self.parent, |w, p| w.write_resource_location(p))?;

        // Display
        write.write_option(&self.display, |w, d| d.write(w))?;

        // Requirements (list of lists of strings)
        write.write_var_int(&VarInt(self.requirements.len() as i32))?;
        for requirement_group in &self.requirements {
            write.write_var_int(&VarInt(requirement_group.len() as i32))?;
            for criterion in requirement_group {
                write.write_string(criterion)?;
            }
        }

        // Sends telemetry event
        write.write_bool(self.sends_telemetry_event)
    }
}

/// Display information for an advancement.
#[derive(Debug, Clone)]
pub struct AdvancementDisplay {
    /// The title of the advancement.
    pub title: TextComponent,
    /// The description of the advancement.
    pub description: TextComponent,
    /// The icon item (as slot data).
    pub icon: AdvancementIcon,
    /// The frame type (task, goal, challenge).
    pub frame: AdvancementFrame,
    /// Flags for display options.
    pub flags: i32,
    /// Background texture for root advancements.
    pub background: Option<ResourceLocation>,
    /// X position in the tab.
    pub x: f32,
    /// Y position in the tab.
    pub y: f32,
}

impl AdvancementDisplay {
    fn write(&self, write: &mut impl Write) -> Result<(), WritingError> {
        // Title
        write.write_slice(&self.title.encode())?;
        // Description
        write.write_slice(&self.description.encode())?;
        // Icon (item stack)
        self.icon.write(write)?;
        // Frame type
        write.write_var_int(&VarInt(self.frame as i32))?;
        // Flags
        write.write_i32_be(self.flags)?;
        // Background (only if flag bit 0 is set)
        if self.flags & 0x01 != 0 {
            if let Some(ref bg) = self.background {
                write.write_resource_location(bg)?;
            }
        }
        // Position
        write.write_f32_be(self.x)?;
        write.write_f32_be(self.y)
    }
}

/// Icon for an advancement (simplified item representation).
#[derive(Debug, Clone)]
pub struct AdvancementIcon {
    /// The item count (0 for empty).
    pub count: i32,
    /// The item ID if count > 0.
    pub item_id: Option<VarInt>,
    /// Number of components to add.
    pub components_to_add: i32,
    /// Number of components to remove.
    pub components_to_remove: i32,
}

impl AdvancementIcon {
    /// Creates an empty icon.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            count: 0,
            item_id: None,
            components_to_add: 0,
            components_to_remove: 0,
        }
    }

    /// Creates an icon with just an item ID.
    #[must_use]
    pub fn item(item_id: i32) -> Self {
        Self {
            count: 1,
            item_id: Some(VarInt(item_id)),
            components_to_add: 0,
            components_to_remove: 0,
        }
    }

    fn write(&self, write: &mut impl Write) -> Result<(), WritingError> {
        write.write_var_int(&VarInt(self.count))?;
        if self.count > 0 {
            if let Some(ref item_id) = self.item_id {
                write.write_var_int(item_id)?;
            }
            write.write_var_int(&VarInt(self.components_to_add))?;
            write.write_var_int(&VarInt(self.components_to_remove))?;
            // TODO: Write actual component data if needed
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum AdvancementFrame {
    #[default]
    Task = 0,
    Goal = 1,
    Challenge = 2,
}

impl AdvancementFrame {
    #[must_use]
    pub const fn from_ordinal(ordinal: i32) -> Option<Self> {
        match ordinal {
            0 => Some(Self::Task),
            1 => Some(Self::Goal),
            2 => Some(Self::Challenge),
            _ => None,
        }
    }
}

/// Progress mapping for an advancement.
#[derive(Debug, Clone)]
pub struct AdvancementProgressMapping {
    /// The advancement identifier.
    pub id: ResourceLocation,
    /// The progress data.
    pub progress: AdvancementProgress,
}

impl AdvancementProgressMapping {
    /// Creates a new progress mapping.
    #[must_use]
    pub fn new(id: ResourceLocation, progress: AdvancementProgress) -> Self {
        Self { id, progress }
    }

    fn write(&self, write: &mut impl Write) -> Result<(), WritingError> {
        write.write_resource_location(&self.id)?;
        self.progress.write(write)
    }
}

/// Progress for a single advancement.
#[derive(Debug, Clone, Default)]
pub struct AdvancementProgress {
    /// Progress for each criterion.
    pub criteria: Vec<CriterionProgressEntry>,
}

impl AdvancementProgress {
    /// Creates empty progress.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            criteria: Vec::new(),
        }
    }

    /// Creates progress with the given criterion entries.
    #[must_use]
    pub fn with_criteria(criteria: Vec<CriterionProgressEntry>) -> Self {
        Self { criteria }
    }

    /// Creates progress marking all given criteria as done.
    #[must_use]
    pub fn all_done(criterion_names: &[String]) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        Self {
            criteria: criterion_names
                .iter()
                .map(|name| CriterionProgressEntry {
                    criterion: name.clone(),
                    progress: Some(now),
                })
                .collect(),
        }
    }

    fn write(&self, write: &mut impl Write) -> Result<(), WritingError> {
        write.write_var_int(&VarInt(self.criteria.len() as i32))?;
        for entry in &self.criteria {
            entry.write(write)?;
        }
        Ok(())
    }
}

/// Progress entry for a single criterion.
#[derive(Debug, Clone)]
pub struct CriterionProgressEntry {
    /// The criterion name.
    pub criterion: String,
    /// The time when this criterion was achieved (Unix timestamp in millis), or `None` if not achieved.
    pub progress: Option<i64>,
}

impl CriterionProgressEntry {
    /// Creates a criterion entry that is not yet achieved.
    #[must_use]
    pub fn not_achieved(criterion: String) -> Self {
        Self {
            criterion,
            progress: None,
        }
    }

    /// Creates a criterion entry that is achieved at the given time.
    #[must_use]
    pub fn achieved_at(criterion: String, time: i64) -> Self {
        Self {
            criterion,
            progress: Some(time),
        }
    }

    /// Creates a criterion entry that is achieved now.
    #[must_use]
    pub fn achieved_now(criterion: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        Self {
            criterion,
            progress: Some(now),
        }
    }

    fn write(&self, write: &mut impl Write) -> Result<(), WritingError> {
        write.write_string(&self.criterion)?;
        write.write_option(&self.progress, |w, time| w.write_i64_be(*time))
    }
}
