use crate::models::{
    CommitteeMember, Competition, Dance, IdentityType, Judge, Level, Officials,
    Participant, WDSFScore, Round, RoundData,
};
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
    pub selectors: SelectorConfig,
}

impl DtvNative {
    /// Creates a new DtvNative parser.
    pub fn new(selectors: SelectorConfig) -> Self {
        Self {
            selectors,
        }
    }

    pub fn do_parse_date(&self, s: &str) -> Option<NaiveDate> {
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
            let m = crate::i18n::map_month(mon_str)?;
            return NaiveDate::from_ymd_opt(y, m, d);
        }

        // Handle German month names
        let re_de = Regex::new(r"(\d{1,2})\.\s*([a-zA-Zä]+)\s+(\d{4})").unwrap();
        if let Some(caps) = re_de.captures(s) {
            let d = caps[1].parse::<u32>().ok()?;
            let mon_str = &caps[2];
            let y = caps[3].parse::<i32>().ok()?;
            let m = crate::i18n::map_month(mon_str)?;
            return NaiveDate::from_ymd_opt(y, m, d);
        }

        None
    }

    pub fn parse_dances(&self, s: &str) -> Vec<Dance> {
        crate::i18n::parse_dances(s)
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
                        if let Some(canonical_role) = crate::i18n::map_role(&role) {
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
            round_names.push(crate::i18n::parse_round_name(&text).unwrap_or(text));
        }

        if round_names.is_empty() {
             for head in document.select(&comphead_sel) {
                  let text = head.text().collect::<Vec<_>>().join(" ").trim().to_string();
                  if text.to_lowercase().contains("runde") || text.to_lowercase().contains("table") || text.to_lowercase().contains("ergebnis") || text.to_lowercase().contains("ranking") {
                       round_names.push(crate::i18n::parse_round_name(&text).unwrap_or(text));
                  }
             }

             let td_sel = Selector::parse("td.td1, td.td3").unwrap();
             for td in document.select(&td_sel) {
                 let text = td.text().collect::<Vec<_>>().join(" ").trim().to_string();
                 if let Some(name) = crate::i18n::parse_round_name(&text) {
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

             let mut wdsf = None;
             if let Some(ref wdsf_res) = wdsf_results {
                  if i == 0 { // Assume WDSF results are for the first round detected in the file
                       wdsf = Some(wdsf_res.clone());
                  }
             }

             if let Some(wdsf_scores) = wdsf {
                  rounds.push(Round {
                       name,
                       order: i as u32,
                       dances: dances.to_vec(),
                       data: RoundData::WDSF { wdsf_scores },
                  });
             } else if let Some(dtv_ranks) = ranks {
                  rounds.push(Round {
                       name,
                       order: i as u32,
                       dances: dances.to_vec(),
                       data: RoundData::DTV { dtv_ranks },
                  });
             } else if let Some(marking_crosses) = marks {
                  rounds.push(Round {
                       name,
                       order: i as u32,
                       dances: dances.to_vec(),
                       data: RoundData::Marking { marking_crosses },
                  });
             }
        }

        rounds
    }

    pub fn parse_tabges(
        &self,
        html: &str,
        dances: &[Dance],
    ) -> Vec<(String, BTreeMap<String, BTreeMap<String, BTreeMap<Dance, bool>>>)> {
        let mut all_results: Vec<(String, BTreeMap<String, BTreeMap<String, BTreeMap<Dance, bool>>>)> = Vec::new();
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
                                                 .entry(bib.to_string()).or_insert_with(BTreeMap::new);
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
                                        if let Some(bib_map) = judge_map.get(&bib.to_string()) {
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
                                let bib_map = current_results.entry(judge_code.clone()).or_insert_with(BTreeMap::new).entry(bib.to_string()).or_insert_with(BTreeMap::new);
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

    pub fn parse_ergwert(&self, html: &str, dances: &[Dance]) -> (Vec<(String, BTreeMap<String, BTreeMap<String, BTreeMap<Dance, bool>>>)>, Vec<(String, BTreeMap<String, BTreeMap<String, BTreeMap<Dance, u32>>>)>) {
        let mut all_rank_results: Vec<(String, BTreeMap<String, BTreeMap<String, BTreeMap<Dance, u32>>>)> = Vec::new();
        let mut all_mark_results: Vec<(String, BTreeMap<String, BTreeMap<String, BTreeMap<Dance, bool>>>)> = Vec::new();
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
                       let name = crate::i18n::get_round_name_from_id(p);
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
                                           results.entry(adj_code).or_insert_with(BTreeMap::new).entry(bib.to_string()).or_insert_with(BTreeMap::new).insert(*d, rank);
                                      }
                                 } else if val.to_lowercase().contains('x') || val == "-" {
                                      let has_cross = val.to_lowercase().contains('x');
                                      while all_mark_results.len() <= global_idx {
                                           all_mark_results.push((String::new(), BTreeMap::new()));
                                      }
                                      all_mark_results[global_idx].0 = global_round_ids[&sorted_ids[global_idx]].clone();
                                      let results = &mut all_mark_results[global_idx].1;
                                      if let Some(d) = dance {
                                           results.entry(adj_code).or_insert_with(BTreeMap::new).entry(bib.to_string()).or_insert_with(BTreeMap::new).insert(*d, has_cross);
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

    pub fn parse_wdsf_scores(&self, html: &str) -> BTreeMap<String, BTreeMap<String, WDSFScore>> {
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
                        .entry(current_bib.to_string()).or_insert(WDSFScore {
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

    pub fn parse_competition_from_title(&self, title: &str) -> Result<Competition, ParsingError> {
        let title_up = title.to_uppercase();
        let mut sorted_age_keys = crate::i18n::age_group_keys();
        sorted_age_keys.sort_by_key(|k| k.len());
        sorted_age_keys.reverse();

        let mut sorted_disc_keys = crate::i18n::style_keys();
        sorted_disc_keys.sort_by_key(|k| k.len());
        sorted_disc_keys.reverse();

        let mut age_group = None;
        let mut style = None;
        let mut level = None;

        for key in &sorted_age_keys {
            if title_up.contains(&key.to_uppercase()) {
                age_group = crate::i18n::map_age_group(key);
                break;
            }
        }

        for key in &sorted_disc_keys {
            if title_up.contains(&key.to_uppercase()) {
                style = crate::i18n::map_discipline(key);
                break;
            }
        }

        for l_id in ["S", "A", "B", "C", "D", "E"] {
            let pattern = format!(" {} ", l_id);
            if title_up.contains(&pattern) || title_up.ends_with(&format!(" {}", l_id)) {
                level = crate::i18n::parse_level(l_id);
                break;
            }
        }

        if level.is_none() && (title_up.contains("WDSF") || title_up.contains("OPEN")) {
            level = Some(Level::S);
        }

        let date = self.do_parse_date(title).unwrap_or_else(|| NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        if age_group.is_none() || style.is_none() || level.is_none() {
             return Err(ParsingError::MissingRequiredData(format!("Incomplete metadata in title: {}", title)));
        }

        let age_group = age_group.unwrap();
        let style = style.unwrap();
        let level = level.unwrap();
        let dances = self.parse_dances(title);
        let min_dances = crate::i18n::get_min_dances(level, date);

        Ok(Competition {
            name: title.to_string(),
            date: Some(date),
            organizer: None,
            hosting_club: None,
            source_url: None,
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

    fn parse_date(&self, s: &str) -> Option<NaiveDate> {
        self.do_parse_date(s)
    }

    fn parse(&self, html: &str) -> Result<Competition, ParsingError> {
        let document = Html::parse_document(html);

        let title_sel = Selector::parse("title").unwrap();
        let title = document.select(&title_sel).next().map(|n| n.inner_html()).unwrap_or_default();

        let name_sel = Selector::parse(&self.selectors.event_name).unwrap();
        let event_name = document.select(&name_sel).next().map(|e| e.text().collect::<String>().trim().to_string());

        let date_sel = Selector::parse(&self.selectors.event_date).unwrap();
        let date_text = document.select(&date_sel).next().map(|e| e.text().collect::<String>().trim().to_string());

        let event_date = date_text.and_then(|dt| self.do_parse_date(&dt)).or_else(|| self.do_parse_date(&title));

        let organizer_sel = Selector::parse(&self.selectors.organizer).unwrap();
        let organizer = document.select(&organizer_sel).next().map(|e| e.text().collect::<String>().trim().to_string());

        let hosting_club_sel = Selector::parse(&self.selectors.hosting_club).unwrap();
        let hosting_club = document.select(&hosting_club_sel).next().map(|e| e.text().collect::<String>().trim().to_string());

        let mut competitions = Vec::new();
        let item_sel = Selector::parse(&self.selectors.competition_item).unwrap();

        for item in document.select(&item_sel) {
            let item_text = item.text().collect::<String>().trim().to_string();
            if let Ok(mut comp) = self.parse_competition_from_title(&item_text) {
                comp.organizer = organizer.clone();
                comp.hosting_club = hosting_club.clone();
                if comp.date.is_none() { comp.date = event_date; }
                competitions.push(comp);
            }
        }

        if competitions.is_empty() {
            if let Some(ref name) = event_name {
                if let Ok(mut comp) = self.parse_competition_from_title(name) {
                    comp.organizer = organizer.clone();
                    comp.hosting_club = hosting_club.clone();
                    if comp.date.is_none() { comp.date = event_date; }
                    competitions.push(comp);
                }
            }
            if competitions.is_empty() && !title.is_empty() {
                if let Ok(mut comp) = self.parse_competition_from_title(&title) {
                    comp.organizer = organizer.clone();
                    comp.hosting_club = hosting_club.clone();
                    if comp.date.is_none() { comp.date = event_date; }
                    competitions.push(comp);
                }
            }
        }

        if competitions.is_empty() {
            return Err(ParsingError::InvalidTableStructure("No valid competitions found in event index".to_string()));
        }

        // Return the first one as Competition is now single-contest
        let mut comp = competitions.remove(0);
        if comp.organizer.is_none() { comp.organizer = organizer; }
        if comp.hosting_club.is_none() { comp.hosting_club = hosting_club; }
        if comp.date.is_none() { comp.date = event_date; }

        Ok(comp)
    }
}

pub fn extract_event_data(data_dir: &str) -> Result<Competition> {
    let parser = DtvNative::new(SelectorConfig::default());

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

    let mut comp = if let Some(ref html) = index_html {
        match parser.parse(html) {
            Ok(c) => c,
            Err(_) if erg_html.is_some() => parser.parse(erg_html.as_ref().unwrap()).map_err(|e| anyhow::anyhow!("Parsing error: {}", e))?,
            Err(e) => return Err(anyhow::anyhow!("Parsing error: {}", e)),
        }
    } else if let Some(ref html) = erg_html {
        parser.parse(html).map_err(|e| anyhow::anyhow!("Parsing error: {}", e))?
    } else {
        return Err(anyhow::anyhow!("No valid htm files found in {}", data_dir));
    };

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
                                 // Simple replacement for now, favoring newer parsed data
                                 *existing = r;
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
                                if crate::i18n::is_organizer_marker(&key) && comp.organizer.is_none() {
                                    comp.organizer = Some(val);
                                } else if crate::i18n::is_hosting_club_marker(&key) && comp.hosting_club.is_none() {
                                    comp.hosting_club = Some(val);
                                }
                            }
                        }
                    }
                    "tabges.htm" | "ergwert.htm" => {
                        let rounds = parser.parse_rounds(&content, &comp.dances);
                        for r in rounds {
                            if let Some(existing) = comp.rounds.iter_mut().find(|existing| existing.name == r.name) {
                                 *existing = r;
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

    Ok(comp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AgeGroup, Level, Style};
    use std::fs;

    #[test]
    fn test_parse_date() {
        let parser = DtvNative::new(SelectorConfig::default());

        assert_eq!(parser.parse_date("11.05.2024"), Some(NaiveDate::from_ymd_opt(2024, 5, 11).unwrap()));
        assert_eq!(parser.parse_date("05/Jul/2025"), Some(NaiveDate::from_ymd_opt(2025, 7, 5).unwrap()));
        assert_eq!(parser.parse_date("17. Mai 2025"), Some(NaiveDate::from_ymd_opt(2025, 5, 17).unwrap()));
    }

    #[test]
    fn test_parse_competition_from_title() {
        let parser = DtvNative::new(SelectorConfig::default());

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
        let parser = DtvNative::new(SelectorConfig::default());

        let crosses = parser.parse_tabges(html, &dances);
        assert!(crosses[0].1["AT"]["101"][&Dance::SlowWaltz]);
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
        let parser = DtvNative::new(SelectorConfig::default());

        let scores = parser.parse_wdsf_scores(html);
        let s = &scores["A"]["284"];
        assert_eq!(s.technical_quality, 9.75);
        assert_eq!(s.partnering_skills, 9.75);
        assert_eq!(s.movement_to_music, 9.50);
        assert_eq!(s.choreography, 9.50);
    }

    #[test]
    fn test_min_dances_2026_compliance() {
        let parser = DtvNative::new(SelectorConfig::default());

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
        let parser = DtvNative::new(SelectorConfig::default());
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
        let parser = DtvNative::new(SelectorConfig::default());
        let officials = parser.parse_officials(html).unwrap();
        assert!(officials.responsible_person.is_some());
        assert_eq!(officials.judges[0].code, "AT");
    }

    #[test]
    fn test_real_wdsf_world_open_tabges() {
        let html = fs::read_to_string("tests/44-0507_wdsfworldopenlatadult/tabges.htm").unwrap();
        let dances = vec![Dance::Samba];
        let parser = DtvNative::new(SelectorConfig::default());

        let results = parser.parse_tabges(&html, &dances);
        // Bib 284 is seeded and only starts in Round 2 (index 1)
        assert!(results[1].1["A"]["284"][&Dance::Samba]);
    }

    #[test]
    fn test_real_wdsf_rising_stars_ergwert() {
        let html = fs::read_to_string("tests/47-0507_wdsfopenstdrisingstars/ergwert.htm").unwrap();
        let dances = vec![Dance::SlowWaltz, Dance::Tango, Dance::VienneseWaltz, Dance::SlowFoxtrot, Dance::Quickstep];
        let parser = DtvNative::new(SelectorConfig::default());

        let results = parser.parse_ergwert(&html, &dances);
        // Bib 721 is seeded and starts later. In this file, Endrunde (index 4) contains its ranks.
        assert!(results.1[4].1.contains_key("D"));
        assert!(results.1[4].1["D"].contains_key("721"));
    }
}
