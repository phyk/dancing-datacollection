use dancing_datacollection::sources::dtv_native::{DtvNative, SelectorConfig};
use dancing_datacollection::sources::ResultSource;
use dancing_datacollection::crawler::client::Config;
use dancing_datacollection::i18n::I18n;
use std::fs;
use std::path::Path;
use serde_json;
use toml;

fn run_full_pipeline_test(dir_name: &str) {
    let config_path = "config/config.toml";
    let aliases_path = "assets/aliases.toml";
    let config_content = fs::read_to_string(config_path).unwrap();
    let config: Config = toml::from_str(&config_content).unwrap();
    let i18n = I18n::new(aliases_path).unwrap();
    let parser = DtvNative::new(config, SelectorConfig::default(), i18n);

    let dir_path = Path::new("tests").join(dir_name);
    let index_path = dir_path.join("index.htm");
    let index_html = fs::read_to_string(&index_path).unwrap();

    let mut event = match parser.parse(&index_html) {
        Ok(e) => e,
        Err(_) => {
            let erg_path = dir_path.join("erg.htm");
            let erg_html = fs::read_to_string(&erg_path).unwrap();
            parser.parse(&erg_html).expect(&format!("Failed to parse both index and erg for {}", dir_name))
        }
    };

    for comp in &mut event.competitions_list {
        let files = ["erg.htm", "deck.htm", "tabges.htm", "ergwert.htm"];
        for file in files {
            let p = dir_path.join(file);
            if p.exists() {
                let content = fs::read_to_string(&p).unwrap();
                match file {
                    "erg.htm" => {
                        if let Ok(parts) = parser.parse_participants(&content) {
                            comp.participants = parts;
                        }
                    }
                    "deck.htm" => {
                        if let Ok(off) = parser.parse_officials(&content) {
                            comp.officials = off;
                        }
                    }
                    "tabges.htm" | "ergwert.htm" => {
                        let rounds = parser.parse_rounds(&content, &comp.dances);
                        for r in rounds {
                            if !comp.rounds.iter().any(|existing| existing.name == r.name) {
                                comp.rounds.push(r);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        assert!(!comp.participants.is_empty(), "No participants parsed for level {:?} in {}", comp.level, dir_name);
        assert!(!comp.officials.judges.is_empty(), "No judges parsed for level {:?} in {}", comp.level, dir_name);
        assert!(!comp.rounds.is_empty(), "No rounds parsed for level {:?} in {}", comp.level, dir_name);
    }
}

#[test] fn test_integration_44_wdsf_lat() { run_full_pipeline_test("44-0507_wdsfworldopenlatadult"); }
#[test] fn test_integration_75_wdsf_std() { run_full_pipeline_test("75-0607_wdsfworldopenstdadult"); }
#[test] fn test_integration_56_dtv_d_std() { run_full_pipeline_test("56-0507_ot_mas1dstd"); }
#[test] fn test_integration_31_dtv_c_std() { run_full_pipeline_test("31-0507_ot_hgrcstd"); }
#[test] fn test_integration_54_dtv_b_std() { run_full_pipeline_test("54-0507_ot_hgr2bstd"); }
#[test] fn test_integration_15_dtv_a_std() { run_full_pipeline_test("15-0407_ot_hgr2astd"); }
#[test] fn test_integration_47_dtv_s_std() { run_full_pipeline_test("47-0507_wdsfopenstdrisingstars"); }
#[test] fn test_integration_03_dtv_d_lat() { run_full_pipeline_test("3-0407_ot_mas2dlat"); }
#[test] fn test_integration_37_dtv_c_lat() { run_full_pipeline_test("37-0507_ot_mas1clat"); }
#[test] fn test_integration_42_dtv_b_lat() { run_full_pipeline_test("42-0507_ot_mas1blat"); }
#[test] fn test_integration_61_dtv_a_lat() { run_full_pipeline_test("61-0607_ot_hgr2alat"); }
#[test] fn test_integration_24_dtv_s_lat() { run_full_pipeline_test("24-0407_wdsfopenlatrisingstars"); }

#[test]
fn test_golden_file_03_dtv_d_lat() {
    use dancing_datacollection::models::validation::validate_event_fidelity;
    use chrono::NaiveDate;

    let dir_name = "3-0407_ot_mas2dlat";
    let config_path = "config/config.toml";
    let aliases_path = "assets/aliases.toml";
    let config_content = fs::read_to_string(config_path).unwrap();
    let config: Config = toml::from_str(&config_content).unwrap();
    let i18n = I18n::new(aliases_path).unwrap();
    let parser = DtvNative::new(config, SelectorConfig::default(), i18n);

    let dir_path = Path::new("tests").join(dir_name);

    let mut event = dancing_datacollection::sources::dtv_native::extract_event_data(dir_path.to_str().unwrap()).unwrap();

    // 1. Verify against Ground Truth
    let ground_truth_json = r#"{
      "name": "04.07.2025 Mas.II D Latein",
      "date": "2025-07-04",
      "organizer": "Tanzsportverband Nordrhein-Westfalen e.V.",
      "hosting_club": "Tanzsportverband Nordrhein-Westfalen e.V.",
      "competitions_list": [
        {
          "level": "D",
          "age_group": "Sen2",
          "style": "Latein",
          "dances": ["ChaChaCha", "Rumba", "Jive"],
          "min_dances": 3,
          "officials": {
            "responsible_person": { "name": "Frank Wichter", "club": "TTC Rot-Gold Köln" },
            "assistant": { "name": "Anja Ott", "club": "casino blau-gelb essen e.v." },
            "judges": [
              { "code": "AA", "name": "Thierry Ball", "club": "Tanz Sport Academy Allround Havelland" },
              { "code": "AG", "name": "Bettina Bäumer", "club": "VTG Grün-Gold Recklinghausen" },
              { "code": "BR", "name": "Doris Kösel", "club": "T.C.H. Oldenburg" },
              { "code": "CR", "name": "Mario Schiena", "club": "TSA d. SG Langenfeld 92/72" },
              { "code": "DB", "name": "Alexander von Lennep", "club": "TD Tanzsportclub Düsseldorf Rot-Weiß" }
            ]
          },
          "participants": [
            {
              "identity_type": "Couple",
              "name_one": "Ingo Wanke",
              "name_two": "Johanna Witt",
              "bib_number": 1408,
              "affiliation": "TD Tanzsportclub Düsseldorf Rot-Weiß",
              "final_rank": 1
            },
            {
              "identity_type": "Couple",
              "name_one": "Christian Schöffl",
              "name_two": "Dr. Susan Schöffl",
              "bib_number": 1136,
              "affiliation": "TSA d. 1. SSV Saalfeld 92",
              "final_rank": 2
            }
          ],
          "rounds": [
            {
              "name": "Ergebnis mit Wertung",
              "marking_crosses": null,
              "dtv_ranks": {
                "AA": { "1408": { "ChaChaCha": 2, "Rumba": 2, "Jive": 2 }, "1136": { "ChaChaCha": 1, "Rumba": 1, "Jive": 1 } },
                "AG": { "1408": { "ChaChaCha": 1, "Rumba": 2, "Jive": 2 }, "1136": { "ChaChaCha": 2, "Rumba": 1, "Jive": 1 } },
                "BR": { "1408": { "ChaChaCha": 1, "Rumba": 1, "Jive": 1 }, "1136": { "ChaChaCha": 2, "Rumba": 2, "Jive": 2 } },
                "CR": { "1408": { "ChaChaCha": 1, "Rumba": 1, "Jive": 2 }, "1136": { "ChaChaCha": 2, "Rumba": 2, "Jive": 1 } },
                "DB": { "1408": { "ChaChaCha": 2, "Rumba": 1, "Jive": 1 }, "1136": { "ChaChaCha": 1, "Rumba": 2, "Jive": 2 } }
              },
              "wdsf_scores": null
            }
          ]
        }
      ]
    }"#;

    let expected_event: dancing_datacollection::models::Event = serde_json::from_str(ground_truth_json).unwrap();
    assert_eq!(event.name, expected_event.name);
    assert_eq!(event.date, expected_event.date);
    assert_eq!(event.organizer, expected_event.organizer);
    assert_eq!(event.hosting_club, expected_event.hosting_club);
    assert_eq!(event.competitions_list.len(), expected_event.competitions_list.len());

    for (actual, expected) in event.competitions_list.iter().zip(expected_event.competitions_list.iter()) {
        assert_eq!(actual.level, expected.level);
        assert_eq!(actual.age_group, expected.age_group);
        assert_eq!(actual.style, expected.style);
        assert_eq!(actual.dances, expected.dances);
        assert_eq!(actual.min_dances, expected.min_dances);
        assert_eq!(actual.officials, expected.officials);
        assert_eq!(actual.participants, expected.participants);
        assert_eq!(actual.rounds, expected.rounds);
    }

    // 2. Verify 2025 Temporal Rule
    assert!(validate_event_fidelity(&event), "Event should be valid for 2025 (min_dances=3)");

    // 3. Verify 2026 Temporal Rule (should fail)
    event.date = Some(NaiveDate::from_ymd_opt(2026, 3, 4).unwrap());
    for comp in &mut event.competitions_list {
        comp.min_dances = dancing_datacollection::models::validation::get_min_dances_for_level(
            &parser.config.levels,
            &comp.level,
            &event.date.unwrap()
        );
    }
    assert!(!validate_event_fidelity(&event), "Event should be invalid for 2026 (min_dances=4 required for Level D)");
}
