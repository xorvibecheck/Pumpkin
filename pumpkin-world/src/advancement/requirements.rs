use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Defines the requirements for completing an advancement.
///
/// Requirements are organized as a list of lists (AND of ORs).
/// Each inner list represents a group of criteria where at least one must be completed.
/// All groups must have at least one criterion completed for the advancement to be done.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AdvancementRequirements {
    /// The requirement groups. Outer list is AND, inner lists are OR.
    pub requirements: Vec<Vec<String>>,
}

impl AdvancementRequirements {
    /// Creates empty requirements.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            requirements: Vec::new(),
        }
    }

    /// Creates requirements from a list of requirement groups.
    #[must_use]
    pub fn new(requirements: Vec<Vec<String>>) -> Self {
        Self { requirements }
    }

    /// Creates requirements where ALL criteria must be completed.
    #[must_use]
    pub fn all_of(criteria: impl IntoIterator<Item = String>) -> Self {
        Self {
            requirements: criteria.into_iter().map(|c| vec![c]).collect(),
        }
    }

    /// Creates requirements where ANY ONE criterion must be completed.
    #[must_use]
    pub fn any_of(criteria: impl IntoIterator<Item = String>) -> Self {
        let criteria: Vec<String> = criteria.into_iter().collect();
        if criteria.is_empty() {
            Self::empty()
        } else {
            Self {
                requirements: vec![criteria],
            }
        }
    }

    /// Returns whether these requirements are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.requirements.is_empty()
    }

    /// Returns the total number of requirement groups.
    #[must_use]
    pub fn len(&self) -> usize {
        self.requirements.len()
    }

    /// Returns all unique criterion names referenced in these requirements.
    #[must_use]
    pub fn get_names(&self) -> HashSet<String> {
        let mut names = HashSet::new();
        for group in &self.requirements {
            for name in group {
                names.insert(name.clone());
            }
        }
        names
    }

    /// Tests whether the requirements are satisfied given a set of completed criteria.
    #[must_use]
    pub fn test(&self, completed: &HashSet<String>) -> bool {
        if self.requirements.is_empty() {
            return true;
        }

        for group in &self.requirements {
            // At least one criterion in each group must be completed
            let group_satisfied = group.iter().any(|criterion| completed.contains(criterion));
            if !group_satisfied {
                return false;
            }
        }
        true
    }

    /// Counts the number of requirement groups that are satisfied.
    #[must_use]
    pub fn count_completed(&self, completed: &HashSet<String>) -> usize {
        self.requirements
            .iter()
            .filter(|group| group.iter().any(|criterion| completed.contains(criterion)))
            .count()
    }
}

/// Defines how criteria should be merged into requirements.
#[derive(Debug, Clone, Copy, Default)]
pub enum CriterionMerger {
    /// All criteria must be completed (each criterion is its own requirement group).
    #[default]
    And,
    /// Any one criterion must be completed (all criteria in one requirement group).
    Or,
}

impl CriterionMerger {
    /// Creates requirements from criteria names using this merger strategy.
    #[must_use]
    pub fn create(&self, criteria: impl IntoIterator<Item = String>) -> AdvancementRequirements {
        match self {
            Self::And => AdvancementRequirements::all_of(criteria),
            Self::Or => AdvancementRequirements::any_of(criteria),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_of() {
        let reqs = AdvancementRequirements::all_of(["a".to_string(), "b".to_string()]);
        
        let mut completed = HashSet::new();
        assert!(!reqs.test(&completed));
        
        completed.insert("a".to_string());
        assert!(!reqs.test(&completed));
        
        completed.insert("b".to_string());
        assert!(reqs.test(&completed));
    }

    #[test]
    fn test_any_of() {
        let reqs = AdvancementRequirements::any_of(["a".to_string(), "b".to_string()]);
        
        let mut completed = HashSet::new();
        assert!(!reqs.test(&completed));
        
        completed.insert("a".to_string());
        assert!(reqs.test(&completed));
    }
}
