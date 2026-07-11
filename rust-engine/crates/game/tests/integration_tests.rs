//! Integration tests for the full game workflow.

use llm_game::{
    characters::{Character, CharacterManager, Personality},
    dialogue::DialogueManager,
    knowledge::{KnowledgeBase, KnowledgeCategory, KnowledgeEntry},
    script::ScriptParser,
};
use std::path::Path;

#[tokio::test]
async fn test_full_character_workflow() {
    // Create a character
    let mut character = Character::new("sakura", "Sakura");
    character.description = "A cheerful girl who loves cherry blossoms".to_string();
    character.background = "Grew up in a small town".to_string();

    // Set personality
    character.personality.extraversion = 0.8;
    character.personality.agreeableness = 0.9;
    character.personality.speech_style = "cheerful".to_string();

    // Set emotion
    character.set_emotion("happy");
    assert_eq!(character.emotion, "happy");

    // Update relationship
    character.update_relationship("player", 0.2);
    assert_eq!(character.relationships.get("player"), Some(&0.2));

    // Build system prompt
    let prompt = character.build_system_prompt();
    assert!(prompt.contains("Sakura"));
    assert!(prompt.contains("cheerful"));
    assert!(prompt.contains("cherry blossoms"));
}

#[tokio::test]
async fn test_character_manager_workflow() {
    let mut manager = CharacterManager::new();

    // Add characters
    let mut char1 = Character::new("sakura", "Sakura");
    char1.description = "A cheerful girl".to_string();
    manager.add_character(char1);

    let mut char2 = Character::new("yuki", "Yuki");
    char2.description = "A quiet bookworm".to_string();
    manager.add_character(char2);

    // Verify characters
    assert_eq!(manager.character_ids().len(), 2);
    assert!(manager.get_character("sakura").is_some());
    assert!(manager.get_character("yuki").is_some());
    assert!(manager.get_character("unknown").is_none());
}

#[tokio::test]
async fn test_dialogue_workflow() {
    let mut dm = DialogueManager::new();

    // Load dialogue scripts from data directory
    let data_path = Path::new("data/dialogue");
    if data_path.exists() {
        let count = dm.load_from_directory(data_path).await.unwrap();
        assert!(count > 0);

        // Start dialogue
        dm.start_dialogue("meeting_sakura").await.unwrap();
        assert!(dm.is_active());

        // Get current context
        let context = dm.get_current_context();
        assert!(context.is_some());

        // Advance dialogue
        dm.advance().await.unwrap();
        assert!(dm.is_active());
    }
}

#[tokio::test]
async fn test_knowledge_base_workflow() {
    let mut kb = KnowledgeBase::new();

    // Add knowledge entries
    let mut entry1 = KnowledgeEntry::new(
        "park",
        KnowledgeCategory::Location,
        "Cherry Blossom Park",
        "A beautiful park with cherry blossom trees.",
    );
    entry1.tags = vec!["nature".to_string(), "peaceful".to_string()];
    entry1.importance = 0.8;

    let mut entry2 = KnowledgeEntry::new(
        "sakura",
        KnowledgeCategory::Character,
        "Sakura",
        "A cheerful girl who loves cherry blossoms.",
    );
    entry2.tags = vec!["character".to_string(), "friendly".to_string()];
    entry2.importance = 0.9;

    kb.add_entry(entry1);
    kb.add_entry(entry2);

    // Search
    let results = kb.search("cherry", 10);
    assert_eq!(results.len(), 2);

    // Search by tag
    let nature_entries = kb.get_by_tag("nature");
    assert_eq!(nature_entries.len(), 1);

    // Search by category
    let locations = kb.get_by_category(&KnowledgeCategory::Location);
    assert_eq!(locations.len(), 1);
}

#[test]
fn test_script_parser_workflow() {
    let script = r#"
; 这是一个完整的对话脚本
changeBg=park.png
Sakura:happy 你好！ -v=v1.ogg
这是一段旁白。
changeFigure=sakura.png -position=center
Sakura:今天天气真好！
choose=去散步:walk,聊天:chat
end
"#;

    let commands = ScriptParser::parse(script);

    // Verify command count
    assert!(commands.len() >= 6);

    // Verify first command is ChangeBg
    assert!(matches!(
        &commands[0],
        llm_game::script::ScriptCommand::ChangeBg { .. }
    ));

    // Verify last command is End
    assert!(matches!(
        commands.last().unwrap(),
        llm_game::script::ScriptCommand::End
    ));

    // Verify dialogue commands
    let say_commands: Vec<_> = commands
        .iter()
        .filter(|c| matches!(c, llm_game::script::ScriptCommand::Say { .. }))
        .collect();
    assert!(say_commands.len() >= 2);
}

#[test]
fn test_personality_system() {
    let personality = Personality {
        openness: 0.8,
        extraversion: 0.7,
        likes: vec!["cherry blossoms".to_string()],
        dislikes: vec!["rudeness".to_string()],
        speech_style: "cheerful".to_string(),
        ..Default::default()
    };

    // Build description
    let description = personality.to_prompt_description();
    assert!(description.contains("cheerful"));
    assert!(description.contains("cherry blossoms"));
    assert!(description.contains("rudeness"));
}

#[test]
fn test_error_context() {
    use llm_core::EngineError;

    // Test dialogue error with context
    let err = EngineError::dialogue("main", "start", "Node not found");
    let msg = err.to_string();
    assert!(msg.contains("main"));
    assert!(msg.contains("start"));
    assert!(msg.contains("Node not found"));

    // Test inference error with context
    let err = EngineError::inference("API", "Connection timeout");
    let msg = err.to_string();
    assert!(msg.contains("API"));
    assert!(msg.contains("Connection timeout"));

    // Test config error with context
    let err = EngineError::config("api_key", "Invalid key format");
    let msg = err.to_string();
    assert!(msg.contains("api_key"));
    assert!(msg.contains("Invalid key format"));
}

#[test]
fn test_multiple_script_formats() {
    // Test narrator dialogue
    let commands = ScriptParser::parse("这是一段旁白。");
    assert_eq!(commands.len(), 1);

    // Test character dialogue
    let commands = ScriptParser::parse("Sakura:你好！");
    assert_eq!(commands.len(), 1);

    // Test dialogue with emotion
    let commands = ScriptParser::parse("Sakura:happy 今天天气真好！");
    assert_eq!(commands.len(), 1);

    // Test dialogue with voice
    let commands = ScriptParser::parse("Sakura:你好 -v=v1.ogg");
    assert_eq!(commands.len(), 1);

    // Test commands
    let commands = ScriptParser::parse("changeBg=park.png");
    assert_eq!(commands.len(), 1);

    // Test choices
    let commands = ScriptParser::parse("choose=是:yes,否:no");
    assert_eq!(commands.len(), 1);
}
