use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Level {
    E,
    D,
    C,
    B,
    A,
    S,
}

#[pymethods]
impl Level {
    fn __hash__(&self) -> isize {
        *self as isize
    }

    #[staticmethod]
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

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Style {
    Standard,
    Latein,
}

#[pymethods]
impl Style {
    fn __hash__(&self) -> isize {
        *self as isize
    }

    #[staticmethod]
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

#[pyclass(eq, eq_int)]
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

#[pymethods]
impl Dance {
    fn __hash__(&self) -> isize {
        *self as isize
    }
}

#[pyclass(eq, eq_int)]
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

#[pymethods]
impl AgeGroup {
    fn __hash__(&self) -> isize {
        *self as isize
    }

    #[staticmethod]
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

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Judge {
    #[pyo3(get, set)]
    pub code: String,
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub club: Option<String>,
}

#[pymethods]
impl Judge {
    #[new]
    #[pyo3(signature = (code, name, club=None))]
    pub fn new(code: String, name: String, club: Option<String>) -> Self {
        Self { code, name, club }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeMember {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub club: Option<String>,
}

#[pymethods]
impl CommitteeMember {
    #[new]
    #[pyo3(signature = (name, club=None))]
    pub fn new(name: String, club: Option<String>) -> Self {
        Self { name, club }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Officials {
    #[pyo3(get, set)]
    pub responsible_person: Option<CommitteeMember>,
    #[pyo3(get, set)]
    pub assistant: Option<CommitteeMember>,
    #[pyo3(get, set)]
    pub judges: Vec<Judge>,
}

#[pymethods]
impl Officials {
    #[new]
    #[pyo3(signature = (responsible_person=None, assistant=None, judges=Vec::new()))]
    pub fn new(
        responsible_person: Option<CommitteeMember>,
        assistant: Option<CommitteeMember>,
        judges: Vec<Judge>,
    ) -> Self {
        Self {
            responsible_person,
            assistant,
            judges,
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum IdentityType {
    Solo,
    Couple,
}

#[pymethods]
impl IdentityType {
    fn __hash__(&self) -> isize {
        *self as isize
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    #[pyo3(get, set)]
    pub identity_type: IdentityType,
    #[pyo3(get, set)]
    pub name_one: String,
    #[pyo3(get, set)]
    pub name_two: Option<String>,
    #[pyo3(get, set)]
    pub affiliation: Option<String>,
    #[pyo3(get, set)]
    pub bib_number: u32,
    #[pyo3(get, set)]
    pub final_rank: Option<u32>,
}

#[pymethods]
impl Participant {
    #[new]
    #[pyo3(signature = (identity_type, name_one, bib_number, name_two=None, affiliation=None, final_rank=None))]
    pub fn new(
        identity_type: IdentityType,
        name_one: String,
        bib_number: u32,
        name_two: Option<String>,
        affiliation: Option<String>,
        final_rank: Option<u32>,
    ) -> Self {
        Self {
            identity_type,
            name_one,
            name_two,
            affiliation,
            bib_number,
            final_rank,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WDSFScore {
    #[pyo3(get, set)]
    pub technical_quality: f64,
    #[pyo3(get, set)]
    pub movement_to_music: f64,
    #[pyo3(get, set)]
    pub partnering_skills: f64,
    #[pyo3(get, set)]
    pub choreography: f64,
}

#[pymethods]
impl WDSFScore {
    #[new]
    pub fn new(
        technical_quality: f64,
        movement_to_music: f64,
        partnering_skills: f64,
        choreography: f64,
    ) -> Self {
        Self {
            technical_quality,
            movement_to_music,
            partnering_skills,
            choreography,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Round {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub marking_crosses: Option<HashMap<String, HashMap<u32, HashMap<Dance, bool>>>>,
    #[pyo3(get, set)]
    pub dtv_ranks: Option<HashMap<String, HashMap<u32, HashMap<Dance, u32>>>>,
    #[pyo3(get, set)]
    pub wdsf_scores: Option<HashMap<String, HashMap<u32, WDSFScore>>>,
}

#[pymethods]
impl Round {
    #[new]
    #[pyo3(signature = (name, marking_crosses=None, dtv_ranks=None, wdsf_scores=None))]
    pub fn new(
        name: String,
        marking_crosses: Option<HashMap<String, HashMap<u32, HashMap<Dance, bool>>>>,
        dtv_ranks: Option<HashMap<String, HashMap<u32, HashMap<Dance, u32>>>>,
        wdsf_scores: Option<HashMap<String, HashMap<u32, WDSFScore>>>,
    ) -> Self {
        Self {
            name,
            marking_crosses,
            dtv_ranks,
            wdsf_scores,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Competition {
    #[pyo3(get, set)]
    pub level: Level,
    #[pyo3(get, set)]
    pub age_group: AgeGroup,
    #[pyo3(get, set)]
    pub style: Style,
    #[pyo3(get, set)]
    pub dances: Vec<Dance>,
    #[pyo3(get, set)]
    pub officials: Officials,
    #[pyo3(get, set)]
    pub participants: Vec<Participant>,
    #[pyo3(get, set)]
    pub rounds: Vec<Round>,
}

#[pymethods]
impl Competition {
    #[new]
    pub fn new(
        level: Level,
        age_group: AgeGroup,
        style: Style,
        dances: Vec<Dance>,
        officials: Officials,
        participants: Vec<Participant>,
        rounds: Vec<Round>,
    ) -> Self {
        Self {
            level,
            age_group,
            style,
            dances,
            officials,
            participants,
            rounds,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub organizer: Option<String>,
    #[pyo3(get, set)]
    pub hosting_club: Option<String>,
    #[pyo3(get, set)]
    pub competitions_list: Vec<Competition>,
}

#[pymethods]
impl Event {
    #[new]
    #[pyo3(signature = (name, organizer=None, hosting_club=None, competitions_list=Vec::new()))]
    pub fn new(
        name: String,
        organizer: Option<String>,
        hosting_club: Option<String>,
        competitions_list: Vec<Competition>,
    ) -> Self {
        Self {
            name,
            organizer,
            hosting_club,
            competitions_list,
        }
    }
}
