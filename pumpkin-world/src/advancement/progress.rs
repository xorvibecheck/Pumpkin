use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::AdvancementRequirements;

/// Tracks the progress of a single advancement for a player.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdvancementProgress {
    /// Progress for each criterion, keyed by criterion name.
    #[serde(default)]
    pub criteria: HashMap<String, CriterionProgress>,
    /// Whether this advancement is done (all requirements met).
    #[serde(skip)]
    done: bool,
}

impl AdvancementProgress {
    /// Creates a new empty progress tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            criteria: HashMap::new(),
            done: false,
        }
    }

    /// Initializes progress with the given requirements.
    /// Sets up criterion entries for all required criteria.
    pub fn init(&mut self, requirements: &AdvancementRequirements) {
        for name in requirements.get_names() {
            self.criteria.entry(name).or_insert_with(CriterionProgress::new);
        }
        self.update_done(requirements);
    }

    /// Grants a criterion, marking it as obtained.
    /// Returns `true` if the criterion was newly granted.
    pub fn grant_criterion(&mut self, criterion: &str) -> bool {
        if let Some(progress) = self.criteria.get_mut(criterion) {
            if !progress.is_obtained() {
                progress.obtain();
                return true;
            }
        } else {
            let mut progress = CriterionProgress::new();
            progress.obtain();
            self.criteria.insert(criterion.to_string(), progress);
            return true;
        }
        false
    }

    /// Revokes a criterion, marking it as not obtained.
    /// Returns `true` if the criterion was previously obtained.
    pub fn revoke_criterion(&mut self, criterion: &str) -> bool {
        if let Some(progress) = self.criteria.get_mut(criterion) {
            if progress.is_obtained() {
                progress.reset();
                self.done = false;
                return true;
            }
        }
        false
    }

    /// Returns whether a specific criterion is obtained.
    #[must_use]
    pub fn is_criterion_obtained(&self, criterion: &str) -> bool {
        self.criteria
            .get(criterion)
            .is_some_and(CriterionProgress::is_obtained)
    }

    /// Returns all obtained criteria names.
    #[must_use]
    pub fn get_obtained_criteria(&self) -> Vec<&str> {
        self.criteria
            .iter()
            .filter(|(_, p)| p.is_obtained())
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Returns all remaining (not obtained) criteria names.
    #[must_use]
    pub fn get_remaining_criteria(&self, requirements: &AdvancementRequirements) -> Vec<String> {
        let obtained: std::collections::HashSet<_> = self.get_obtained_criteria().into_iter().collect();
        requirements
            .get_names()
            .into_iter()
            .filter(|name| !obtained.contains(name.as_str()))
            .collect()
    }

    /// Updates the done status based on requirements.
    pub fn update_done(&mut self, requirements: &AdvancementRequirements) {
        let obtained: std::collections::HashSet<String> = self
            .criteria
            .iter()
            .filter(|(_, p)| p.is_obtained())
            .map(|(name, _)| name.clone())
            .collect();
        self.done = requirements.test(&obtained);
    }

    /// Returns whether this advancement is complete.
    #[must_use]
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// Returns the completion percentage (0.0 to 1.0).
    #[must_use]
    pub fn get_percent(&self, requirements: &AdvancementRequirements) -> f32 {
        if requirements.is_empty() {
            return 1.0;
        }
        let obtained: std::collections::HashSet<String> = self
            .criteria
            .iter()
            .filter(|(_, p)| p.is_obtained())
            .map(|(name, _)| name.clone())
            .collect();
        let completed = requirements.count_completed(&obtained);
        completed as f32 / requirements.len() as f32
    }

    /// Returns the earliest time any criterion was obtained.
    #[must_use]
    pub fn get_earliest_progress_time(&self) -> Option<i64> {
        self.criteria
            .values()
            .filter_map(|p| p.obtained_time)
            .min()
    }
}

/// Tracks the progress of a single criterion.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CriterionProgress {
    /// The time when this criterion was obtained (Unix timestamp in milliseconds).
    /// `None` if not yet obtained.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obtained_time: Option<i64>,
}

impl CriterionProgress {
    /// Creates a new criterion progress that is not yet obtained.
    #[must_use]
    pub const fn new() -> Self {
        Self { obtained_time: None }
    }

    /// Returns whether this criterion is obtained.
    #[must_use]
    pub const fn is_obtained(&self) -> bool {
        self.obtained_time.is_some()
    }

    /// Marks this criterion as obtained at the current time.
    pub fn obtain(&mut self) {
        if self.obtained_time.is_none() {
            // Use system time in milliseconds since Unix epoch
            self.obtained_time = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0),
            );
        }
    }

    /// Marks this criterion as obtained at a specific time.
    pub fn obtain_at(&mut self, time: i64) {
        self.obtained_time = Some(time);
    }

    /// Resets this criterion to not obtained.
    pub fn reset(&mut self) {
        self.obtained_time = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracking() {
        let requirements = AdvancementRequirements::all_of([
            "criterion_a".to_string(),
            "criterion_b".to_string(),
        ]);

        let mut progress = AdvancementProgress::new();
        progress.init(&requirements);

        assert!(!progress.is_done());
        assert_eq!(progress.get_percent(&requirements), 0.0);

        progress.grant_criterion("criterion_a");
        progress.update_done(&requirements);
        assert!(!progress.is_done());
        assert_eq!(progress.get_percent(&requirements), 0.5);

        progress.grant_criterion("criterion_b");
        progress.update_done(&requirements);
        assert!(progress.is_done());
        assert_eq!(progress.get_percent(&requirements), 1.0);
    }
}
