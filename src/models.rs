use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod skating;
pub mod validation;

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
        #[serde(
            serialize_with = "serialize_bib_btreemap",
            deserialize_with = "deserialize_bib_btreemap"
        )]
        marking_crosses: BTreeMap<String, BTreeMap<u32, BTreeMap<Dance, bool>>>,
    },
    DTV {
        #[serde(
            serialize_with = "serialize_bib_btreemap",
            deserialize_with = "deserialize_bib_btreemap"
        )]
        dtv_ranks: BTreeMap<String, BTreeMap<u32, BTreeMap<Dance, u32>>>,
    },
    WDSF {
        #[serde(
            serialize_with = "serialize_bib_btreemap",
            deserialize_with = "deserialize_bib_btreemap"
        )]
        wdsf_scores: BTreeMap<String, BTreeMap<u32, BTreeMap<Dance, WDSFScore>>>,
    },
}

fn serialize_bib_btreemap<S, V>(
    map: &BTreeMap<String, BTreeMap<u32, V>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    V: Serialize,
{
    use serde::ser::SerializeMap;
    let mut outer_map = serializer.serialize_map(Some(map.len()))?;
    for (judge_code, bib_map) in map {
        let mut string_bib_map = BTreeMap::new();
        for (bib, value) in bib_map {
            string_bib_map.insert(bib.to_string(), value);
        }
        outer_map.serialize_entry(judge_code, &string_bib_map)?;
    }
    outer_map.end()
}

fn deserialize_bib_btreemap<'de, D, V>(
    deserializer: D,
) -> Result<BTreeMap<String, BTreeMap<u32, V>>, D::Error>
where
    D: serde::Deserializer<'de>,
    V: Deserialize<'de>,
{
    let outer: BTreeMap<String, BTreeMap<String, V>> = BTreeMap::deserialize(deserializer)?;
    let mut res = BTreeMap::new();
    for (judge_code, string_bib_map) in outer {
        let mut bib_map = BTreeMap::new();
        for (bib_str, value) in string_bib_map {
            if let Ok(bib) = bib_str.parse::<u32>() {
                bib_map.insert(bib, value);
            }
        }
        res.insert(judge_code, bib_map);
    }
    Ok(res)
}

impl RoundData {
    /// Counts the total number of entries (marks/ranks/scores) across all judges and participants.
    pub fn count_entries(&self) -> usize {
        match self {
            Self::Marking { marking_crosses } => marking_crosses
                .values()
                .map(|jm| jm.values().map(|pm| pm.len()).sum::<usize>())
                .sum(),
            Self::DTV { dtv_ranks } => dtv_ranks
                .values()
                .map(|jm| jm.values().map(|pm| pm.len()).sum::<usize>())
                .sum(),
            Self::WDSF { wdsf_scores } => wdsf_scores
                .values()
                .map(|jm| jm.values().map(|pm| pm.len()).sum::<usize>())
                .sum(),
        }
    }

    /// Returns a set of all participant bib numbers present in this round.
    pub fn participant_bibs(&self) -> std::collections::HashSet<u32> {
        let mut bibs = std::collections::HashSet::new();
        match self {
            Self::Marking { marking_crosses } => {
                for judge_map in marking_crosses.values() {
                    for &bib in judge_map.keys() {
                        bibs.insert(bib);
                    }
                }
            }
            Self::DTV { dtv_ranks } => {
                for judge_map in dtv_ranks.values() {
                    for &bib in judge_map.keys() {
                        bibs.insert(bib);
                    }
                }
            }
            Self::WDSF { wdsf_scores } => {
                for judge_map in wdsf_scores.values() {
                    for &bib in judge_map.keys() {
                        bibs.insert(bib);
                    }
                }
            }
        }
        bibs
    }

    /// Checks if a specific judge has provided data for all specified dances for a given participant.
    pub fn has_marks_for(&self, judge_code: &str, bib: u32, dances: &[Dance]) -> bool {
        match self {
            Self::Marking { marking_crosses } => marking_crosses
                .get(judge_code)
                .and_then(|judge_map| judge_map.get(&bib))
                .map(|bib_map| dances.iter().all(|d| bib_map.contains_key(d)))
                .unwrap_or(false),
            Self::DTV { dtv_ranks } => dtv_ranks
                .get(judge_code)
                .and_then(|judge_map| judge_map.get(&bib))
                .map(|bib_map| dances.iter().all(|d| bib_map.contains_key(d)))
                .unwrap_or(false),
            Self::WDSF { wdsf_scores } => wdsf_scores
                .get(judge_code)
                .and_then(|judge_map| judge_map.get(&bib))
                .map(|bib_map| dances.iter().all(|d| bib_map.contains_key(d)))
                .unwrap_or(false),
        }
    }
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
    // 64 chars keeps filenames safe on all major filesystems (FAT32, ext4, APFS).
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
