use std::path::Path;

fn run_full_pipeline_test(dir_name: &str) {
    let dir_path = Path::new("tests").join(dir_name);
    let event = dancing_datacollection::sources::dtv_native::extract_event_data(dir_path.to_str().unwrap()).unwrap();

    assert!(!event.participants.is_empty(), "No participants parsed in {}", dir_name);
    assert!(!event.officials.judges.is_empty(), "No judges parsed in {}", dir_name);
    assert!(!event.rounds.is_empty(), "No rounds parsed in {}", dir_name);

    // Verify against Ground Truth if it exists
    // NOTE: Ground truth files might need update to match new Competition-only format.
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
#[test] fn test_integration_51_dtv_d_std() { run_full_pipeline_test("51-1105_ot_hgr2dstd"); }
#[test] fn test_integration_52_dtv_c_std() { run_full_pipeline_test("52-1105_ot_hgr2cstd"); }
#[test] fn test_integration_53_dtv_b_std() { run_full_pipeline_test("53-1105_ot_hgr2bstd"); }

#[test]
fn test_golden_file_03_dtv_d_lat() {
    use dancing_datacollection::models::validation::validate_competition_fidelity;
    use chrono::NaiveDate;

    let dir_name = "3-0407_ot_mas2dlat";
    let dir_path = Path::new("tests").join(dir_name);

    let mut event = dancing_datacollection::sources::dtv_native::extract_event_data(dir_path.to_str().unwrap()).unwrap();

    // 1. Verify 2025 Temporal Rule
    assert!(validate_competition_fidelity(&event), "Event should be valid for 2025 (min_dances=3)");

    // 2. Verify 2026 Temporal Rule (should fail)
    event.date = Some(NaiveDate::from_ymd_opt(2026, 3, 4).unwrap());
    event.min_dances = dancing_datacollection::models::validation::get_min_dances_for_level(
        &event.level,
        &event.date.unwrap()
    );
    assert!(!validate_competition_fidelity(&event), "Event should be invalid for 2026 (min_dances=4 required for Level D)");
}

#[test]
fn test_fidelity_corruption() {
    use dancing_datacollection::models::validation::validate_competition_fidelity;
    use dancing_datacollection::models::RoundEnum;
    let dir_name = "3-0407_ot_mas2dlat";
    let dir_path = std::path::Path::new("tests").join(dir_name);
    let mut event = dancing_datacollection::sources::dtv_native::extract_event_data(dir_path.to_str().unwrap()).unwrap();

    // Verify initially valid
    assert!(validate_competition_fidelity(&event), "Original event should be valid");

    // Corrupt by removing judges (Integrity Layer: < 3 judges)
    let mut corrupt_judges = event.clone();
    corrupt_judges.officials.judges.truncate(2);
    assert!(!validate_competition_fidelity(&corrupt_judges), "Event should be invalid with only 2 judges");

    // Corrupt by removing a scoring record from a round
    let mut corrupt_round = event.clone();
    let first_judge_code = corrupt_round.officials.judges[0].code.clone();
    let mut corrupted = false;
    for round in &mut corrupt_round.rounds {
        match round {
            RoundEnum::Mark(r) => {
                 if r.marking_crosses.contains_key(&first_judge_code) {
                      r.marking_crosses.remove(&first_judge_code);
                      corrupted = true;
                      break;
                 }
            }
            RoundEnum::DTV(r) => {
                if r.dtv_ranks.contains_key(&first_judge_code) {
                     r.dtv_ranks.remove(&first_judge_code);
                     corrupted = true;
                     break;
                }
            }
            RoundEnum::WDSF(r) => {
                if r.wdsf_scores.contains_key(&first_judge_code) {
                     r.wdsf_scores.remove(&first_judge_code);
                     corrupted = true;
                     break;
                }
            }
        }
    }
    assert!(corrupted, "Should have found a round to corrupt");
    assert!(!validate_competition_fidelity(&corrupt_round), "Event should be invalid if a judge's records are missing in a round");
}
