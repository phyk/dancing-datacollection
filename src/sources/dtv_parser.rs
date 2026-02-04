use crate::i18n::I18n;
use crate::models::{
    CommitteeMember, Competition, Dance, Event, IdentityType, Judge, Level, Officials, Participant,
};
use crate::scraper::Config;
use crate::sources::{ParsingError, ResultSource};
use chrono::NaiveDate;
use regex::Regex;
use scraper::{Html, Selector};

/// Configuration for CSS selectors used by the DTV parser.
#[derive(Clone, Debug)]
pub struct SelectorConfig {
    pub event_name: String,
    pub event_date: String,
    pub organizer: String,
    pub hosting_club: String,
    pub competition_item: String,
    pub competition_title: String,
    pub participant_row: String,
    pub participant_cell_rank: String,
    pub participant_cell_data: String,
    pub official_row: String,
    pub official_cell_role: String,
    pub official_cell_data: String,
}

impl Default for SelectorConfig {
    fn default() -> Self {
        Self {
            event_name: ".eventhead td".to_string(),
            event_date: ".comphead".to_string(),
            organizer: ".organizer".to_string(),
            hosting_club: ".hosting-club".to_string(),
            competition_item: "center a".to_string(),
            competition_title: ".compbutton".to_string(),
            participant_row: "table.tab1 tr, table.tab2 tr".to_string(),
            participant_cell_rank: "td.td3r".to_string(),
            participant_cell_data: "td.td5, td.td6".to_string(),
            official_row: "table.tab1 tr".to_string(),
            official_cell_role: "td.td2, td.td2r".to_string(),
            official_cell_data: "td.td5".to_string(),
        }
    }
}

/// Parser for DTV (German Dance Sport Federation) competition results.
pub struct DtvParser {
    pub config: Config,
    pub selectors: SelectorConfig,
    pub i18n: I18n,
}

impl DtvParser {
    /// Creates a new DtvParser.
    pub fn new(config: Config, selectors: SelectorConfig, i18n: I18n) -> Self {
        Self {
            config,
            selectors,
            i18n,
        }
    }

    fn parse_date(&self, s: &str) -> Option<NaiveDate> {
        let s = s.trim();

        // Regex for DD.MM.YYYY
        let re_dots = Regex::new(r"(\d{2})\.(\d{2})\.(\d{4})").unwrap();
        if let Some(caps) = re_dots.captures(s) {
            let d = caps[1].parse::<u32>().ok()?;
            let m = caps[2].parse::<u32>().ok()?;
            let y = caps[3].parse::<i32>().ok()?;
            return NaiveDate::from_ymd_opt(y, m, d);
        }

        // Regex for DD/Mon/YYYY (e.g. 17/May/2025)
        let re_slashes = Regex::new(r"(\d{2})/([a-zA-Z]{3})/(\d{4})").unwrap();
        if let Some(caps) = re_slashes.captures(s) {
            let d = caps[1].parse::<u32>().ok()?;
            let mon_str = &caps[2];
            let y = caps[3].parse::<i32>().ok()?;
            let m = match mon_str.to_lowercase().as_str() {
                "jan" => 1,
                "feb" => 2,
                "mar" => 3,
                "apr" => 4,
                "may" => 5,
                "jun" => 6,
                "jul" => 7,
                "aug" => 8,
                "sep" => 9,
                "oct" => 10,
                "nov" => 11,
                "dec" => 12,
                _ => return None,
            };
            return NaiveDate::from_ymd_opt(y, m, d);
        }

        None
    }

    fn parse_dances(&self, s: &str) -> Vec<Dance> {
        let mut dances = Vec::new();
        let s_up = s.to_uppercase();

        // Standard
        if s_up.contains("SW") || s_up.contains("LW") || s_up.contains("WALZER") {
            dances.push(Dance::SlowWaltz);
        }
        if s_up.contains("TG") || s_up.contains("TANGO") {
            dances.push(Dance::Tango);
        }
        if s_up.contains("VW") || s_up.contains("WIENER") {
            dances.push(Dance::VienneseWaltz);
        }
        if s_up.contains("SF") || s_up.contains("SLOW") || s_up.contains("FOX") {
            dances.push(Dance::SlowFoxtrot);
        }
        if s_up.contains("QS") || s_up.contains("QU") || s_up.contains("QUICK") {
            dances.push(Dance::Quickstep);
        }

        // Latin
        if dances.is_empty() {
            if s_up.contains("CC") || s_up.contains("CHA") {
                dances.push(Dance::ChaChaCha);
            }
            if s_up.contains("SB") || s_up.contains("SA") || s_up.contains("SAMBA") {
                dances.push(Dance::Samba);
            }
            if s_up.contains("RB") || s_up.contains("RU") || s_up.contains("RUMBA") {
                dances.push(Dance::Rumba);
            }
            if s_up.contains("PD") || s_up.contains("PASO") {
                dances.push(Dance::PasoDoble);
            }
            if s_up.contains("JV") || s_up.contains("JI") || s_up.contains("JIVE") {
                dances.push(Dance::Jive);
            }
        }

        dances
    }

    pub fn parse_participants(&self, html: &str) -> Result<Vec<Participant>, ParsingError> {
        let fragment = Html::parse_document(html);
        let row_sel = Selector::parse(&self.selectors.participant_row).unwrap();
        let rank_sel = Selector::parse(&self.selectors.participant_cell_rank).unwrap();
        let data_sel = Selector::parse(&self.selectors.participant_cell_data).unwrap();

        let mut participants = Vec::new();
        let name_bib_re = Regex::new(r"(?s)^(.*?)\s*\((\d+)\)$").unwrap();

        for row in fragment.select(&row_sel) {
            let rank_text = row
                .select(&rank_sel)
                .next()
                .map(|e| e.text().collect::<Vec<_>>().join(" ").trim().to_string());

            if rank_text.is_none() {
                continue;
            }
            let rank_text = rank_text.unwrap();
            let final_rank = rank_text
                .split('.')
                .next()
                .and_then(|s| s.split('-').next())
                .and_then(|s| s.trim().parse::<u32>().ok());

            let mut data_iter = row.select(&data_sel);
            if let Some(td5) = data_iter.next() {
                let full_text = td5.text().collect::<Vec<_>>().join(" ").trim().to_string();

                // DTV sometimes puts club in <i> inside td5, or in td6
                let mut club = td5
                    .select(&Selector::parse("i").unwrap())
                    .next()
                    .map(|e| e.text().collect::<Vec<_>>().join(" ").trim().to_string());

                if club.is_none() {
                    if let Some(td6) = data_iter.next() {
                        club = Some(td6.text().collect::<Vec<_>>().join(" ").trim().to_string());
                    }
                }

                // Remove club text from full_text if it was inside td5
                let name_bib_text = if let Some(ref c) = club {
                    full_text.replace(c, "").trim().to_string()
                } else {
                    full_text
                };

                if let Some(caps) = name_bib_re.captures(&name_bib_text) {
                    let names_part = caps[1].trim();
                    let bib_number = caps[2].parse::<u32>().unwrap_or(0);

                    let (identity_type, name_one, name_two) = if names_part.contains(" / ") {
                        let parts: Vec<&str> = names_part.split(" / ").collect();
                        (
                            IdentityType::Couple,
                            parts[0].trim().to_string(),
                            Some(parts[1].trim().to_string()),
                        )
                    } else {
                        (IdentityType::Solo, names_part.to_string(), None)
                    };

                    participants.push(Participant {
                        identity_type,
                        name_one,
                        bib_number,
                        name_two,
                        affiliation: club,
                        final_rank,
                    });
                }
            }
        }

        Ok(participants)
    }

    pub fn parse_officials(&self, html: &str) -> Result<Officials, ParsingError> {
        let fragment = Html::parse_document(html);
        let row_sel = Selector::parse(&self.selectors.official_row).unwrap();
        let role_sel = Selector::parse(&self.selectors.official_cell_role).unwrap();
        let data_sel = Selector::parse(&self.selectors.official_cell_data).unwrap();

        let mut officials = Officials {
            responsible_person: None,
            assistant: None,
            judges: Vec::new(),
        };

        let span_sel = Selector::parse("span").unwrap();
        for row in fragment.select(&row_sel) {
            let role_text = row
                .select(&role_sel)
                .next()
                .map(|e| e.text().collect::<Vec<_>>().join(" ").trim().replace(":", ""));

            if let Some(role) = role_text {
                if let Some(td5) = row.select(&data_sel).next() {
                    let mut spans = td5.select(&span_sel);
                    let name = spans
                        .next()
                        .map(|e| e.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        .filter(|s| !s.is_empty());
                    let club = spans
                        .next()
                        .map(|e| e.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        .filter(|s| !s.is_empty());

                    if let Some(n) = name {
                        if let Some(canonical_role) = self.i18n.map_role(&role) {
                            let member = CommitteeMember {
                                name: n,
                                club: club.clone(),
                            };
                            match canonical_role.as_str() {
                                "responsible_person" => officials.responsible_person = Some(member),
                                "assistant" => officials.assistant = Some(member),
                                _ => {}
                            }
                        } else if (role.len() == 1 || role.len() == 2)
                            && role.chars().all(|c| c.is_ascii_uppercase())
                        {
                            officials.judges.push(Judge {
                                code: role,
                                name: n,
                                club,
                            });
                        }
                    }
                }
            }
        }

        if officials.responsible_person.is_none()
            && officials.assistant.is_none()
            && officials.judges.is_empty()
        {
            return Err(ParsingError::ValidationError("MissingOfficial".to_string()));
        }

        Ok(officials)
    }
}

impl ResultSource for DtvParser {
    fn name(&self) -> &str {
        "DTV"
    }

    fn fetch(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let resp = reqwest::blocking::get(url)?;
        Ok(resp.text()?)
    }

    fn parse(&self, html: &str) -> Result<Event, ParsingError> {
        let fragment = Html::parse_document(html);

        let name_sel = Selector::parse(&self.selectors.event_name).unwrap();
        let event_name = fragment
            .select(&name_sel)
            .next()
            .map(|e| e.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .ok_or_else(|| ParsingError::MissingRequiredData("Event Name".to_string()))?;

        let date_sel = Selector::parse(&self.selectors.event_date).unwrap();
        let date_text = fragment
            .select(&date_sel)
            .next()
            .map(|e| e.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .ok_or_else(|| ParsingError::MissingRequiredData("Event Date".to_string()))?;

        let event_date = self.parse_date(&date_text).ok_or_else(|| {
            ParsingError::MissingRequiredData("Event Date (invalid format)".to_string())
        })?;

        let org_sel = Selector::parse(&self.selectors.organizer).unwrap();
        let organizer = fragment
            .select(&org_sel)
            .next()
            .map(|e| e.text().collect::<Vec<_>>().join(" ").trim().to_string());

        let host_sel = Selector::parse(&self.selectors.hosting_club).unwrap();
        let hosting_club = fragment
            .select(&host_sel)
            .next()
            .map(|e| e.text().collect::<Vec<_>>().join(" ").trim().to_string());

        let mut sorted_age_keys: Vec<_> = self.i18n.aliases.age_groups.keys().collect();
        sorted_age_keys.sort_by_key(|k| k.len());
        sorted_age_keys.reverse();

        let mut sorted_disc_keys: Vec<_> = self.i18n.aliases.dances.keys().collect();
        sorted_disc_keys.sort_by_key(|k| k.len());
        sorted_disc_keys.reverse();

        let mut competitions = Vec::new();
        let item_sel = Selector::parse(&self.selectors.competition_item).unwrap();
        let title_sel = Selector::parse(&self.selectors.competition_title).unwrap();

        for item in fragment.select(&item_sel) {
            if let Some(title_elem) = item.select(&title_sel).next() {
                let title = title_elem
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string();

                let mut age_group = None;
                let mut style = None;
                let mut level = None;

                for key in &sorted_age_keys {
                    if title.contains(*key) {
                        age_group = self.i18n.map_age_group(key);
                        break;
                    }
                }

                for key in &sorted_disc_keys {
                    if title.contains(*key) {
                        style = self.i18n.map_discipline(key);
                        break;
                    }
                }

                for l_id in ["S", "A", "B", "C", "D", "E"] {
                    let pattern = format!(" {} ", l_id);
                    let pattern_comma = format!(" {},", l_id);
                    if title.contains(&pattern)
                        || title.contains(&pattern_comma)
                        || title.ends_with(&format!(" {}", l_id))
                    {
                        level = Level::from_id(l_id);
                        break;
                    }
                }

                if level.is_none() && (title.contains("WDSF") || title.contains("Open")) {
                    level = Some(Level::S);
                }

                if age_group.is_none() || style.is_none() || level.is_none() {
                    log::warn!("Incomplete metadata for competition: {}", title);
                    continue;
                }

                let age_group = age_group.unwrap();
                let style = style.unwrap();
                let level = level.unwrap();
                let dances = self.parse_dances(&title);
                let min_dances = self.config.get_min_dances(&level, &event_date);

                // Fidelity Gate: Structure Check
                if (dances.len() as u32) < min_dances {
                    log::error!(
                        "PARSING_ERROR: Competition level {:?} requires {} dances but only {} found in '{}'",
                        level,
                        min_dances,
                        dances.len(),
                        title
                    );
                    // According to spec, we should log PARSING_ERROR and NOT save.
                    // We skip this competition.
                    continue;
                }

                competitions.push(Competition {
                    level,
                    age_group,
                    style,
                    dances,
                    min_dances,
                    officials: Officials {
                        responsible_person: None,
                        assistant: None,
                        judges: Vec::new(),
                    },
                    participants: Vec::new(),
                    rounds: Vec::new(),
                });
            }
        }

        // Fidelity Gate: A competition is invalid if it lacks Officials, Judges, or Results.
        // NOTE: Since this index parser only bootstraps competitions, we can't fully enforce
        // Officials/Judges/Results here. This check should happen after full scraping.
        // However, we can check if we found ANY competitions.
        if competitions.is_empty() {
            return Err(ParsingError::MissingRequiredData(
                "No valid competitions found in event index".to_string(),
            ));
        }

        Ok(Event {
            name: event_name,
            organizer,
            hosting_club: hosting_club,
            competitions_list: competitions,
            date: Some(event_date),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AgeGroup, Level, Style};
    use std::collections::HashMap;

    #[test]
    fn test_parse_index_page() {
        let html = r#"
            <html>
            <body>
                <div class="eventhead">
                    <table><tr><td>Hessen tanzt 2025</td></tr></table>
                </div>
                <div class="organizer">Hessischer Tanzsportverband</div>
                <div class="hosting-club">TC Der Frankfurter Kreis</div>
                <div class="maincontainer">
                    <div class="comphead">On 16/May/2025 till 18/May/2025 in Frankfurt am Main.</div>
                    <center>
                        <a href="52-1705_wdsfintopenstdadult/index.htm"><span class="compbutton">17/May, WDSF INT. OPEN Standard Adult (SW, TG, VW, SF, QS)</span></a>
                        <a href="67-1805_ot_hgrdstd/index.htm"><span class="compbutton">18/May, OT, Hgr. D Standard (SW, TG, QS)</span></a>
                        <a href="fake/index.htm"><span class="compbutton">18/May, OT, Sen.III S Latein (SA, CC, RB, PD, JV)</span></a>
                    </center>
                </div>
            </body>
            </html>
        "#;

        let config_str = r#"
            [sources]
            urls = []
            [levels.D]
            min_dances_legacy = 3
            min_dances_2026 = 4
            [levels.S]
            min_dances = 5
        "#;
        let config: Config = toml::from_str(config_str).unwrap();

        let aliases_content = r#"
            [age_groups]
            "Adult" = "adult"
            "Hgr." = "adult"
            "Sen.III" = "sen_3"
            [dances]
            "Standard" = "std"
            "Latein" = "lat"
        "#;
        let aliases: crate::i18n::Aliases = toml::from_str(aliases_content).unwrap();
        let i18n = I18n { aliases };

        let parser = DtvParser::new(config, SelectorConfig::default(), i18n);
        let event = parser.parse(html).unwrap();

        assert_eq!(event.name, "Hessen tanzt 2025");
        assert_eq!(
            event.organizer.unwrap(),
            "Hessischer Tanzsportverband".to_string()
        );
        assert_eq!(
            event.hosting_club.unwrap(),
            "TC Der Frankfurter Kreis".to_string()
        );
        assert_eq!(event.competitions_list.len(), 3);

        // 1st comp: 17/May, WDSF INT. OPEN Standard Adult
        let c1 = &event.competitions_list[0];
        assert_eq!(c1.age_group, AgeGroup::Adult);
        assert_eq!(c1.style, Style::Standard);
        assert_eq!(c1.level, Level::S);
        assert_eq!(c1.min_dances, 5);
        assert_eq!(c1.dances.len(), 5);
        assert!(c1.dances.contains(&Dance::SlowWaltz));
        assert!(c1.dances.contains(&Dance::Tango));
        assert!(c1.dances.contains(&Dance::VienneseWaltz));
        assert!(c1.dances.contains(&Dance::SlowFoxtrot));
        assert!(c1.dances.contains(&Dance::Quickstep));

        // 2nd comp: 18/May, OT, Hgr. D Standard
        let c2 = &event.competitions_list[1];
        assert_eq!(c2.age_group, AgeGroup::Adult);
        assert_eq!(c2.style, Style::Standard);
        assert_eq!(c2.level, Level::D);
        assert_eq!(c2.min_dances, 3); // 2025 is legacy
        assert_eq!(c2.dances.len(), 3);
        assert!(c2.dances.contains(&Dance::SlowWaltz));
        assert!(c2.dances.contains(&Dance::Tango));
        assert!(c2.dances.contains(&Dance::Quickstep));

        // 3rd comp: 18/May, OT, Sen.III S Latein (SA, CC, RB, PD, JV)
        let c3 = &event.competitions_list[2];
        assert_eq!(c3.age_group, AgeGroup::Sen3);
        assert_eq!(c3.style, Style::Latein);
        assert_eq!(c3.level, Level::S);
        assert_eq!(c3.dances.len(), 5);
        assert!(c3.dances.contains(&Dance::Samba));
        assert!(c3.dances.contains(&Dance::ChaChaCha));
        assert!(c3.dances.contains(&Dance::Rumba));
        assert!(c3.dances.contains(&Dance::PasoDoble));
        assert!(c3.dances.contains(&Dance::Jive));
    }

    #[test]
    fn test_structure_check_failure() {
        let html = r#"
            <html>
            <body>
                <div class="eventhead"><table><tr><td>Test Event</td></tr></table></div>
                <div class="maincontainer">
                    <div class="comphead">On 16/May/2025.</div>
                    <center>
                        <a href="fail/index.htm"><span class="compbutton">Standard S (SW, TG)</span></a>
                    </center>
                </div>
            </body>
            </html>
        "#;
        let config_str = r#"
            [sources]
            urls = []
            [levels.S]
            min_dances = 5
        "#;
        let config: Config = toml::from_str(config_str).unwrap();
        let i18n = I18n {
            aliases: crate::i18n::Aliases {
                age_groups: HashMap::new(),
                dances: HashMap::new(),
                roles: HashMap::new(),
            },
        };
        let parser = DtvParser::new(config, SelectorConfig::default(), i18n);
        let event = parser.parse(html).unwrap();
        // Should have 0 competitions because Standard S needs 5 dances but only 2 were found
        assert_eq!(event.competitions_list.len(), 0);
        assert_eq!(
            event.date.unwrap(),
            NaiveDate::from_ymd_opt(2024, 5, 10).unwrap()
        );
    }

    #[test]
    fn test_parse_participants() {
        let html = r#"
            <TABLE class="tab1">
                <TR><TD class="td3r">1.</TD>
                    <TD class="td5">Jonathan Kummetz / Elisabeth Findeiß (610)<BR><i>1. TC Rot-Gold Bayreuth</i></TD>
                </TR>
                <TR><TD class="td3r">2.</TD>
                    <TD class="td5">Konstantin Plöger / Laura Utz (616)<BR><i>TSZ Blau-Gold Casino, Darmstadt</i></TD>
                </TR>
            </TABLE>
            <TABLE class="tab2">
                <TR><TD class="td3r">7.</TD>
                    <TD class="td5">Thilo Schmid / Katharina Zierer (621)</TD>
                    <TD class="td6">Dance Unlimited</TD>
                </TR>
                <TR><TD class="td3r">10.- 12.</TD>
                    <TD class="td5">Solo Dancer (123)</TD>
                    <TD class="td6">Solo Club</TD>
                </TR>
            </TABLE>
        "#;

        let i18n = I18n {
            aliases: crate::i18n::Aliases {
                age_groups: HashMap::new(),
                dances: HashMap::new(),
                roles: HashMap::new(),
            },
        };
        let config = Config {
            sources: crate::scraper::Sources { urls: vec![] },
            levels: None,
        };

        let parser = DtvParser::new(config, SelectorConfig::default(), i18n);
        let participants = parser.parse_participants(html).unwrap();

        assert_eq!(participants.len(), 4);

        assert_eq!(participants[0].bib_number, 610);
        assert_eq!(participants[0].name_one, "Jonathan Kummetz");
        assert_eq!(
            participants[0].name_two,
            Some("Elisabeth Findeiß".to_string())
        );
        assert_eq!(participants[0].final_rank, Some(1));
        assert_eq!(
            participants[0].affiliation,
            Some("1. TC Rot-Gold Bayreuth".to_string())
        );
        assert_eq!(participants[0].identity_type, IdentityType::Couple);

        assert_eq!(participants[2].bib_number, 621);
        assert_eq!(participants[2].final_rank, Some(7));
        assert_eq!(
            participants[2].affiliation,
            Some("Dance Unlimited".to_string())
        );

        assert_eq!(participants[3].bib_number, 123);
        assert_eq!(participants[3].name_one, "Solo Dancer");
        assert_eq!(participants[3].name_two, None);
        assert_eq!(participants[3].final_rank, Some(10));
        assert_eq!(participants[3].identity_type, IdentityType::Solo);
    }

    #[test]
    fn test_parse_officials() {
        let html = r#"
            <TABLE class="tab1">
                <TR>
                    <TD class="td2">Turnierleiter:</TD>
                    <TD class="td5"><span class="col1">Jungbluth, Kai</span><span>Tanz-Sport-Club Fischbach</span></TD>
                </TR>
                <TR>
                    <TD class="td2">Beisitzer:</TD>
                    <TD class="td5"><span class="col1">Bittighofer, Mechthild</span><span>Tanz-Freunde Fulda</span></TD>
                </TR>
                <TR>
                    <TD class="td2r">AT:</TD>
                    <TD class="td5"><span class="col1">Bärschneider, Marcus</span><span>TSC Blau-Gelb Hagen</span></TD>
                </TR>
                <TR>
                    <TD class="td2r">AX:</TD>
                    <TD class="td5"><span class="col1">Block, Robert</span><span>Schwarz-Rot-Club Wetzlar</span></TD>
                </TR>
                <TR>
                    <TD class="td2r">A:</TD>
                    <TD class="td5"><span class="col1">Single, Letter</span><span>Club Single</span></TD>
                </TR>
            </TABLE>
        "#;

        let mut roles = HashMap::new();
        roles.insert("Turnierleiter".to_string(), "responsible_person".to_string());
        roles.insert("Beisitzer".to_string(), "assistant".to_string());

        let i18n = I18n {
            aliases: crate::i18n::Aliases {
                age_groups: HashMap::new(),
                dances: HashMap::new(),
                roles,
            },
        };
        let config = Config {
            sources: crate::scraper::Sources { urls: vec![] },
            levels: None,
        };

        let parser = DtvParser::new(config, SelectorConfig::default(), i18n);
        let officials = parser.parse_officials(html).unwrap();

        assert!(officials.responsible_person.is_some());
        assert_eq!(officials.responsible_person.as_ref().unwrap().name, "Jungbluth, Kai");
        assert_eq!(
            officials.responsible_person.as_ref().unwrap().club,
            Some("Tanz-Sport-Club Fischbach".to_string())
        );

        assert!(officials.assistant.is_some());
        assert_eq!(officials.assistant.as_ref().unwrap().name, "Bittighofer, Mechthild");

        assert_eq!(officials.judges.len(), 3);
        assert_eq!(officials.judges[0].code, "AT");
        assert_eq!(officials.judges[0].name, "Bärschneider, Marcus");
        assert_eq!(
            officials.judges[0].club,
            Some("TSC Blau-Gelb Hagen".to_string())
        );
        assert_eq!(officials.judges[1].code, "AX");
        assert_eq!(officials.judges[2].code, "A");
        assert_eq!(officials.judges[2].name, "Single, Letter");
    }

    #[test]
    fn test_parse_officials_validation() {
        let i18n = I18n {
            aliases: crate::i18n::Aliases {
                age_groups: HashMap::new(),
                dances: HashMap::new(),
                roles: HashMap::new(),
            },
        };
        let config = Config {
            sources: crate::scraper::Sources { urls: vec![] },
            levels: None,
        };
        let parser = DtvParser::new(config, SelectorConfig::default(), i18n);

        let res = parser.parse_officials("<html><body><table></table></body></html>");
        assert!(res.is_err());
        match res.unwrap_err() {
            ParsingError::ValidationError(msg) => assert_eq!(msg, "MissingOfficial"),
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_min_dances_2026() {
        let config_str = r#"
            [sources]
            urls = []
            [levels.D]
            min_dances_legacy = 3
            min_dances_2026 = 4
        "#;
        let config: Config = toml::from_str(config_str).unwrap();

        let d2025 = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let d2026 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        assert_eq!(config.get_min_dances(&Level::D, &d2025), 3);
        assert_eq!(config.get_min_dances(&Level::D, &d2026), 4);
    }
}
