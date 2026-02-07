use crate::models::{AgeGroup, Style};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Aliases {
    pub age_groups: HashMap<String, String>,
    pub dances: HashMap<String, String>,
    #[serde(default)]
    pub roles: HashMap<String, String>,
}

#[derive(Clone)]
pub struct I18n {
    pub aliases: Aliases,
}

impl I18n {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let aliases: Aliases = toml::from_str(&content)?;
        Ok(Self { aliases })
    }

    pub fn map_age_group(&self, s: &str) -> Option<AgeGroup> {
        self.aliases
            .age_groups
            .get(s)
            .and_then(|id| AgeGroup::from_id(id))
    }

    pub fn map_discipline(&self, s: &str) -> Option<Style> {
        self.aliases
            .dances
            .get(s)
            .and_then(|id| Style::from_id(id))
    }

    pub fn map_role(&self, s: &str) -> Option<String> {
        self.aliases.roles.get(s).cloned()
    }
}
