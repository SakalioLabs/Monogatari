use super::*;
use crate::quality_suite_validation::parse_quality_suite_document;

fn checked_in_project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("data")
        .canonicalize()
        .expect("checked-in project root")
}

fn provenance() -> QualitySuiteRunProvenance {
    QualitySuiteRunProvenance {
        generated_at: "2026-07-14T00:00:00Z".to_string(),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: "test-commit".to_string(),
        git_short_commit: "test".to_string(),
    }
}

#[test]
fn checked_in_character_stability_suite_passes_without_tauri() {
    let root = checked_in_project_root();
    let suite = parse_quality_suite_document(include_str!(
        "../../../../../data/quality_suites/character_stability.json"
    ))
    .unwrap();
    let catalog = StoryEventCatalog::load_from_project_root(&root).unwrap();

    let report = execute_quality_suite(
        &suite,
        Some(&root),
        "quality_suites/character_stability.json",
        "a",
        &catalog,
        provenance(),
    );

    assert_eq!(report.failed, 0, "{:#?}", report.scenarios);
    assert_eq!(report.total, suite.scenarios.len());
    assert_eq!(report.run_metadata.generated_at, "2026-07-14T00:00:00Z");
}

#[test]
fn tideglass_quality_workflows_reach_full_coverage_without_tauri() {
    let root = checked_in_project_root();
    let suite = parse_quality_suite_document(include_str!(
        "../../../../../data/quality_suites/tideglass_acceptance.json"
    ))
    .unwrap();
    let catalog = StoryEventCatalog::load_from_project_root(&root).unwrap();

    let report = execute_quality_suite(
        &suite,
        Some(&root),
        "quality_suites/tideglass_acceptance.json",
        "b",
        &catalog,
        provenance(),
    );

    assert_eq!(report.failed, 0, "{:#?}", report.scenarios);
    let workflow_reports = report
        .scenarios
        .iter()
        .filter_map(|scenario| scenario.workflow_coverage.as_ref())
        .collect::<Vec<_>>();
    assert!(!workflow_reports.is_empty());
    assert!(workflow_reports
        .iter()
        .all(|coverage| coverage.coverage_percent == 100.0));
}

#[test]
fn workflow_choice_selections_reach_all_blue_frame_routes_without_tauri() {
    let root = checked_in_project_root();
    let suite = parse_quality_suite_document(
        r#"{
          "version":"1",
          "name":"Blue Frame Routes",
          "description":"Choice coverage",
          "scenarios":[{
            "id":"routes",
            "category":"workflow_coverage",
            "description":"Three routes",
            "workflow_path":"workflows/blue_frame_route.json",
            "workflow_max_steps":32,
            "workflow_choice_selections":[
              {"first_test":0,"classroom_response":0,"evidence_form":0,"publication_choice":0},
              {"first_test":1,"classroom_response":1,"evidence_form":1,"publication_choice":1},
              {"first_test":2,"classroom_response":2,"evidence_form":0,"publication_choice":2}
            ],
            "expect":{"min_workflow_coverage_percent":100,"expected_workflow_unvisited_nodes":[]}
          }]
        }"#,
    )
    .unwrap();
    let catalog = StoryEventCatalog::load_from_project_root(&root).unwrap();

    let report = execute_quality_suite(
        &suite,
        Some(&root),
        "quality_suites/blue_frame_routes.json",
        "c",
        &catalog,
        provenance(),
    );

    assert_eq!(report.failed, 0, "{:#?}", report.scenarios);
    let coverage = report.scenarios[0].workflow_coverage.as_ref().unwrap();
    assert_eq!(coverage.coverage_percent, 100.0);
    assert_eq!(coverage.run_count, 3);
    assert_eq!(coverage.runs[2].choice_selections["publication_choice"], 2);
}

#[test]
fn checked_in_blue_frame_roleplay_suite_reaches_all_dynamic_endings() {
    let root = checked_in_project_root();
    let suite = parse_quality_suite_document(include_str!(
        "../../../../../data/quality_suites/blue_frame_roleplay.json"
    ))
    .unwrap();
    let catalog = StoryEventCatalog::load_from_project_root(&root).unwrap();

    let report = execute_quality_suite(
        &suite,
        Some(&root),
        "quality_suites/blue_frame_roleplay.json",
        "d",
        &catalog,
        provenance(),
    );

    assert_eq!(report.failed, 0, "{:#?}", report.scenarios);
    assert_eq!(report.total, 3);
    assert_eq!(report.audit_summary.roleplay_coverage.len(), 3);
    assert!(report.scenarios.iter().all(|scenario| {
        scenario.roleplay_preview.as_ref().is_some_and(|preview| {
            preview.report.completed
                && preview.report.coverage_percent == 100.0
                && preview.source_sha256.len() == 64
        })
    }));
}

#[test]
fn failed_expectations_return_actionable_headless_evidence() {
    let suite = parse_quality_suite_document(
        r#"{"version":"1","name":"Failure","description":"Failure evidence","scenarios":[{"id":"low-score","category":"quality","description":"Must fail","messages":[{"role":"player","content":"Hello"}],"expect":{"min_overall":1.0}}]}"#,
    )
    .unwrap();

    let report = execute_quality_suite(
        &suite,
        None,
        "quality_suites/failure.json",
        "c",
        &StoryEventCatalog::default(),
        provenance(),
    );

    assert_eq!(report.failed, 1);
    assert!(report.scenarios[0]
        .issues
        .iter()
        .any(|issue| issue.contains("overall expected >= 1")));
    assert_eq!(
        report.audit_summary.failed_scenario_ids,
        ["low-score".to_string()]
    );
}
