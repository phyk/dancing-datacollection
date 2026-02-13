use std::path::Path;

fn run_full_pipeline_test(dir_name: &str) {
    let dir_path = Path::new("tests").join(dir_name);
    let event = dancing_datacollection::sources::dtv_native::extract_event_data(dir_path.to_str().unwrap()).unwrap();

    assert!(!event.participants.is_empty(), "No participants parsed in {}", dir_name);
    assert!(!event.officials.judges.is_empty(), "No judges parsed in {}", dir_name);
    assert!(!event.rounds.is_empty(), "No rounds parsed in {}", dir_name);

    // Verify against Ground Truth if it exists
    let ground_truth_path = dir_path.join("tabges.jsonl");
    if ground_truth_path.exists() {
        let ground_truth_json = std::fs::read_to_string(&ground_truth_path).unwrap();
        let expected: dancing_datacollection::models::Competition = serde_json::from_str(ground_truth_json.trim()).unwrap();

        assert_eq!(event.name, expected.name, "Name mismatch for {}", dir_name);
        assert_eq!(event.date, expected.date, "Date mismatch for {}", dir_name);
        assert_eq!(event.organizer, expected.organizer, "Organizer mismatch for {}", dir_name);
        assert_eq!(event.hosting_club, expected.hosting_club, "Hosting club mismatch for {}", dir_name);
        assert_eq!(event.level, expected.level);
        assert_eq!(event.age_group, expected.age_group);
        assert_eq!(event.style, expected.style);
        assert_eq!(event.dances, expected.dances);
        assert_eq!(event.min_dances, expected.min_dances);
        assert_eq!(event.officials, expected.officials);
        assert_eq!(event.participants, expected.participants);
        assert_eq!(event.rounds, expected.rounds);
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
