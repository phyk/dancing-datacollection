use crate::i18n::I18n;
use crate::models::{
    CommitteeMember, Competition, Dance, Event, IdentityType, Judge, Level, Officials,
    Participant, Round, WDSFScore,
};
use crate::crawler::client::Config;
use crate::sources::{ParsingError, ResultSource};
use anyhow::Result;
use chrono::NaiveDate;
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

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
            competition_item: "center a, .pro_p_zeile a, .t_zeile a".to_string(),
            competition_title: ".compbutton, b".to_string(),
            participant_row: "table.tab1 tr, table.tab2 tr".to_string(),
            participant_cell_rank: "td.td3r, td.td3c".to_string(),
            participant_cell_data: "td.td2c, td.td5, td.td6".to_string(),
            official_row: "table.tab1 tr".to_string(),
            official_cell_role: "td.td2, td.td2r".to_string(),
            official_cell_data: "td.td5".to_string(),
        }
    }
}

/// Parser for DTV (German Dance Sport Federation) competition results.
pub struct DtvNative {
    pub config: Config,
    pub selectors: SelectorConfig,
    pub i18n: I18n,
}

impl DtvNative {
    /// Creates a new DtvNative parser.
    pub fn new(config: Config, selectors: SelectorConfig, i18n: I18n) -> Self {
        Self {
            config,
            selectors,
            i18n,
        }
    }

    pub fn parse_date(&self, s: &str) -> Option<NaiveDate> {
        let s = s.trim();

        // Regex for DD.MM.YYYY
        let re_dots = Regex::new(r"(\d{1,2})\.(\d{1,2})\.(\d{4})").unwrap();
        if let Some(caps) = re_dots.captures(s) {
            let d = caps[1].parse::<u32>().ok()?;
            let m = caps[2].parse::<u32>().ok()?;
            let y = caps[3].parse::<i32>().ok()?;
            return NaiveDate::from_ymd_opt(y, m, d);
        }

        // Regex for DD/Mon/YYYY (e.g. 05/Jul/2025)
        let re_slashes = Regex::new(r"(\d{1,2})/([a-zA-Z]{3})/(\d{4})").unwrap();
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

        // Handle German month names
        let re_de = Regex::new(r"(\d{1,2})\.\s*([a-zA-Zä]+)\s+(\d{4})").unwrap();
        if let Some(caps) = re_de.captures(s) {
            let d = caps[1].parse::<u32>().ok()?;
            let mon_str = &caps[2];
            let y = caps[3].parse::<i32>().ok()?;
            let m = match mon_str.to_lowercase().as_str() {
                "januar" => 1,
                "februar" => 2,
                "märz" => 3,
                "april" => 4,
                "mai" => 5,
                "juni" => 6,
                "juli" => 7,
                "august" => 8,
                "september" => 9,
                "oktober" => 10,
                "november" => 11,
                "dezember" => 12,
                _ => return None,
            };
            return NaiveDate::from_ymd_opt(y, m, d);
        }

        None
    }

    pub fn parse_dances(&self, s: &str) -> Vec<Dance> {
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
        if s_up.contains("VW") || s_up.contains("WW") || s_up.contains("WIENER") {
            dances.push(Dance::VienneseWaltz);
        }
        if (s_up.contains("SF") && !s_up.contains("WDSF")) || s_up.contains("SLOW") || s_up.contains("FOX") {
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

        // Fallback for broad disciplines
        if dances.is_empty() {
             if s_up.contains("STANDARD") {
                 dances = vec![Dance::SlowWaltz, Dance::Tango, Dance::VienneseWaltz, Dance::SlowFoxtrot, Dance::Quickstep];
             } else if s_up.contains("LATEIN") || s_up.contains("LATIN") {
                 dances = vec![Dance::Samba, Dance::ChaChaCha, Dance::Rumba, Dance::PasoDoble, Dance::Jive];
             }
        }

        dances
    }

    pub fn parse_dances_from_table(&self, html: &str) -> Vec<Dance> {
        let fragment = Html::parse_document(html);
        let td_sel = Selector::parse("td.td2cw, td.td2ww").unwrap();
        let mut dances = Vec::new();
        for td in fragment.select(&td_sel) {
            let text = td.text().collect::<String>().trim().to_string();
            let d = self.parse_dances(&text);
            if !d.is_empty() {
                dances.push(d[0]);
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

            let mut bib_number = 0;
            let mut name_text = String::new();
            let mut club = None;

            let data_cells: Vec<_> = row.select(&data_sel).collect();

            // Check if bib is in separate cell (usually td2c)
            if data_cells.len() >= 2 {
                 let first_cell_text = data_cells[0].text().collect::<String>().trim().to_string();
                 if let Ok(bib) = first_cell_text.parse::<u32>() {
                      bib_number = bib;
                      name_text = data_cells[1].text().collect::<Vec<_>>().join(" ").trim().to_string();
                      if data_cells.len() >= 3 {
                           club = Some(data_cells[2].text().collect::<Vec<_>>().join(" ").trim().to_string());
                      }
                 }
            }

            if bib_number == 0 && !data_cells.is_empty() {
                 // Try fallback: bib in parens in name
                 let full_text = data_cells[0].text().collect::<Vec<_>>().join(" ").trim().to_string();

                 club = data_cells[0]
                    .select(&Selector::parse("i").unwrap())
                    .next()
                    .map(|e| e.text().collect::<Vec<_>>().join(" ").trim().to_string());

                 if club.is_none() && data_cells.len() >= 2 {
                      club = Some(data_cells[1].text().collect::<Vec<_>>().join(" ").trim().to_string());
                 }

                 let name_bib_text = if let Some(ref c) = club {
                     full_text.replace(c, "").trim().to_string()
                 } else {
                     full_text
                 };

                 if let Some(caps) = name_bib_re.captures(&name_bib_text) {
                     name_text = caps[1].trim().to_string();
                     bib_number = caps[2].parse::<u32>().unwrap_or(0);
                 } else {
                     name_text = name_bib_text;
                 }
            }

            if !name_text.is_empty() {
                let (identity_type, name_one, name_two) = if name_text.contains(" / ") {
                    let parts: Vec<&str> = name_text.split(" / ").collect();
                    (
                        IdentityType::Couple,
                        parts[0].trim().to_string(),
                        Some(parts[1].trim().to_string()),
                    )
                } else {
                    (IdentityType::Solo, name_text.to_string(), None)
                };

                participants.push(Participant {
                    identity_type,
                    name_one,
                    bib_number,
                    name_two,
                    affiliation: club.filter(|s| !s.is_empty()),
                    final_rank,
                });
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

    pub fn parse_rounds(&self, html: &str, dances: &[Dance]) -> Vec<Round> {
        let document = Html::parse_document(html);
        let h2_sel = Selector::parse("h2").unwrap();
        let comphead_sel = Selector::parse(".comphead").unwrap();

        let mut round_names = Vec::new();
        for h2 in document.select(&h2_sel) {
            let text = h2.text().collect::<Vec<_>>().join(" ").trim().to_string();
            round_names.push(self.canonicalize_round_name(&text).unwrap_or(text));
        }

        if round_names.is_empty() {
             for head in document.select(&comphead_sel) {
                  let text = head.text().collect::<Vec<_>>().join(" ").trim().to_string();
                  if text.to_lowercase().contains("runde") || text.to_lowercase().contains("table") || text.to_lowercase().contains("ergebnis") || text.to_lowercase().contains("ranking") {
                       round_names.push(self.canonicalize_round_name(&text).unwrap_or(text));
                  }
             }

             let td_sel = Selector::parse("td.td1, td.td3").unwrap();
             for td in document.select(&td_sel) {
                 let text = td.text().collect::<Vec<_>>().join(" ").trim().to_string();
                 if let Some(name) = self.canonicalize_round_name(&text) {
                      if !round_names.contains(&name) {
                          round_names.push(name);
                      }
                 }
             }
        }

        if round_names.is_empty() {
             round_names.push("Result Table".to_string());
        }

        let mut rounds = Vec::new();
        let marking_results = self.parse_tabges(html, dances);
        let (erg_marks, erg_ranks) = self.parse_ergwert(html, dances);
        let wdsf_results = if html.contains("TQ") || html.contains("MM") {
             Some(self.parse_wdsf_scores(html))
        } else {
             None
        };

        let num_rounds = round_names.len()
             .max(marking_results.len())
             .max(erg_marks.len())
             .max(erg_ranks.len());

        for i in 0..num_rounds {
             let mut round_name = round_names.get(i).cloned();

             let mut marks = marking_results.get(i).map(|r| r.1.clone());
             let mut ranks = None;

             if let Some(r) = erg_marks.get(i) {
                  if round_name.is_none() || round_name.as_ref().unwrap().starts_with("Round") || round_name.as_ref().unwrap().contains("Ergebnis") {
                       if !r.0.is_empty() {
                            round_name = Some(r.0.clone());
                       }
                  }
                  // Merge or prefer detailed marks from ergwert
                  if let Some(ref mut m) = marks {
                       for (judge, bibs) in &r.1 {
                            m.entry(judge.clone()).or_default().extend(bibs.clone());
                       }
                  } else {
                       marks = Some(r.1.clone());
                  }
             }

             if let Some(r) = erg_ranks.get(i) {
                  if round_name.is_none() || round_name.as_ref().unwrap().starts_with("Round") || round_name.as_ref().unwrap().contains("Ergebnis") {
                       if !r.0.is_empty() {
                            round_name = Some(r.0.clone());
                       }
                  }
                  ranks = Some(r.1.clone());
             }

             let name = round_name.unwrap_or_else(|| format!("Round {}", i + 1));

             let mut round = Round {
                  name,
                  marking_crosses: marks,
                  dtv_ranks: ranks,
                  wdsf_scores: None,
             };

             if let Some(ref wdsf) = wdsf_results {
                  if i == 0 { // Assume WDSF results are for the first round detected in the file
                       round.wdsf_scores = Some(wdsf.clone());
                  }
             }
             if round.marking_crosses.is_some() || round.dtv_ranks.is_some() || round.wdsf_scores.is_some() {
                  rounds.push(round);
             }
        }

        rounds
    }

    pub fn parse_tabges(
        &self,
        html: &str,
        dances: &[Dance],
    ) -> Vec<(String, BTreeMap<String, BTreeMap<u32, BTreeMap<Dance, bool>>>)> {
        let mut all_results: Vec<(String, BTreeMap<String, BTreeMap<u32, BTreeMap<Dance, bool>>>)> = Vec::new();
        let document = Html::parse_document(html);
        let tr_sel = Selector::parse("tr").unwrap();
        let td_sel = Selector::parse("td").unwrap();
        let table_sel = Selector::parse("table").unwrap();

        for table in document.select(&table_sel) {
            let mut table_round_idx = 0;
            let mut current_results = BTreeMap::new();
            let mut rows_iter = table.select(&tr_sel);
            let first_row = rows_iter.next();
            if first_row.is_none() { continue; }

            let mut bibs_in_cols = Vec::new();
            let header_cells: Vec<_> = first_row.unwrap().select(&td_sel).collect();
            for cell in &header_cells {
                 let text = cell.text().collect::<String>().trim().to_string();
                 let mut num_str = String::new();
                 for c in text.chars() {
                      if c.is_ascii_digit() { num_str.push(c); } else { break; }
                 }
                 if let Ok(bib) = num_str.parse::<u32>() {
                      bibs_in_cols.push(bib);
                 }
            }

            if bibs_in_cols.is_empty() {
                 if let Some(second_row) = rows_iter.next() {
                      let cells: Vec<_> = second_row.select(&td_sel).collect();
                      for cell in &cells {
                           let text = cell.text().collect::<String>().trim().to_string();
                           let mut num_str = String::new();
                           for c in text.chars() {
                                if c.is_ascii_digit() { num_str.push(c); } else { break; }
                           }
                           if let Ok(bib) = num_str.parse::<u32>() {
                                bibs_in_cols.push(bib);
                           }
                      }
                 }
            }

            if !bibs_in_cols.is_empty() {
                for row in table.select(&tr_sel) {
                    let cells: Vec<_> = row.select(&td_sel).collect();
                    if cells.len() < 2 { continue; }

                    let first_cell_text = cells[0].text().collect::<String>().trim().to_string();
                    let adj_re = Regex::new(r"([A-Z]{1,2})\)").unwrap();
                    let mut adj_codes = Vec::new();
                    for caps in adj_re.captures_iter(&first_cell_text) {
                         adj_codes.push(caps[1].to_string());
                    }

                    if !adj_codes.is_empty() {
                         // Start a new round if we already have results in the current one
                         if !current_results.is_empty() {
                              if table_round_idx < all_results.len() {
                                   for (judge, bibs) in current_results {
                                        all_results[table_round_idx].1.entry(judge).or_default().extend(bibs);
                                   }
                              } else {
                                   all_results.push((format!("Round {}", all_results.len() + 1), current_results));
                              }
                              table_round_idx += 1;
                              current_results = BTreeMap::new();
                         }

                         for (col_idx, bib) in bibs_in_cols.iter().enumerate() {
                              let cell_idx = cells.len() - bibs_in_cols.len() + col_idx;
                              if cell_idx < cells.len() {
                                   let cell_content = cells[cell_idx].inner_html();
                                   let lines: Vec<_> = cell_content.split("<br>").collect();
                                   for (line_idx, adj_code) in adj_codes.iter().enumerate() {
                                        if line_idx < lines.len() {
                                             let val = lines[line_idx].trim();
                                             let has_cross = val.to_lowercase().contains('x') || val.parse::<u32>().unwrap_or(0) > 0;
                                             let bib_map = current_results.entry(adj_code.clone()).or_insert_with(BTreeMap::new)
                                                 .entry(*bib).or_insert_with(BTreeMap::new);
                                             for dance in dances {
                                                  bib_map.insert(*dance, has_cross);
                                             }
                                        }
                                   }
                              }
                         }
                    }

                    if first_cell_text.contains("Ergebnis") || first_cell_text.contains("Result") {
                         for (col_idx, bib) in bibs_in_cols.iter().enumerate() {
                              let cell_idx = cells.len() - bibs_in_cols.len() + col_idx;
                              if cell_idx < cells.len() {
                                   let provided_total: u32 = cells[cell_idx].text().collect::<String>().trim().parse().unwrap_or(0);
                                   let mut calculated_total = 0;
                                   for judge_map in current_results.values() {
                                        if let Some(bib_map) = judge_map.get(bib) {
                                             if bib_map.values().any(|&v| v) {
                                                  calculated_total += 1;
                                             }
                                        }
                                   }
                                   if provided_total > 0 && calculated_total == 0 {
                                        log::warn!("VALIDATION_WARNING: Bib {} has total {} but no crosses parsed", bib, provided_total);
                                   }
                              }
                         }
                    }
                }
                if !current_results.is_empty() {
                     if table_round_idx < all_results.len() {
                          for (judge, bibs) in current_results {
                               all_results[table_round_idx].1.entry(judge).or_default().extend(bibs);
                          }
                     } else {
                          all_results.push((format!("Round {}", all_results.len() + 1), current_results));
                     }
                }
            } else {
                // Horizontal layout
                let mut judge_codes = Vec::new();
                for td in header_cells.iter().skip(2) {
                    let text = td.text().collect::<String>().trim().to_string();
                    if text.len() == 1 || text.len() == 2 {
                        judge_codes.push(text);
                    }
                }

                for row in table.select(&tr_sel).skip(1) {
                    let cells: Vec<_> = row.select(&td_sel).collect();
                    if cells.len() < 3 { continue; }
                    let bib_text = cells[1].text().collect::<String>().trim().to_string();
                    if let Ok(bib) = bib_text.parse::<u32>() {
                        for (i, judge_code) in judge_codes.iter().enumerate() {
                            let cell_idx = 2 + i;
                            if cell_idx < cells.len() {
                                let cross_text = cells[cell_idx].text().collect::<String>();
                                let bib_map = current_results.entry(judge_code.clone()).or_insert_with(BTreeMap::new).entry(bib).or_insert_with(BTreeMap::new);
                                for dance in dances {
                                    bib_map.insert(*dance, cross_text.to_lowercase().contains('x'));
                                }
                            }
                        }
                    }
                }
                if !current_results.is_empty() {
                     // In horizontal layout, we assume one round per table.
                     // But we should still merge if we have multiple tables for the same round.
                     if all_results.is_empty() {
                          all_results.push((format!("Round 1"), current_results));
                     } else {
                          for (judge, bibs) in current_results {
                               all_results[0].1.entry(judge).or_default().extend(bibs);
                          }
                     }
                }
            }
        }
        all_results
    }

    pub fn parse_ergwert(&self, html: &str, dances: &[Dance]) -> (Vec<(String, BTreeMap<String, BTreeMap<u32, BTreeMap<Dance, bool>>>)>, Vec<(String, BTreeMap<String, BTreeMap<u32, BTreeMap<Dance, u32>>>)>) {
        let mut all_rank_results: Vec<(String, BTreeMap<String, BTreeMap<u32, BTreeMap<Dance, u32>>>)> = Vec::new();
        let mut all_mark_results: Vec<(String, BTreeMap<String, BTreeMap<u32, BTreeMap<Dance, bool>>>)> = Vec::new();
        let document = Html::parse_document(html);
        let tr_sel = Selector::parse("tr").unwrap();
        let td_sel = Selector::parse("td").unwrap();
        let tooltip_sel = Selector::parse(".tooltip2w").unwrap();

        // Pass 1: Determine global round order
        let mut global_round_ids = BTreeMap::new(); // id string -> canonical name
        let r_cell_sel = Selector::parse("td.td5c").unwrap();
        for row in document.select(&tr_sel) {
             if let Some(idx_cell) = row.select(&r_cell_sel).next() {
                  let idx_html = idx_cell.inner_html();
                  for part in idx_html.split("<br>") {
                       let p = part.trim();
                       if p.is_empty() { continue; }
                       let name = if p == "F" {
                            "Endrunde".to_string()
                       } else if let Ok(n) = p.parse::<u32>() {
                            if n == 1 {
                                 "Vorrunde".to_string()
                            } else if n > 1 {
                                 format!("{}. Zwischenrunde", n - 1)
                            } else {
                                 p.to_string()
                            }
                       } else {
                            p.to_string()
                       };
                       global_round_ids.insert(p.to_string(), name);
                  }
             }
        }

        // Logical ordering for round IDs: 1, 2, 3, ..., F
        let mut sorted_ids: Vec<String> = global_round_ids.keys().cloned().collect();
        sorted_ids.sort_by(|a, b| {
             if a == "F" { return std::cmp::Ordering::Greater; }
             if b == "F" { return std::cmp::Ordering::Less; }
             let an = a.parse::<u32>().unwrap_or(0);
             let bn = b.parse::<u32>().unwrap_or(0);
             an.cmp(&bn)
        });

        let mut judge_codes = Vec::new();
        let mut seen_codes = std::collections::HashSet::new();

        for row in document.select(&tr_sel).take(5) {
             let cells: Vec<_> = row.select(&td_sel).collect();
             let mut found_codes = Vec::new();
             seen_codes.clear();
             for cell in &cells {
                  if cell.select(&tooltip_sel).next().is_some() {
                       // Short code is just the first text child
                       let t = cell.text().next().unwrap_or("").trim();
                       let mut code = String::new();
                       for c in t.chars() {
                            if c.is_ascii_uppercase() { code.push(c); } else { break; }
                       }

                       if !code.is_empty() && code.len() <= 2 {
                            if seen_codes.contains(&code) {
                                 break;
                            }
                            seen_codes.insert(code.clone());
                            found_codes.push(code);
                       }
                  }
             }
             if found_codes.len() > 3 {
                  judge_codes = found_codes;
                  break;
             }
        }

        let bib_sel = Selector::parse("td.td2cv, td.td2c").unwrap();
        let td5w_sel = Selector::parse("td.td5w").unwrap();

        for row in document.select(&tr_sel) {
             let cells: Vec<_> = row.select(&td_sel).collect();
             if cells.len() < 5 { continue; }

             let bib_text = row.select(&bib_sel).next()
                 .map(|c| {
                      let t = c.text().collect::<String>().trim().to_string();
                      let mut num_str = String::new();
                      for ch in t.chars() { if ch.is_ascii_digit() { num_str.push(ch); } else { break; } }
                      num_str
                 })
                 .unwrap_or_default();

             if let Ok(bib) = bib_text.parse::<u32>() {
                  let mut row_round_indices = Vec::new();
                  if let Some(idx_cell) = row.select(&Selector::parse("td.td5c").unwrap()).next() {
                       let idx_html = idx_cell.inner_html();
                       for part in idx_html.split("<br>") {
                            let p = part.trim();
                            if let Some(pos) = sorted_ids.iter().position(|id| id == p) {
                                 row_round_indices.push(pos);
                            }
                       }
                  }

                  let mut dance_cell_count = 0;
                  for cell in row.select(&td5w_sel) {
                       let content = cell.inner_html();
                       let lines: Vec<_> = content.split("<br>").collect();
                       for (line_idx, line) in lines.iter().enumerate() {
                            let val = line.trim();
                            if val.is_empty() { continue; }

                            let judge_idx = dance_cell_count % (if judge_codes.is_empty() { 1 } else { judge_codes.len() });
                            let dance_idx = dance_cell_count / (if judge_codes.is_empty() { 1 } else { judge_codes.len() });
                            let adj_code = judge_codes.get(judge_idx).cloned().unwrap_or_else(|| "A".to_string());
                            let dance = dances.get(dance_idx);

                            if let Some(&global_idx) = row_round_indices.get(line_idx) {
                                 if let Ok(rank) = val.parse::<u32>() {
                                      while all_rank_results.len() <= global_idx {
                                           all_rank_results.push((String::new(), BTreeMap::new()));
                                      }
                                      all_rank_results[global_idx].0 = global_round_ids[&sorted_ids[global_idx]].clone();
                                      let results = &mut all_rank_results[global_idx].1;
                                      if let Some(d) = dance {
                                           results.entry(adj_code).or_insert_with(BTreeMap::new).entry(bib).or_insert_with(BTreeMap::new).insert(*d, rank);
                                      }
                                 } else if val.to_lowercase().contains('x') || val == "-" {
                                      let has_cross = val.to_lowercase().contains('x');
                                      while all_mark_results.len() <= global_idx {
                                           all_mark_results.push((String::new(), BTreeMap::new()));
                                      }
                                      all_mark_results[global_idx].0 = global_round_ids[&sorted_ids[global_idx]].clone();
                                      let results = &mut all_mark_results[global_idx].1;
                                      if let Some(d) = dance {
                                           results.entry(adj_code).or_insert_with(BTreeMap::new).entry(bib).or_insert_with(BTreeMap::new).insert(*d, has_cross);
                                      }
                                 }
                            }
                       }
                       dance_cell_count += 1;
                  }
             }
        }

        (all_mark_results, all_rank_results)
    }

    pub fn parse_wdsf_scores(&self, html: &str) -> BTreeMap<String, BTreeMap<u32, WDSFScore>> {
        let mut results = BTreeMap::new();
        let document = Html::parse_document(html);
        let tr_sel = Selector::parse("tr").unwrap();
        let td_sel = Selector::parse("td").unwrap();
        let score_re = Regex::new(r"(\d+[\.,]\d+)").unwrap();

        let mut current_bib = 0;
        let mut current_judge = String::new();

        for row in document.select(&tr_sel) {
            let cells: Vec<_> = row.select(&td_sel).collect();
            if cells.is_empty() { continue; }

            let text = cells[0].text().collect::<String>().trim().to_string();
            if let Some(caps) = Regex::new(r"\((\d+)\)").unwrap().captures(&text) {
                current_bib = caps[1].parse().unwrap_or(0);
            }

            if text.len() == 1 && text.chars().next().unwrap().is_ascii_uppercase() {
                current_judge = text;
            }

            if !current_judge.is_empty() && current_bib != 0 {
                let cell_text = row.text().collect::<Vec<_>>().join(" ");
                let scores: Vec<f64> = score_re.find_iter(&cell_text)
                    .filter_map(|m| m.as_str().replace(',', ".").parse().ok())
                    .collect();

                if !scores.is_empty() {
                    let score_entry = results.entry(current_judge.clone()).or_insert_with(BTreeMap::new)
                        .entry(current_bib).or_insert(WDSFScore {
                            technical_quality: 0.0,
                            movement_to_music: 0.0,
                            partnering_skills: 0.0,
                            choreography: 0.0,
                        total: 0.0,
                        });

                    if cell_text.contains("TQ") { score_entry.technical_quality = scores[0]; }
                    if cell_text.contains("MM") { score_entry.movement_to_music = scores[0]; }
                    if cell_text.contains("PS") { score_entry.partnering_skills = scores[scores.len()-1]; }
                    if cell_text.contains("CP") { score_entry.choreography = scores[scores.len()-1]; }

                    if scores.len() >= 2 && cell_text.contains("TQ") && cell_text.contains("PS") {
                        score_entry.technical_quality = scores[0];
                        score_entry.partnering_skills = scores[1];
                    }
                    if scores.len() == 1 && cell_text.contains("MM") && cell_text.contains("CP") {
                        score_entry.movement_to_music = scores[0];
                        score_entry.choreography = scores[0];
                    }

                    if cell_text.contains("Summe") || cell_text.contains("Total") {
                        score_entry.total = scores[0];
                    }
                }
            }
        }
        results
    }

    pub fn canonicalize_round_name(&self, name: &str) -> Option<String> {
        let lower = name.to_lowercase();
        if lower.contains("vorrunde") {
             Some("Vorrunde".to_string())
        } else if lower.contains("zwischenrunde") {
             if lower.contains("1.") || lower.contains("erste") {
                  Some("1. Zwischenrunde".to_string())
             } else if lower.contains("2.") || lower.contains("zweite") {
                  Some("2. Zwischenrunde".to_string())
             } else if lower.contains("3.") || lower.contains("dritte") {
                  Some("3. Zwischenrunde".to_string())
             } else {
                  Some("Zwischenrunde".to_string())
             }
        } else if lower.contains("endrunde") || lower.contains("finale") || lower.contains("final") {
             Some("Endrunde".to_string())
        } else {
             None
        }
    }

    pub fn parse_competition_from_title(&self, title: &str) -> Result<Competition, ParsingError> {
        let title_up = title.to_uppercase();
        let mut sorted_age_keys: Vec<_> = self.i18n.aliases.age_groups.keys().collect();
        sorted_age_keys.sort_by_key(|k| k.len());
        sorted_age_keys.reverse();

        let mut sorted_disc_keys: Vec<_> = self.i18n.aliases.dances.keys().collect();
        sorted_disc_keys.sort_by_key(|k| k.len());
        sorted_disc_keys.reverse();

        let mut age_group = None;
        let mut style = None;
        let mut level = None;

        for key in &sorted_age_keys {
            if title_up.contains(&key.to_uppercase()) {
                age_group = self.i18n.map_age_group(key);
                break;
            }
        }

        for key in &sorted_disc_keys {
            if title_up.contains(&key.to_uppercase()) {
                style = self.i18n.map_discipline(key);
                break;
            }
        }

        for l_id in ["S", "A", "B", "C", "D", "E"] {
            let pattern = format!(" {} ", l_id);
            if title_up.contains(&pattern) || title_up.ends_with(&format!(" {}", l_id)) {
                level = Level::from_id(l_id);
                break;
            }
        }

        if level.is_none() && (title_up.contains("WDSF") || title_up.contains("OPEN")) {
            level = Some(Level::S);
        }

        let date = self.parse_date(title).unwrap_or_else(|| NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        if age_group.is_none() || style.is_none() || level.is_none() {
             return Err(ParsingError::MissingRequiredData(format!("Incomplete metadata in title: {}", title)));
        }

        let age_group = age_group.unwrap();
        let style = style.unwrap();
        let level = level.unwrap();
        let dances = self.parse_dances(title);
        let min_dances = crate::models::validation::get_min_dances_for_level(&self.config.levels, &level, &date);

        Ok(Competition {
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
        })
    }
}

impl ResultSource for DtvNative {
    fn name(&self) -> &str {
        "DTV"
    }

    fn fetch(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let resp = reqwest::blocking::get(url)?;
        Ok(resp.text()?)
    }

    fn parse(&self, html: &str) -> Result<Event, ParsingError> {
        let document = Html::parse_document(html);

        let title_sel = Selector::parse("title").unwrap();
        let title = document.select(&title_sel).next().map(|n| n.inner_html()).unwrap_or_default();

        let name_sel = Selector::parse(&self.selectors.event_name).unwrap();
        let event_name = document.select(&name_sel).next().map(|e| e.text().collect::<String>().trim().to_string());

        let date_sel = Selector::parse(&self.selectors.event_date).unwrap();
        let date_text = document.select(&date_sel).next().map(|e| e.text().collect::<String>().trim().to_string());

        let event_date = date_text.and_then(|dt| self.parse_date(&dt)).or_else(|| self.parse_date(&title));

        let organizer_sel = Selector::parse(&self.selectors.organizer).unwrap();
        let organizer = document.select(&organizer_sel).next().map(|e| e.text().collect::<String>().trim().to_string());

        let hosting_club_sel = Selector::parse(&self.selectors.hosting_club).unwrap();
        let hosting_club = document.select(&hosting_club_sel).next().map(|e| e.text().collect::<String>().trim().to_string());

        let mut competitions = Vec::new();
        let item_sel = Selector::parse(&self.selectors.competition_item).unwrap();

        for item in document.select(&item_sel) {
            let item_text = item.text().collect::<String>().trim().to_string();
            if let Ok(comp) = self.parse_competition_from_title(&item_text) {
                competitions.push(comp);
            }
        }

        if competitions.is_empty() {
            if let Some(ref name) = event_name {
                if let Ok(comp) = self.parse_competition_from_title(name) {
                    competitions.push(comp);
                }
            }
            if competitions.is_empty() && !title.is_empty() {
                if let Ok(comp) = self.parse_competition_from_title(&title) {
                    competitions.push(comp);
                }
            }
        }

        if competitions.is_empty() {
            return Err(ParsingError::MissingRequiredData("No valid competitions found in event index".to_string()));
        }

        Ok(Event {
            name: event_name.unwrap_or(title),
            organizer,
            hosting_club,
            competitions_list: competitions,
            date: event_date,
        })
    }
}

pub fn get_config_and_i18n(config_path: &str) -> Result<(Config, I18n)> {
    let config_content = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_content)?;

    let aliases_path = "assets/aliases.toml";
    let i18n = I18n::new(aliases_path)?;
    Ok((config, i18n))
}

pub fn extract_event_data(data_dir: &str) -> Result<Event> {
    let config_path = "config/config.toml";
    let (config, i18n) = get_config_and_i18n(config_path)?;
    let parser = DtvNative::new(config, SelectorConfig::default(), i18n);

    let dir_path = Path::new(data_dir);
    let index_path = dir_path.join("index.htm");

    let index_html = if index_path.exists() {
        fs::read_to_string(&index_path).ok()
    } else {
        None
    };

    let erg_path = dir_path.join("erg.htm");
    let erg_html = if erg_path.exists() {
        fs::read_to_string(&erg_path).ok()
    } else {
        None
    };

    let mut event = if let Some(ref html) = index_html {
        match parser.parse(html) {
            Ok(e) => e,
            Err(_) if erg_html.is_some() => parser.parse(erg_html.as_ref().unwrap()).map_err(|e| anyhow::anyhow!("Parsing error: {}", e))?,
            Err(e) => return Err(anyhow::anyhow!("Parsing error: {}", e)),
        }
    } else if let Some(ref html) = erg_html {
        parser.parse(html).map_err(|e| anyhow::anyhow!("Parsing error: {}", e))?
    } else {
        return Err(anyhow::anyhow!("No valid htm files found in {}", data_dir));
    };

    for comp in &mut event.competitions_list {
        let files = ["erg.htm", "deck.htm", "tabges.htm", "ergwert.htm"];
        for file in files {
            let p = dir_path.join(file);
            if p.exists() {
                if let Ok(content) = fs::read_to_string(&p) {
                    match file {
                        "erg.htm" => {
                            if let Ok(parts) = parser.parse_participants(&content) {
                                comp.participants = parts;
                            }
                            let detected_dances = parser.parse_dances_from_table(&content);
                            if !detected_dances.is_empty() {
                                comp.dances = detected_dances;
                            }
                            let rounds = parser.parse_rounds(&content, &comp.dances);
                            for r in rounds {
                                if let Some(existing) = comp.rounds.iter_mut().find(|existing| existing.name == r.name) {
                                     if let Some(new_marks) = r.marking_crosses {
                                          if let Some(ref mut existing_marks) = existing.marking_crosses {
                                               // Merge: prefer new_marks if they are more detailed (vary by dance)
                                               // Actually, just always merge judge by judge, bib by bib.
                                               for (judge, bibs) in new_marks {
                                                    let existing_judge = existing_marks.entry(judge).or_default();
                                                    for (bib, dances) in bibs {
                                                         let existing_bib = existing_judge.entry(bib).or_default();
                                                         for (dance, has_cross) in dances {
                                                              // Only overwrite if existing is missing or if new has more info?
                                                              // Simplify: if new_marks come from ergwert, they are detailed.
                                                              // Tabges aggregate marks will set all dances to the same value.
                                                              existing_bib.insert(dance, has_cross);
                                                         }
                                                    }
                                               }
                                          } else {
                                               existing.marking_crosses = Some(new_marks);
                                          }
                                     }
                                     if let Some(new_ranks) = r.dtv_ranks {
                                          if let Some(ref mut existing_ranks) = existing.dtv_ranks {
                                               for (judge, bibs) in new_ranks {
                                                    let existing_judge = existing_ranks.entry(judge).or_default();
                                                    for (bib, dances) in bibs {
                                                         let existing_bib = existing_judge.entry(bib).or_default();
                                                         for (dance, rank) in dances {
                                                              existing_bib.insert(dance, rank);
                                                         }
                                                    }
                                               }
                                          } else {
                                               existing.dtv_ranks = Some(new_ranks);
                                          }
                                     }
                                     if r.wdsf_scores.is_some() { existing.wdsf_scores = r.wdsf_scores; }
                                } else {
                                    comp.rounds.push(r);
                                }
                            }
                        }
                        "deck.htm" => {
                            if let Ok(off) = parser.parse_officials(&content) {
                                comp.officials = off;
                            }
                            // Extract organizer and hosting club if missing
                            let doc = scraper::Html::parse_document(&content);
                            let tr_sel = scraper::Selector::parse("tr").unwrap();
                            let td_sel = scraper::Selector::parse("td").unwrap();
                            for row in doc.select(&tr_sel) {
                                let cells: Vec<_> = row.select(&td_sel).collect();
                                if cells.len() >= 2 {
                                    let key = cells[0].text().collect::<String>();
                                    let val = cells[1].text().collect::<String>().trim().to_string();
                                    if key.contains("Veranstalter") && event.organizer.is_none() {
                                        event.organizer = Some(val);
                                    } else if key.contains("Ausrichter") && event.hosting_club.is_none() {
                                        event.hosting_club = Some(val);
                                    }
                                }
                            }
                        }
                        "tabges.htm" | "ergwert.htm" => {
                            let rounds = parser.parse_rounds(&content, &comp.dances);
                            for r in rounds {
                                if let Some(existing) = comp.rounds.iter_mut().find(|existing| existing.name == r.name) {
                                     if r.marking_crosses.is_some() { existing.marking_crosses = r.marking_crosses; }
                                     if r.dtv_ranks.is_some() { existing.dtv_ranks = r.dtv_ranks; }
                                     if r.wdsf_scores.is_some() { existing.wdsf_scores = r.wdsf_scores; }
                                } else {
                                    comp.rounds.push(r);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(event)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Aliases;
    use crate::models::{AgeGroup, Level, Style};
    use std::fs;

    #[test]
    fn test_parse_date() {
        let i18n = I18n { aliases: Aliases { age_groups: HashMap::new(), dances: HashMap::new(), roles: HashMap::new() } };
        let config = Config { sources: crate::crawler::client::Sources { urls: vec![] }, levels: None };
        let parser = DtvNative::new(config, SelectorConfig::default(), i18n);

        assert_eq!(parser.parse_date("11.05.2024"), Some(NaiveDate::from_ymd_opt(2024, 5, 11).unwrap()));
        assert_eq!(parser.parse_date("05/Jul/2025"), Some(NaiveDate::from_ymd_opt(2025, 7, 5).unwrap()));
        assert_eq!(parser.parse_date("17. Mai 2025"), Some(NaiveDate::from_ymd_opt(2025, 5, 17).unwrap()));
    }

    #[test]
    fn test_parse_competition_from_title() {
        let aliases_content = r#"
            [age_groups]
            "Hgr.II" = "adult_2"
            [dances]
            "Standard" = "std"
        "#;
        let aliases: Aliases = toml::from_str(aliases_content).unwrap();
        let i18n = I18n { aliases };
        let config = Config { sources: crate::crawler::client::Sources { urls: vec![] }, levels: None };
        let parser = DtvNative::new(config, SelectorConfig::default(), i18n);

        let comp = parser.parse_competition_from_title("11.05.2024 Hgr.II D Standard").unwrap();
        assert_eq!(comp.level, Level::D);
        assert_eq!(comp.age_group, AgeGroup::Adult2);
        assert_eq!(comp.style, Style::Standard);
    }

    #[test]
    fn test_parse_tabges_vertical() {
         let html = r#"
            <table>
                <tr><td>Adjudicators</td><td>101</td></tr>
                <tr><td>AT) Judge Name</td><td>x</td></tr>
            </table>
         "#;
        let dances = vec![Dance::SlowWaltz];
        let i18n = I18n { aliases: Aliases { age_groups: HashMap::new(), dances: HashMap::new(), roles: HashMap::new() } };
        let config = Config { sources: crate::crawler::client::Sources { urls: vec![] }, levels: None };
        let parser = DtvNative::new(config, SelectorConfig::default(), i18n);

        let crosses = parser.parse_tabges(html, &dances);
        assert!(crosses[0].1["AT"][&101][&Dance::SlowWaltz]);
    }

    #[test]
    fn test_parse_wdsf_scores() {
         let html = r#"
            <table>
                <tr><td>(284) Rohde</td></tr>
                <tr><td>A</td><td>TQ|PS 9.75|9.75</td></tr>
                <tr><td>A</td><td>MM+CP 9.50</td></tr>
            </table>
         "#;
        let i18n = I18n { aliases: Aliases { age_groups: HashMap::new(), dances: HashMap::new(), roles: HashMap::new() } };
        let config = Config { sources: crate::crawler::client::Sources { urls: vec![] }, levels: None };
        let parser = DtvNative::new(config, SelectorConfig::default(), i18n);

        let scores = parser.parse_wdsf_scores(html);
        let s = &scores["A"][&284];
        assert_eq!(s.technical_quality, 9.75);
        assert_eq!(s.partnering_skills, 9.75);
        assert_eq!(s.movement_to_music, 9.50);
        assert_eq!(s.choreography, 9.50);
    }

    #[test]
    fn test_min_dances_2026_compliance() {
        let config_str = r#"
            [sources]
            urls = []
            [levels.D]
            min_dances_legacy = 3
            min_dances_2026 = 4
        "#;
        let config: Config = toml::from_str(config_str).unwrap();
        let aliases_content = r#"
            [age_groups]
            "Hgr.II" = "adult_2"
            [dances]
            "Standard" = "std"
        "#;
        let aliases: Aliases = toml::from_str(aliases_content).unwrap();
        let i18n = I18n { aliases };
        let parser = DtvNative::new(config, SelectorConfig::default(), i18n);

        let comp2024 = parser.parse_competition_from_title("11.05.2024 Hgr.II D Standard").unwrap();
        assert_eq!(comp2024.min_dances, 3);

        let comp2026 = parser.parse_competition_from_title("11.05.2026 Hgr.II D Standard").unwrap();
        assert_eq!(comp2026.min_dances, 4);
    }

    #[test]
    fn test_parse_participants() {
        let html = r#"
            <TABLE class="tab1">
                <TR><TD class="td3r">1.</TD>
                    <TD class="td5">Jonathan Kummetz / Elisabeth Findeiß (610)<BR><i>1. TC Rot-Gold Bayreuth</i></TD>
                </TR>
            </TABLE>
        "#;
        let i18n = I18n { aliases: Aliases { age_groups: HashMap::new(), dances: HashMap::new(), roles: HashMap::new() } };
        let config = Config { sources: crate::crawler::client::Sources { urls: vec![] }, levels: None };
        let parser = DtvNative::new(config, SelectorConfig::default(), i18n);
        let participants = parser.parse_participants(html).unwrap();
        assert_eq!(participants[0].bib_number, 610);
        assert_eq!(participants[0].name_one, "Jonathan Kummetz");
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
                    <TD class="td2r">AT:</TD>
                    <TD class="td5"><span class="col1">Marcus Bärschneider</span><span>TSC Hagen</span></TD>
                </TR>
            </TABLE>
        "#;
        let mut roles = HashMap::new();
        roles.insert("Turnierleiter".to_string(), "responsible_person".to_string());
        let i18n = I18n { aliases: Aliases { age_groups: HashMap::new(), dances: HashMap::new(), roles } };
        let config = Config { sources: crate::crawler::client::Sources { urls: vec![] }, levels: None };
        let parser = DtvNative::new(config, SelectorConfig::default(), i18n);
        let officials = parser.parse_officials(html).unwrap();
        assert!(officials.responsible_person.is_some());
        assert_eq!(officials.judges[0].code, "AT");
    }

    #[test]
    fn test_real_wdsf_world_open_tabges() {
        let html = fs::read_to_string("tests/44-0507_wdsfworldopenlatadult/tabges.htm").unwrap();
        let dances = vec![Dance::Samba];
        let i18n = I18n { aliases: Aliases { age_groups: HashMap::new(), dances: HashMap::new(), roles: HashMap::new() } };
        let config = Config { sources: crate::crawler::client::Sources { urls: vec![] }, levels: None };
        let parser = DtvNative::new(config, SelectorConfig::default(), i18n);

        let results = parser.parse_tabges(&html, &dances);
        // Bib 284 is seeded and only starts in Round 2 (index 1)
        assert!(results[1].1["A"][&284][&Dance::Samba]);
    }

    #[test]
    fn test_real_wdsf_rising_stars_ergwert() {
        let html = fs::read_to_string("tests/47-0507_wdsfopenstdrisingstars/ergwert.htm").unwrap();
        let dances = vec![Dance::SlowWaltz, Dance::Tango, Dance::VienneseWaltz, Dance::SlowFoxtrot, Dance::Quickstep];
        let i18n = I18n { aliases: Aliases { age_groups: HashMap::new(), dances: HashMap::new(), roles: HashMap::new() } };
        let config = Config { sources: crate::crawler::client::Sources { urls: vec![] }, levels: None };
        let parser = DtvNative::new(config, SelectorConfig::default(), i18n);

        let results = parser.parse_ergwert(&html, &dances);
        assert!(results[0].1.contains_key("D"));
        assert!(results[0].1["D"].contains_key(&721));
    }
}
