use super::*;

#[test]
fn parses_a_minimal_quality_suite_without_tauri_runtime() {
    let suite = parse_quality_suite_document(
        r#"{"version":"1","name":"Smoke","description":"Smoke suite","scenarios":[{"id":"base","category":"story","description":"Base","expect":{}}]}"#,
    )
    .unwrap();

    assert_eq!(suite.scenarios.len(), 1);
}

#[test]
fn rejects_conflicting_expectations_and_invalid_score_ranges() {
    let error = parse_quality_suite_document(
        r#"{"version":"1","name":"Broken","description":"Broken suite","scenarios":[{"id":"base","category":"story","description":"Base","expect":{"min_overall":1.2,"expected_events":["same"],"forbidden_events":["same"]}}]}"#,
    )
    .unwrap_err();

    assert!(error.contains("min_overall"));
    assert!(error.contains("cannot appear in both"));
}

#[test]
fn reports_cross_catalog_quality_references() {
    let suite = parse_quality_suite_document(
        r#"{"version":"1","name":"Refs","description":"Reference suite","scenarios":[{"id":"base","category":"story","description":"Base","character_id":"missing","workflow_path":"workflows/missing.json","expect":{"required_knowledge_refs":["missing_lore"],"expected_events":["missing_event"]}}]}"#,
    )
    .unwrap();
    let loaded = vec![LoadedQualitySuiteDocument {
        suite,
        source_path: "quality_suites/refs.json".into(),
        absolute_path: PathBuf::from("refs.json"),
    }];

    let issues = validate_quality_suite_references(
        &loaded,
        &HashSet::new(),
        &HashSet::new(),
        &HashSet::new(),
        &HashSet::new(),
    );

    assert_eq!(issues.len(), 4);
}
