use crate::i18n::I18n;
use crate::models::{Competition, Dance, Event, Level, Officials};
use crate::scraper::Config;
use crate::sources::{ParsingError, ResultSource};
use chrono::NaiveDate;
use regex::Regex;
use scraper::{Html, Selector};

#[derive(Clone, Debug)]
pub struct SelectorConfig {
    pub event_name: String,
    pub event_date: String,
    pub organizer: String,
    pub hosting_club: String,
    pub competition_item: String,
    pub competition_title: String,
}

impl Default for SelectorConfig {
    fn default() -> Self {
        Self {
            event_name: ".eventhead td".to_string(),
            event_date: ".comphead".to_string(),
            organizer: ".organizer".to_string(), // Placeholder
            hosting_club: ".hosting-club".to_string(), // Placeholder
            competition_item: "center a".to_string(),
            competition_title: ".compbutton".to_string(),
        }
    }
}

pub struct DtvParser {
    pub config: Config,
    pub selectors: SelectorConfig,
    pub i18n: I18n,
}

impl DtvParser {
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

        // Latin (if still empty or if explicitly Latin)
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
}

impl ResultSource for DtvParser {
    fn parse(&self, html: &str) -> Result<Event, ParsingError> {
        let fragment = Html::parse_document(html);

        let name_sel = Selector::parse(&self.selectors.event_name).unwrap();
        let event_name = fragment
            .select(&name_sel)
            .next()
            .map(|e| e.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .ok_or_else(|| ParsingError::MissingRequiredMetadata("Event Name".to_string()))?;

        let date_sel = Selector::parse(&self.selectors.event_date).unwrap();
        let date_text = fragment
            .select(&date_sel)
            .next()
            .map(|e| e.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .ok_or_else(|| ParsingError::MissingRequiredMetadata("Event Date".to_string()))?;

        let event_date = self.parse_date(&date_text).ok_or_else(|| {
            ParsingError::MissingRequiredMetadata("Event Date (invalid format)".to_string())
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

        let mut sorted_disc_keys: Vec<_> = self.i18n.aliases.disciplines.keys().collect();
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

        Ok(Event {
            name: event_name,
            date: Some(event_date),
            organizer,
            hosting_club,
            competitions_list: competitions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AgeGroup, Level, Style};

    #[test]
    fn test_parse_index_page() {
        let html = r#"
            <html>
            <body>
                <div class="eventhead">
                    <table><tr><td>Hessen tanzt 2025</td></tr></table>
                </div>
                <div class="maincontainer">
                    <div class="comphead">On 16/May/2025 till 18/May/2025 in Frankfurt am Main.</div>
                    <center>
                        <a href="52-1705_wdsfintopenstdadult/index.htm"><span class="compbutton">17/May, WDSF INT. OPEN Standard Adult </span></a>
                        <a href="67-1805_ot_hgrdstd/index.htm"><span class="compbutton">18/May, OT, Hgr. D Standard </span></a>
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
            [disciplines]
            "Standard" = "std"
            "Latein" = "lat"
        "#;
        let aliases: crate::i18n::Aliases = toml::from_str(aliases_content).unwrap();
        let i18n = I18n { aliases };

        let parser = DtvParser::new(config, SelectorConfig::default(), i18n);
        let event = parser.parse(html).unwrap();

        assert_eq!(event.name, "Hessen tanzt 2025");
        assert_eq!(
            event.date.unwrap(),
            NaiveDate::from_ymd_opt(2025, 5, 16).unwrap()
        );
        assert_eq!(event.competitions_list.len(), 3);

        // 1st comp: 17/May, WDSF INT. OPEN Standard Adult
        let c1 = &event.competitions_list[0];
        assert_eq!(c1.age_group, AgeGroup::Adult);
        assert_eq!(c1.style, Style::Standard);
        assert_eq!(c1.level, Level::S);
        assert_eq!(c1.min_dances, 5);

        // 2nd comp: 18/May, OT, Hgr. D Standard
        let c2 = &event.competitions_list[1];
        assert_eq!(c2.age_group, AgeGroup::Adult);
        assert_eq!(c2.style, Style::Standard);
        assert_eq!(c2.level, Level::D);
        assert_eq!(c2.min_dances, 3); // 2025 is legacy

        // 3rd comp: 18/May, OT, Sen.III S Latein (SA, CC, RB, PD, JV)
        let c3 = &event.competitions_list[2];
        assert_eq!(c3.age_group, AgeGroup::Sen3);
        assert_eq!(c3.style, Style::Latein);
        assert_eq!(c3.level, Level::S);
        assert_eq!(c3.dances.len(), 5);
        assert!(c3.dances.contains(&Dance::Samba));
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
