use dancing_datacollection::sources::dtv_native::{DtvNative, SelectorConfig};
use dancing_datacollection::sources::ResultSource;
use dancing_datacollection::crawler::client::Config;
use dancing_datacollection::i18n::I18n;
use std::fs;
use std::path::Path;

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
