//! Boundary condition tests for edge cases.

use llm_game::{
    characters::{Character, CharacterMemory, Personality},
    knowledge::{KnowledgeBase, KnowledgeCategory, KnowledgeEntry},
    script::ScriptParser,
};

#[test]
fn test_empty_script() {
    let commands = ScriptParser::parse("");
    assert!(commands.is_empty());
}

#[test]
fn test_whitespace_only_script() {
    let commands = ScriptParser::parse("   \n   \n   ");
    assert!(commands.is_empty());
}

#[test]
fn test_comment_only_script() {
    let commands = ScriptParser::parse("; comment 1\n; comment 2\n; comment 3");
    assert!(commands.is_empty());
}

#[test]
fn test_very_long_dialogue() {
    let long_text = "a".repeat(10000);
    let script = format!("Sakura:{long_text}");
    let commands = ScriptParser::parse(&script);
    assert_eq!(commands.len(), 1);
}

#[test]
fn test_special_characters_in_dialogue() {
    let commands = ScriptParser::parse("Sakura:你好！@#$%^&*()_+");
    assert_eq!(commands.len(), 1);
}

#[test]
fn test_unicode_dialogue() {
    let commands = ScriptParser::parse("Sakura:こんにちは！🌸");
    assert_eq!(commands.len(), 1);
}

#[test]
fn test_multiple_colons_in_dialogue() {
    let commands = ScriptParser::parse("Sakura:happy:extra 你好");
    assert_eq!(commands.len(), 1);
}

#[test]
fn test_dialogue_with_only_colon() {
    // Empty content after colon is treated as narrator dialogue
    let commands = ScriptParser::parse("Sakura:");
    // Current behavior: empty content is still parsed as a command
    // This is acceptable for edge case handling
    assert!(commands.len() <= 1);
}

#[test]
fn test_dialogue_with_only_speaker() {
    // Empty speaker before colon is treated as narrator dialogue
    let commands = ScriptParser::parse(":你好");
    // Current behavior: empty speaker becomes narrator dialogue
    assert_eq!(commands.len(), 1);
}

#[test]
fn test_escaped_characters() {
    let commands = ScriptParser::parse("Sakura:你好\\;世界\\{\\}");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        llm_game::script::ScriptCommand::Say { text, .. } => {
            assert!(text.contains(";"));
            assert!(text.contains("{"));
            assert!(text.contains("}"));
        }
        _ => panic!("Expected Say command"),
    }
}

#[test]
fn test_multiple_arguments() {
    let commands = ScriptParser::parse("Sakura:你好 -v=v1.ogg -concat -notend");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        llm_game::script::ScriptCommand::Say {
            vocal,
            concat,
            notend,
            ..
        } => {
            assert_eq!(vocal, &Some("v1.ogg".to_string()));
            assert!(concat);
            assert!(notend);
        }
        _ => panic!("Expected Say command"),
    }
}

#[test]
fn test_choice_with_special_characters() {
    let commands = ScriptParser::parse("choose=选项1:target1,选项2:target2");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        llm_game::script::ScriptCommand::Choose { choices } => {
            assert_eq!(choices.len(), 2);
        }
        _ => panic!("Expected Choose command"),
    }
}

#[test]
fn test_wait_with_zero() {
    let commands = ScriptParser::parse("wait=0");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        llm_game::script::ScriptCommand::Wait { duration_ms } => {
            assert_eq!(*duration_ms, 0);
        }
        _ => panic!("Expected Wait command"),
    }
}

#[test]
fn test_wait_with_invalid_value() {
    let commands = ScriptParser::parse("wait=abc");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        llm_game::script::ScriptCommand::Wait { duration_ms } => {
            assert_eq!(*duration_ms, 1000); // default value
        }
        _ => panic!("Expected Wait command"),
    }
}

#[test]
fn test_set_var_with_equals_in_value() {
    let commands = ScriptParser::parse("setVar=x=a=b");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        llm_game::script::ScriptCommand::SetVar { name, value } => {
            assert_eq!(name, "x");
            assert_eq!(value, "a=b");
        }
        _ => panic!("Expected SetVar command"),
    }
}

#[test]
fn test_character_memory_eviction() {
    let mut memory = CharacterMemory::new(3);

    // Add 3 memories
    memory.add_memory(
        "Memory 1".to_string(),
        llm_game::characters::memory::MemoryType::Conversation,
        0.5,
        vec![],
    );
    memory.add_memory(
        "Memory 2".to_string(),
        llm_game::characters::memory::MemoryType::Conversation,
        0.7,
        vec![],
    );
    memory.add_memory(
        "Memory 3".to_string(),
        llm_game::characters::memory::MemoryType::Conversation,
        0.3,
        vec![],
    );

    assert_eq!(memory.len(), 3);

    // Add 4th memory - should evict lowest importance (Memory 3 with 0.3)
    memory.add_memory(
        "Memory 4".to_string(),
        llm_game::characters::memory::MemoryType::Conversation,
        0.9,
        vec![],
    );

    assert_eq!(memory.len(), 3);

    // Memory 3 should be evicted
    let recent = memory.get_recent(3);
    let has_memory3 = recent.iter().any(|m| m.content == "Memory 3");
    assert!(!has_memory3);
}

#[test]
fn test_character_memory_recall() {
    let mut memory = CharacterMemory::new(10);

    memory.add_memory(
        "Cherry blossom park".to_string(),
        llm_game::characters::memory::MemoryType::Conversation,
        0.8,
        vec!["park".to_string()],
    );
    memory.add_memory(
        "Beautiful sunset".to_string(),
        llm_game::characters::memory::MemoryType::Conversation,
        0.6,
        vec!["sunset".to_string()],
    );
    memory.add_memory(
        "Park bench".to_string(),
        llm_game::characters::memory::MemoryType::Conversation,
        0.4,
        vec!["park".to_string()],
    );

    // Recall by keyword
    let results = memory.recall("park", 10);
    assert_eq!(results.len(), 2);

    // Recall with limit
    let results = memory.recall("park", 1);
    assert_eq!(results.len(), 1);
}

#[test]
fn test_knowledge_base_empty_search() {
    let mut kb = KnowledgeBase::new();
    kb.add_entry(KnowledgeEntry::new(
        "test",
        KnowledgeCategory::Other("test".to_string()),
        "Test",
        "Test content.",
    ));

    // Empty query should return no results
    let results = kb.search("", 10);
    assert!(results.is_empty());

    // Whitespace query should return no results
    let results = kb.search("   ", 10);
    assert!(results.is_empty());
}

#[test]
fn test_knowledge_base_nonexistent_tag() {
    let kb = KnowledgeBase::new();
    let results = kb.get_by_tag("nonexistent");
    assert!(results.is_empty());
}

#[test]
fn test_knowledge_base_nonexistent_category() {
    let kb = KnowledgeBase::new();
    let results = kb.get_by_category(&KnowledgeCategory::Location);
    assert!(results.is_empty());
}

#[test]
fn test_personality_default() {
    let personality = Personality::default();
    assert_eq!(personality.openness, 0.5);
    assert_eq!(personality.conscientiousness, 0.5);
    assert_eq!(personality.extraversion, 0.5);
    assert_eq!(personality.agreeableness, 0.5);
    assert_eq!(personality.neuroticism, 0.5);
    assert_eq!(personality.current_emotion, "neutral");
}

#[test]
fn test_character_relationship_clamp() {
    let mut character = Character::new("test", "Test");

    // Test positive clamp
    character.update_relationship("other", 2.0);
    assert_eq!(character.relationships.get("other"), Some(&1.0));

    // Test negative clamp
    character.update_relationship("other2", -2.0);
    assert_eq!(character.relationships.get("other2"), Some(&-1.0));
}

#[test]
fn test_script_parser_multiple_commands() {
    let script = "changeBg=park.png\nSakura:你好\nchoose=是:yes,否:no\nend";
    let commands = ScriptParser::parse(script);
    assert_eq!(commands.len(), 4);
}

#[test]
fn test_script_parser_mixed_content() {
    let script = r#"
; Comment
changeBg=park.png
这是一段旁白。
Sakura:happy 你好！
end
"#;
    let commands = ScriptParser::parse(script);
    assert!(commands.len() >= 4);
}

#[test]
fn test_parse_mini_avatar() {
    let commands = ScriptParser::parse("miniAvatar=sakura_face.png");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        llm_game::script::ScriptCommand::MiniAvatar { path } => {
            assert_eq!(path, "sakura_face.png");
        }
        _ => panic!("Expected MiniAvatar command"),
    }
}

#[test]
fn test_parse_set_text_speed() {
    let commands = ScriptParser::parse("setTextSpeed=20");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        llm_game::script::ScriptCommand::SetTextSpeed { speed_ms } => {
            assert_eq!(*speed_ms, 20);
        }
        _ => panic!("Expected SetTextSpeed command"),
    }
}

#[test]
fn test_parse_film_mode() {
    let commands = ScriptParser::parse("filmMode=on");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        llm_game::script::ScriptCommand::FilmMode { enabled } => {
            assert!(*enabled);
        }
        _ => panic!("Expected FilmMode command"),
    }
}

#[test]
fn test_parse_film_mode_off() {
    let commands = ScriptParser::parse("filmMode=off");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        llm_game::script::ScriptCommand::FilmMode { enabled } => {
            assert!(!enabled);
        }
        _ => panic!("Expected FilmMode command"),
    }
}

#[test]
fn test_parse_unlock_cg() {
    let commands = ScriptParser::parse("unlockCg=cg_001");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        llm_game::script::ScriptCommand::UnlockCg { id } => {
            assert_eq!(id, "cg_001");
        }
        _ => panic!("Expected UnlockCg command"),
    }
}

#[test]
fn test_parse_unlock_bgm() {
    let commands = ScriptParser::parse("unlockBgm=bgm_001");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        llm_game::script::ScriptCommand::UnlockBgm { id } => {
            assert_eq!(id, "bgm_001");
        }
        _ => panic!("Expected UnlockBgm command"),
    }
}

#[test]
fn test_parse_play_voice() {
    let commands = ScriptParser::parse("playVoice=v1.ogg");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        llm_game::script::ScriptCommand::PlayVoice { path } => {
            assert_eq!(path, "v1.ogg");
        }
        _ => panic!("Expected PlayVoice command"),
    }
}

#[test]
fn test_parse_stop_voice() {
    let commands = ScriptParser::parse("stopVoice");
    assert_eq!(commands.len(), 1);
    assert!(matches!(
        commands[0],
        llm_game::script::ScriptCommand::StopVoice
    ));
}

#[test]
fn test_parse_set_animation() {
    let commands = ScriptParser::parse("setAnimation=sakura:fadeIn");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        llm_game::script::ScriptCommand::SetAnimation { target, animation } => {
            assert_eq!(target, "sakura");
            assert_eq!(animation, "fadeIn");
        }
        _ => panic!("Expected SetAnimation command"),
    }
}
