//! Tests for the script DSL parser.

use llm_game::script::ScriptParser;
use llm_game::script::ScriptCommand;

#[test]
fn test_parse_empty_script() {
    let commands = ScriptParser::parse("");
    assert!(commands.is_empty());
}

#[test]
fn test_parse_comment_only() {
    let commands = ScriptParser::parse("; this is a comment\n; another comment");
    assert!(commands.is_empty());
}

#[test]
fn test_parse_narrator_dialogue() {
    let commands = ScriptParser::parse("这是一段旁白。");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::Say { speaker, text, .. } => {
            assert_eq!(speaker, &None);
            assert_eq!(text, "这是一段旁白。");
        }
        _ => panic!("Expected Say command"),
    }
}

#[test]
fn test_parse_speaker_dialogue() {
    let commands = ScriptParser::parse("Sakura:你好！");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::Say { speaker, text, .. } => {
            assert_eq!(speaker, &Some("Sakura".to_string()));
            assert_eq!(text, "你好！");
        }
        _ => panic!("Expected Say command"),
    }
}

#[test]
fn test_parse_dialogue_with_vocal() {
    let commands = ScriptParser::parse("Sakura:你好 -v=v1.ogg");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::Say { speaker, text, vocal, .. } => {
            assert_eq!(speaker, &Some("Sakura".to_string()));
            assert_eq!(text, "你好");
            assert_eq!(vocal, &Some("v1.ogg".to_string()));
        }
        _ => panic!("Expected Say command"),
    }
}

#[test]
fn test_parse_dialogue_with_emotion() {
    let commands = ScriptParser::parse("Sakura:happy 今天天气真好！");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::Say { speaker, text, emotion, .. } => {
            assert_eq!(speaker, &Some("Sakura".to_string()));
            assert_eq!(text, "今天天气真好！");
            assert_eq!(emotion, &Some("happy".to_string()));
        }
        _ => panic!("Expected Say command"),
    }
}

#[test]
fn test_parse_change_bg() {
    let commands = ScriptParser::parse("changeBg=park.png");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::ChangeBg { path, .. } => {
            assert_eq!(path, "park.png");
        }
        _ => panic!("Expected ChangeBg command"),
    }
}

#[test]
fn test_parse_change_figure() {
    let commands = ScriptParser::parse("changeFigure=sakura.png -position=center -live2d");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::ChangeFigure { path, position, live2d, .. } => {
            assert_eq!(path, "sakura.png");
            assert_eq!(position, &Some("center".to_string()));
            assert!(live2d);
        }
        _ => panic!("Expected ChangeFigure command"),
    }
}

#[test]
fn test_parse_bgm() {
    let commands = ScriptParser::parse("bgm=bgm1.mp3");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::Bgm { path } => {
            assert_eq!(path, &Some("bgm1.mp3".to_string()));
        }
        _ => panic!("Expected Bgm command"),
    }
}

#[test]
fn test_parse_choose() {
    let commands = ScriptParser::parse("choose=去公园:park,回家:home");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::Choose { choices } => {
            assert_eq!(choices.len(), 2);
            assert_eq!(choices[0].text, "去公园");
            assert_eq!(choices[0].target, "park");
            assert_eq!(choices[1].text, "回家");
            assert_eq!(choices[1].target, "home");
        }
        _ => panic!("Expected Choose command"),
    }
}

#[test]
fn test_parse_end() {
    let commands = ScriptParser::parse("end");
    assert_eq!(commands.len(), 1);
    assert!(matches!(commands[0], ScriptCommand::End));
}

#[test]
fn test_parse_set_var() {
    let commands = ScriptParser::parse("setVar=x=10");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::SetVar { name, value } => {
            assert_eq!(name, "x");
            assert_eq!(value, "10");
        }
        _ => panic!("Expected SetVar command"),
    }
}

#[test]
fn test_parse_label() {
    let commands = ScriptParser::parse("label=start");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::Label { name } => {
            assert_eq!(name, "start");
        }
        _ => panic!("Expected Label command"),
    }
}

#[test]
fn test_parse_jump_label() {
    let commands = ScriptParser::parse("jumpLabel=end");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::JumpLabel { label } => {
            assert_eq!(label, "end");
        }
        _ => panic!("Expected JumpLabel command"),
    }
}

#[test]
fn test_parse_wait() {
    let commands = ScriptParser::parse("wait=1000");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::Wait { duration_ms } => {
            assert_eq!(*duration_ms, 1000);
        }
        _ => panic!("Expected Wait command"),
    }
}

#[test]
fn test_parse_full_script() {
    let script = r#"
; 这是一个完整的测试脚本
changeBg=park.png -fadeIn
Sakura:happy 你好！ -v=v1.ogg
这是一段旁白。
changeFigure=sakura.png -position=center
Sakura:今天天气真好！
choose=去散步:walk,聊天:chat
end
"#;
    let commands = ScriptParser::parse(script);
    assert!(commands.len() >= 6);
    
    // Verify first command is ChangeBg
    assert!(matches!(&commands[0], ScriptCommand::ChangeBg { .. }));
    
    // Verify last command is End
    assert!(matches!(commands.last().unwrap(), ScriptCommand::End));
}

#[test]
fn test_parse_inline_comment() {
    let commands = ScriptParser::parse("Sakura:你好 ;这是注释");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::Say { text, .. } => {
            assert_eq!(text, "你好");
        }
        _ => panic!("Expected Say command"),
    }
}

#[test]
fn test_parse_escaped_chars() {
    let commands = ScriptParser::parse("Sakura:你好\\;世界");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::Say { text, .. } => {
            assert_eq!(text, "你好;世界");
        }
        _ => panic!("Expected Say command"),
    }
}

#[test]
fn test_parse_set_textbox() {
    let commands = ScriptParser::parse("setTextbox=hide");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::SetTextbox { visible } => {
            assert!(!visible);
        }
        _ => panic!("Expected SetTextbox command"),
    }
}

#[test]
fn test_parse_play_effect() {
    let commands = ScriptParser::parse("playEffect=explosion.mp3");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::PlayEffect { path } => {
            assert_eq!(path, "explosion.mp3");
        }
        _ => panic!("Expected PlayEffect command"),
    }
}

#[test]
fn test_parse_video() {
    let commands = ScriptParser::parse("video=opening.mp4");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::Video { path } => {
            assert_eq!(path, "opening.mp4");
        }
        _ => panic!("Expected Video command"),
    }
}

#[test]
fn test_parse_intro() {
    let commands = ScriptParser::parse("intro=第一章 相遇");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::Intro { text } => {
            assert_eq!(text, "第一章 相遇");
        }
        _ => panic!("Expected Intro command"),
    }
}

#[test]
fn test_parse_change_scene() {
    let commands = ScriptParser::parse("changeScene=chapter2");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::ChangeScene { scene_id } => {
            assert_eq!(scene_id, "chapter2");
        }
        _ => panic!("Expected ChangeScene command"),
    }
}

#[test]
fn test_parse_call_scene() {
    let commands = ScriptParser::parse("callScene=sub_dialogue");
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        ScriptCommand::CallScene { scene_id } => {
            assert_eq!(scene_id, "sub_dialogue");
        }
        _ => panic!("Expected CallScene command"),
    }
}
