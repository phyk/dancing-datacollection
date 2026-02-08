use std::fs;
use std::path::Path;
use serde_json;

fn run_full_pipeline_test(dir_name: &str) {
    let dir_path = Path::new("tests").join(dir_name);
    let event = dancing_datacollection::sources::dtv_native::extract_event_data(dir_path.to_str().unwrap()).unwrap();

    for comp in &event.competitions_list {
        assert!(!comp.participants.is_empty(), "No participants parsed for level {:?} in {}", comp.level, dir_name);
        assert!(!comp.officials.judges.is_empty(), "No judges parsed for level {:?} in {}", comp.level, dir_name);
        assert!(!comp.rounds.is_empty(), "No rounds parsed for level {:?} in {}", comp.level, dir_name);
    }

    // Verify against Ground Truth if it exists
    let ground_truth_path = dir_path.join("tabges.jsonl");
    if ground_truth_path.exists() {
        let ground_truth_json = fs::read_to_string(ground_truth_path).unwrap();
        let expected_event: dancing_datacollection::models::Event = serde_json::from_str(ground_truth_json.trim()).unwrap();

        assert_eq!(event.name, expected_event.name, "Event name mismatch for {}", dir_name);
        assert_eq!(event.date, expected_event.date, "Event date mismatch for {}", dir_name);
        assert_eq!(event.organizer, expected_event.organizer, "Event organizer mismatch for {}", dir_name);
        assert_eq!(event.hosting_club, expected_event.hosting_club, "Event hosting_club mismatch for {}", dir_name);
        assert_eq!(event.competitions_list.len(), expected_event.competitions_list.len(), "Competitions list length mismatch for {}", dir_name);

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
#[test] fn test_integration_51_dtv_d_std() { run_full_pipeline_test("51-1105_ot_hgr2dstd"); }
#[test] fn test_integration_52_dtv_c_std() { run_full_pipeline_test("52-1105_ot_hgr2cstd"); }
#[test] fn test_integration_53_dtv_b_std() { run_full_pipeline_test("53-1105_ot_hgr2bstd"); }

#[test]
fn test_golden_file_03_dtv_d_lat() {
    use dancing_datacollection::models::validation::validate_event_fidelity;
    use chrono::NaiveDate;

    let dir_name = "3-0407_ot_mas2dlat";
    let dir_path = Path::new("tests").join(dir_name);

    let mut event = dancing_datacollection::sources::dtv_native::extract_event_data(dir_path.to_str().unwrap()).unwrap();

    // 1. Verify against Ground Truth
    let ground_truth_path = dir_path.join("tabges.jsonl");
    let ground_truth_json = fs::read_to_string(ground_truth_path).unwrap();
    let expected_event: dancing_datacollection::models::Event = serde_json::from_str(ground_truth_json.trim()).unwrap();
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
            &comp.level,
            &event.date.unwrap()
        );
    }
    assert!(!validate_event_fidelity(&event), "Event should be invalid for 2026 (min_dances=4 required for Level D)");
}

#[test]
fn test_fidelity_corruption() {
    use dancing_datacollection::models::validation::validate_event_fidelity;
    let dir_name = "3-0407_ot_mas2dlat";
    let dir_path = std::path::Path::new("tests").join(dir_name);
    let event = dancing_datacollection::sources::dtv_native::extract_event_data(dir_path.to_str().unwrap()).unwrap();

    // Verify initially valid
    assert!(validate_event_fidelity(&event), "Original event should be valid");

    // Corrupt by removing judges (Integrity Layer: < 3 judges)
    let mut corrupt_judges = event.clone();
    corrupt_judges.competitions_list[0].officials.judges.truncate(2);
    assert!(!validate_event_fidelity(&corrupt_judges), "Event should be invalid with only 2 judges");

    // Corrupt by removing a scoring record from a round
    let mut corrupt_round = event.clone();
    let comp = &mut corrupt_round.competitions_list[0];
    let first_judge_code = comp.officials.judges[0].code.clone();
    let mut corrupted = false;
    for round in &mut comp.rounds {
        if let Some(ref mut crosses) = round.marking_crosses {
            if crosses.contains_key(&first_judge_code) {
                crosses.remove(&first_judge_code);
                corrupted = true;
                break;
            }
        }
        if let Some(ref mut ranks) = round.dtv_ranks {
            if ranks.contains_key(&first_judge_code) {
                ranks.remove(&first_judge_code);
                corrupted = true;
                break;
            }
        }
    }
    assert!(corrupted, "Should have found a round to corrupt");
    assert!(!validate_event_fidelity(&corrupt_round), "Event should be invalid if a judge's records are missing in a round");
}
