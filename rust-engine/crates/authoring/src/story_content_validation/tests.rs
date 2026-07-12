use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::*;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "monogatari_story_content_{label}_{}_{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    for directory in ["scenes", "endings", "dialogue", "assets/backgrounds"] {
        std::fs::create_dir_all(root.join(directory)).unwrap();
    }
    root
}

fn write_valid(root: &Path) {
    std::fs::write(root.join("assets/backgrounds/finale.svg"), "<svg/>").unwrap();
    std::fs::write(root.join("endings/finale.json"), r#"{"schema":"monogatari-story-ending/v1","id":"finale","title":"Finale","description":"Done.","scene_id":"finale","dialogue_id":"finale_dialogue"}"#).unwrap();
}

#[test]
fn loads_endings_that_reference_inferred_background_scenes() {
    let root = root("inferred");
    write_valid(&root);
    let scenes = load_scene_documents(&root).unwrap();
    let ids = scene_ids(&root, &scenes).unwrap();
    let endings = load_story_ending_sources(&root).unwrap();
    assert!(ids.contains("finale"));
    assert_eq!(endings.len(), 1);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_missing_authored_backgrounds() {
    let root = root("missing_background");
    std::fs::write(
        root.join("scenes/park.json"),
        r#"{"id":"park","name":"Park","background_path":"assets/backgrounds/missing.png"}"#,
    )
    .unwrap();
    assert!(load_scene_documents(&root)
        .unwrap_err()
        .contains("does not exist"));
    std::fs::remove_dir_all(root).unwrap();
}
