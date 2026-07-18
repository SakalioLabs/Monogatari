use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

const MINIMAL_SUITE_JSON: &str = r#"{"version":"1","name":"Smoke","description":"Smoke suite","scenarios":[{"id":"base","category":"story","description":"Base","expect":{}}]}"#;

static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(0);

struct TestProject {
    root: PathBuf,
}

impl TestProject {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "monogatari-quality-source-{label}-{}-{}",
            std::process::id(),
            NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn write(&self, path: &str, content: &str) {
        let target = self.root.join(path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(target, content).unwrap();
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn parses_a_minimal_quality_suite_without_tauri_runtime() {
    let suite = parse_quality_suite_document(MINIMAL_SUITE_JSON).unwrap();

    assert_eq!(suite.scenarios.len(), 1);
}

#[test]
fn parses_a_quality_suite_value_without_reserializing_it() {
    let value = serde_json::from_str(MINIMAL_SUITE_JSON).unwrap();
    let suite = parse_quality_suite_value(value).unwrap();

    assert_eq!(suite.name, "Smoke");
    assert_eq!(suite.scenarios[0].id, "base");
}

#[test]
fn loads_exact_quality_sources_with_stable_hashes_and_sorted_summaries() {
    let project = TestProject::new("load");
    let zulu = MINIMAL_SUITE_JSON.replace("Smoke", "Zulu");
    let alpha = MINIMAL_SUITE_JSON.replace("Smoke", "Alpha");
    project.write("quality_suites/zulu.json", &zulu);
    project.write("quality_suites/alpha.json", &alpha);

    let loaded =
        load_project_quality_suite_document(&project.root, "quality_suites/zulu.json").unwrap();
    assert_eq!(loaded.source_path, "quality_suites/zulu.json");
    assert_eq!(loaded.source_sha256, quality_suite_sha256(&zulu));
    assert_eq!(
        loaded.absolute_path,
        project
            .root
            .canonicalize()
            .unwrap()
            .join("quality_suites/zulu.json")
    );

    let summaries = list_project_quality_suite_summaries(&project.root).unwrap();
    assert_eq!(
        summaries
            .iter()
            .map(|summary| summary.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Alpha", "Zulu"]
    );
    assert_eq!(summaries[1].path, "quality_suites/zulu.json");
    assert_eq!(summaries[1].suite_sha256, quality_suite_sha256(&zulu));
}

#[test]
fn rejects_non_quality_oversized_and_case_aliased_sources() {
    let project = TestProject::new("boundaries");
    project.write("characters/smoke.json", MINIMAL_SUITE_JSON);
    project.write("quality_suites/smoke.json", MINIMAL_SUITE_JSON);
    let oversized = format!(
        r#"{{"version":"1","name":"Large","description":"{}","scenarios":[{{"id":"base","category":"story","description":"Base","expect":{{}}}}]}}"#,
        "a".repeat(MAX_QUALITY_SUITE_FILE_BYTES as usize)
    );
    project.write("quality_suites/large.json", &oversized);

    let wrong_catalog =
        load_project_quality_suite_document(&project.root, "characters/smoke.json").unwrap_err();
    assert!(matches!(
        wrong_catalog,
        QualitySuiteSourceError::WrongCatalog {
            actual_catalog: AuthorableJsonCatalog::Characters,
            ..
        }
    ));

    let too_large = load_project_quality_suite_document(&project.root, "quality_suites/large.json")
        .unwrap_err();
    assert!(matches!(
        too_large,
        QualitySuiteSourceError::FileTooLarge {
            max_size_bytes: MAX_QUALITY_SUITE_FILE_BYTES,
            ..
        }
    ));

    let case_alias =
        load_project_quality_suite_document(&project.root, "quality_suites/Smoke.json")
            .unwrap_err();
    assert!(matches!(
        case_alias,
        QualitySuiteSourceError::Catalog(JsonCatalogError {
            code: JsonCatalogErrorCode::PathCaseCollision,
            ..
        })
    ));
}

#[test]
fn distinguishes_missing_quality_sources_for_builtin_fallbacks() {
    let project = TestProject::new("missing");
    let error = load_project_quality_suite_document(
        &project.root,
        "quality_suites/character_stability.json",
    )
    .unwrap_err();

    assert!(error.is_missing());
    assert!(!QualitySuiteSourceError::InvalidDocument {
        path: "quality_suites/broken.json".into(),
        message: "broken".into(),
    }
    .is_missing());
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
fn rejects_malformed_typed_workflow_run_contexts_without_tauri_runtime() {
    let malformed_contexts = [
        r#"{"enabled":"yes"}"#,
        r#"{"relationship":"close"}"#,
        r#"{"already_triggered_events":[42]}"#,
    ];

    for context in malformed_contexts {
        let content = format!(
            r#"{{"version":"1","name":"Broken","description":"Broken context","scenarios":[{{"id":"base","category":"workflow","description":"Base","workflow_run_contexts":[{context}],"expect":{{}}}}]}}"#
        );

        assert!(
            parse_quality_suite_document(&content).is_err(),
            "workflow context should be rejected: {context}"
        );
    }
}

#[test]
fn validates_bounded_per_run_workflow_choice_selections() {
    let suite = parse_quality_suite_document(
        r#"{"version":"1","name":"Choices","description":"Choice coverage","scenarios":[{"id":"routes","category":"workflow_coverage","description":"Routes","workflow_path":"workflows/routes.json","workflow_choice_selections":[{"route":0},{"route":1}],"expect":{"min_workflow_coverage_percent":100}}]}"#,
    )
    .unwrap();
    assert_eq!(suite.scenarios[0].workflow_choice_selections.len(), 2);

    let mismatched = parse_quality_suite_document(
        r#"{"version":"1","name":"Choices","description":"Choice coverage","scenarios":[{"id":"routes","category":"workflow_coverage","description":"Routes","workflow_path":"workflows/routes.json","workflow_run_contexts":[{"enabled":true}],"workflow_choice_selections":[{"route":0},{"route":1}],"expect":{}}]}"#,
    )
    .unwrap_err();
    assert!(mismatched.contains("must contain the same number of runs"));

    let invalid_node = parse_quality_suite_document(
        r#"{"version":"1","name":"Choices","description":"Choice coverage","scenarios":[{"id":"routes","category":"workflow_coverage","description":"Routes","workflow_path":"workflows/routes.json","workflow_choice_selections":[{"../route":0}],"expect":{}}]}"#,
    )
    .unwrap_err();
    assert!(invalid_node.contains("invalid node id"));
}

#[test]
fn reports_cross_catalog_quality_references() {
    let suite = parse_quality_suite_document(
        r#"{"version":"1","name":"Refs","description":"Reference suite","scenarios":[{"id":"base","category":"story","description":"Base","character_id":"missing","workflow_path":"workflows/missing.json","roleplay":{"path":"roleplays/missing.json"},"expect":{"required_knowledge_refs":["missing_lore"],"expected_events":["missing_event"]}}]}"#,
    )
    .unwrap();
    let loaded = vec![LoadedQualitySuiteDocument {
        suite,
        source_path: "quality_suites/refs.json".into(),
        source_sha256: "0".repeat(64),
        absolute_path: PathBuf::from("refs.json"),
    }];

    let issues = validate_quality_suite_references(
        &loaded,
        &HashSet::new(),
        &HashSet::new(),
        &HashSet::new(),
        &HashSet::new(),
        &HashSet::new(),
    );

    assert_eq!(issues.len(), 5);
    assert!(issues
        .iter()
        .any(|(code, _, _)| code == "quality_roleplay_missing"));
}

#[test]
fn validates_typed_scene_roleplay_fixtures_and_expectations() {
    let suite = parse_quality_suite_document(
        r#"{"version":"1","name":"Roleplay","description":"Roleplay coverage","scenarios":[{"id":"route","category":"scene_roleplay","description":"Route","roleplay":{"path":"roleplays/route.json","turns":[]},"expect":{"expected_roleplay_ending":"ending","min_roleplay_coverage_percent":100,"required_roleplay_nodes":["start"],"min_roleplay_scores":{"trust":1.0}}}]}"#,
    )
    .unwrap();
    assert_eq!(
        suite.scenarios[0].roleplay.as_ref().unwrap().path,
        "roleplays/route.json"
    );

    let missing_fixture = parse_quality_suite_document(
        r#"{"version":"1","name":"Roleplay","description":"Roleplay coverage","scenarios":[{"id":"route","category":"scene_roleplay","description":"Route","expect":{"expected_roleplay_ending":"ending"}}]}"#,
    )
    .unwrap_err();
    assert!(missing_fixture.contains("require a roleplay fixture"));

    let invalid_bounds = parse_quality_suite_document(
        r#"{"version":"1","name":"Roleplay","description":"Roleplay coverage","scenarios":[{"id":"route","category":"scene_roleplay","description":"Route","roleplay":{"path":"roleplays/route.json"},"expect":{"min_roleplay_coverage_percent":101,"min_roleplay_scores":{"trust":2.0},"max_roleplay_scores":{"trust":1.0}}}]}"#,
    )
    .unwrap_err();
    assert!(invalid_bounds.contains("min_roleplay_coverage_percent"));
    assert!(invalid_bounds.contains("exceeds its maximum"));
}
