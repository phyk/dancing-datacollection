use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod validation;
pub mod skating;

/// Represents the skill level of a competition.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Level {
    E,
    D,
    C,
    B,
    A,
    S,
}

/// Represents the dance style (Standard or Latin).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Style {
    Standard,
    Latein,
}

/// Represents an individual dance.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Dance {
    SlowWaltz,
    Tango,
    VienneseWaltz,
    SlowFoxtrot,
    Quickstep,
    Samba,
    ChaChaCha,
    Rumba,
    PasoDoble,
    Jive,
}

/// Represents the age group of the participants.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgeGroup {
    Juv1,
    Juv2,
    Jun1,
    Jun2,
    Youth,
    Adult,
    Adult2,
    Sen1,
    Sen2,
    Sen3,
    Sen4,
    Sen5,
    Senior,
}

/// Represents a judge in a competition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Judge {
    pub code: String,
    pub name: String,
    pub club: Option<String>,
}

/// Represents a committee member (e.g., Chairperson, Scrutineer).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommitteeMember {
    pub name: String,
    pub club: Option<String>,
}

/// Contains all officials responsible for a competition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Officials {
    pub responsible_person: Option<CommitteeMember>,
    pub assistant: Option<CommitteeMember>,
    pub judges: Vec<Judge>,
}

/// Defines whether the participant is a solo dancer or a couple.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum IdentityType {
    Solo,
    Couple,
}

/// Represents a participant (solo or couple) in a competition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Participant {
    pub identity_type: IdentityType,
    pub name_one: String,
    pub bib_number: u32,
    pub name_two: Option<String>,
    pub affiliation: Option<String>,
    pub final_rank: Option<u32>,
}

/// Detailed scores for WDSF competitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WDSFScore {
    pub technical_quality: f64,
    pub movement_to_music: f64,
    pub partnering_skills: f64,
    pub choreography: f64,
    pub total: f64,
}

/// Enum containing the specific scoring data for a round.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "round_type")]
pub enum RoundData {
    #[serde(rename = "Mark")]
    Marking {
        marking_crosses: BTreeMap<String, BTreeMap<String, BTreeMap<Dance, bool>>>,
    },
    DTV {
        dtv_ranks: BTreeMap<String, BTreeMap<String, BTreeMap<Dance, u32>>>,
    },
    WDSF {
        wdsf_scores: BTreeMap<String, BTreeMap<String, BTreeMap<Dance, WDSFScore>>>,
    },
}

/// Represents a single round in a competition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Round {
    pub name: String,
    pub order: u32,
    pub dances: Vec<Dance>,
    #[serde(flatten)]
    pub data: RoundData,
}

/// Represents a dance competition and all its results.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Competition {
    pub name: String,
    pub date: Option<chrono::NaiveDate>,
    pub organizer: Option<String>,
    pub hosting_club: Option<String>,
    pub source_url: Option<String>,
    pub level: Level,
    pub age_group: AgeGroup,
    pub style: Style,
    pub dances: Vec<Dance>,
    pub min_dances: u32,
    pub officials: Officials,
    pub participants: Vec<Participant>,
    pub rounds: Vec<Round>,
}

/// Sanitizes a string to be used as a filename or directory name.
pub fn sanitize_name(name: &str) -> String {
    let mut s: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    s.truncate(64);
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_competition_serde() {
        let json = r#"{"name":"test","level":"D","age_group":"Adult","style":"Standard","dances":[],"min_dances":0,"officials":{"responsible_person":null,"assistant":null,"judges":[]},"participants":[],"rounds":[{"round_type":"DTV","name":"Final","order":0,"dances":[],"dtv_ranks":{"A":{"101":{"SlowWaltz":1}}}}]}"#;
        let _: Competition = serde_json::from_str(json).unwrap();
    }
}
