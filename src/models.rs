use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

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

impl Level {
    /// Creates a Level from an ID string.
    pub fn from_id(id: &str) -> Option<Self> {
        Self::from_str(id).ok()
    }
}

impl FromStr for Level {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "E" => Ok(Level::E),
            "D" => Ok(Level::D),
            "C" => Ok(Level::C),
            "B" => Ok(Level::B),
            "A" => Ok(Level::A),
            "S" => Ok(Level::S),
            _ => Err(()),
        }
    }
}

/// Represents the dance style (Standard or Latin).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Style {
    Standard,
    Latein,
}

impl Style {
    /// Creates a Style from an ID string.
    pub fn from_id(id: &str) -> Option<Self> {
        Self::from_str(id).ok()
    }
}

impl FromStr for Style {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "std" | "standard" => Ok(Style::Standard),
            "lat" | "latin" | "latein" => Ok(Style::Latein),
            _ => Err(()),
        }
    }
}

/// Represents an individual dance.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

impl AgeGroup {
    /// Creates an AgeGroup from an ID string.
    pub fn from_id(id: &str) -> Option<Self> {
        Self::from_str(id).ok()
    }
}

impl FromStr for AgeGroup {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "juv_1" => Ok(AgeGroup::Juv1),
            "juv_2" => Ok(AgeGroup::Juv2),
            "jun_1" => Ok(AgeGroup::Jun1),
            "jun_2" => Ok(AgeGroup::Jun2),
            "youth" => Ok(AgeGroup::Youth),
            "adult" => Ok(AgeGroup::Adult),
            "adult_2" => Ok(AgeGroup::Adult2),
            "sen_1" => Ok(AgeGroup::Sen1),
            "sen_2" => Ok(AgeGroup::Sen2),
            "sen_3" => Ok(AgeGroup::Sen3),
            "sen_4" => Ok(AgeGroup::Sen4),
            "sen_5" => Ok(AgeGroup::Sen5),
            "senior" => Ok(AgeGroup::Senior),
            _ => Err(()),
        }
    }
}

/// Represents a judge in a competition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Judge {
    pub code: String,
    pub name: String,
    pub club: Option<String>,
}

/// Represents a committee member (e.g., Chairperson, Scrutineer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeMember {
    pub name: String,
    pub club: Option<String>,
}

/// Contains all officials responsible for a competition.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub identity_type: IdentityType,
    pub name_one: String,
    pub bib_number: u32,
    pub name_two: Option<String>,
    pub affiliation: Option<String>,
    pub final_rank: Option<u32>,
}

/// Detailed scores for WDSF competitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WDSFScore {
    pub technical_quality: f64,
    pub movement_to_music: f64,
    pub partnering_skills: f64,
    pub choreography: f64,
}

/// Represents a round in a competition with its associated results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Round {
    pub name: String,
    pub marking_crosses: Option<HashMap<String, HashMap<u32, HashMap<Dance, bool>>>>,
    pub dtv_ranks: Option<HashMap<String, HashMap<u32, HashMap<Dance, u32>>>>,
    pub wdsf_scores: Option<HashMap<String, HashMap<u32, WDSFScore>>>,
}

/// A specific contest within an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Competition {
    pub level: Level,
    pub age_group: AgeGroup,
    pub style: Style,
    pub dances: Vec<Dance>,
    pub min_dances: u32,
    pub officials: Officials,
    pub participants: Vec<Participant>,
    pub rounds: Vec<Round>,
}

/// A high-level container for a series of competitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub date: Option<chrono::NaiveDate>,
    pub organizer: Option<String>,
    pub hosting_club: Option<String>,
    pub competitions_list: Vec<Competition>,
}
