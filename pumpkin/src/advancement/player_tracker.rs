use std::collections::{HashMap, HashSet};

use pumpkin_protocol::java::client::play::AdvancementProgressMapping;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_world::advancement::{AdvancementProgress, AdvancementRequirements};

#[derive(Debug)]
pub struct PlayerAdvancementTracker {
    progress: HashMap<ResourceLocation, AdvancementProgress>,
    completed: HashSet<ResourceLocation>,
    dirty: HashSet<ResourceLocation>,
    current_tab: Option<ResourceLocation>,
    needs_reset: bool,
    visible: HashSet<ResourceLocation>,
}

impl Default for PlayerAdvancementTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerAdvancementTracker {
    #[must_use]
    pub fn new() -> Self {
        Self {
            progress: HashMap::new(),
            completed: HashSet::new(),
            dirty: HashSet::new(),
            current_tab: None,
            needs_reset: true,
            visible: HashSet::new(),
        }
    }

    #[must_use]
    pub fn get_progress(&self, id: &ResourceLocation) -> Option<&AdvancementProgress> {
        self.progress.get(id)
    }

    pub fn get_or_create_progress(&mut self, id: &ResourceLocation) -> &mut AdvancementProgress {
        self.progress.entry(id.clone()).or_default()
    }

    pub fn grant_criterion(
        &mut self,
        advancement_id: &ResourceLocation,
        criterion: &str,
        requirements: &AdvancementRequirements,
    ) -> bool {
        let progress = self.progress.entry(advancement_id.clone()).or_default();

        if progress.grant_criterion(criterion) {
            progress.update_done(requirements);
            let is_done = progress.is_done();

            self.dirty.insert(advancement_id.clone());

            if is_done {
                self.completed.insert(advancement_id.clone());
            }
            true
        } else {
            false
        }
    }

    pub fn revoke_criterion(
        &mut self,
        advancement_id: &ResourceLocation,
        criterion: &str,
        requirements: &AdvancementRequirements,
    ) -> bool {
        if let Some(progress) = self.progress.get_mut(advancement_id) {
            if progress.revoke_criterion(criterion) {
                progress.update_done(requirements);
                self.dirty.insert(advancement_id.clone());
                self.completed.remove(advancement_id);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn grant_advancement(
        &mut self,
        advancement_id: &ResourceLocation,
        requirements: &AdvancementRequirements,
    ) -> bool {
        let progress = self.progress.entry(advancement_id.clone()).or_default();

        let mut changed = false;

        for criterion_name in requirements.get_names() {
            if progress.grant_criterion(&criterion_name) {
                changed = true;
            }
        }

        if changed {
            progress.update_done(requirements);
            let is_done = progress.is_done();

            self.dirty.insert(advancement_id.clone());
            if is_done {
                self.completed.insert(advancement_id.clone());
            }
        }

        changed
    }

    pub fn revoke_advancement(&mut self, advancement_id: &ResourceLocation) -> bool {
        if self.progress.remove(advancement_id).is_some() {
            self.completed.remove(advancement_id);
            self.dirty.insert(advancement_id.clone());
            true
        } else {
            false
        }
    }

    #[must_use]
    pub fn is_completed(&self, id: &ResourceLocation) -> bool {
        self.completed.contains(id)
    }

    #[must_use]
    pub fn completed_advancements(&self) -> &HashSet<ResourceLocation> {
        &self.completed
    }

    pub fn set_current_tab(&mut self, tab: Option<ResourceLocation>) {
        self.current_tab = tab;
    }

    #[must_use]
    pub fn current_tab(&self) -> Option<&ResourceLocation> {
        self.current_tab.as_ref()
    }

    pub fn mark_needs_reset(&mut self) {
        self.needs_reset = true;
    }

    pub fn clear_dirty(&mut self) {
        self.dirty.clear();
        self.needs_reset = false;
    }

    #[must_use]
    pub fn has_pending_updates(&self) -> bool {
        self.needs_reset || !self.dirty.is_empty()
    }

    #[must_use]
    pub fn dirty_advancements(&self) -> &HashSet<ResourceLocation> {
        &self.dirty
    }

    #[must_use]
    pub fn needs_reset(&self) -> bool {
        self.needs_reset
    }

    pub fn mark_visible(&mut self, id: ResourceLocation) {
        self.visible.insert(id);
    }

    #[must_use]
    pub fn visible_advancements(&self) -> &HashSet<ResourceLocation> {
        &self.visible
    }

    pub fn load_progress(&mut self, data: HashMap<ResourceLocation, AdvancementProgress>) {
        self.progress = data;
        self.completed.clear();

        for (id, progress) in &self.progress {
            if progress.is_done() {
                self.completed.insert(id.clone());
            }
        }

        self.needs_reset = true;
    }

    #[must_use]
    pub fn save_progress(&self) -> &HashMap<ResourceLocation, AdvancementProgress> {
        &self.progress
    }

    #[must_use]
    pub fn to_progress_mappings(&self) -> Vec<AdvancementProgressMapping> {
        self.progress
            .iter()
            .map(|(id, progress)| {
                let criteria = progress
                    .criteria
                    .iter()
                    .map(|(name, cp)| {
                        pumpkin_protocol::java::client::play::CriterionProgressEntry {
                            criterion: name.clone(),
                            progress: cp.obtained_time,
                        }
                    })
                    .collect();

                AdvancementProgressMapping::new(
                    id.clone(),
                    pumpkin_protocol::java::client::play::AdvancementProgress::with_criteria(
                        criteria,
                    ),
                )
            })
            .collect()
    }

    #[must_use]
    pub fn dirty_progress_mappings(&self) -> Vec<AdvancementProgressMapping> {
        self.dirty
            .iter()
            .filter_map(|id| {
                self.progress.get(id).map(|progress| {
                    let criteria = progress
                        .criteria
                        .iter()
                        .map(|(name, cp)| {
                            pumpkin_protocol::java::client::play::CriterionProgressEntry {
                                criterion: name.clone(),
                                progress: cp.obtained_time,
                            }
                        })
                        .collect();

                    AdvancementProgressMapping::new(
                        id.clone(),
                        pumpkin_protocol::java::client::play::AdvancementProgress::with_criteria(
                            criteria,
                        ),
                    )
                })
            })
            .collect()
    }
}
