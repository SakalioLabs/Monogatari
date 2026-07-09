//! Script parser: converts text lines into ScriptCommands.
//!
//! Format: `[speaker]:[content] [-arg1=value1 -arg2=value2] ;`
//!
//! Rules:
//! - Semicolons `;` start inline comments
//! - Backslash escapes: `\;` `\{` `\}` `\:`
//! - Lines starting with `-` followed by a command name are commands
//! - Lines with `:` where the left side is known are speaker dialogue
//! - Lines without recognized patterns are narrator dialogue

use super::script_command::{ChoiceOption, CommandArg, ScriptCommand};

/// Parses a script text into a list of commands.
pub struct ScriptParser;

impl ScriptParser {
    /// Parse a full script text into commands.
    pub fn parse(script: &str) -> Vec<ScriptCommand> {
        script
            .lines()
            .enumerate()
            .filter_map(|(line_num, line)| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    return None;
                }
                match Self::parse_line(trimmed) {
                    Ok(cmd) => Some(cmd),
                    Err(e) => {
                        tracing::warn!("Parse error on line {}: {}", line_num + 1, e);
                        None
                    }
                }
            })
            .collect()
    }

    /// Parse a single line into a ScriptCommand.
    pub fn parse_line(line: &str) -> Result<ScriptCommand, String> {
        // Remove inline comments (everything after unescaped `;`)
        let line = Self::remove_comments(line);
        let line = line.trim();
        if line.is_empty() {
            return Err("Empty line".to_string());
        }

        // Check if it's a command line (starts with known command pattern)
        if let Some(cmd) = Self::try_parse_command(line) {
            return Ok(cmd);
        }

        // Try to parse as dialogue: [speaker]:[content]
        if let Some(cmd) = Self::try_parse_dialogue(line) {
            return Ok(cmd);
        }

        // Default: narrator dialogue
        Ok(ScriptCommand::Say {
            speaker: None,
            text: Self::unescape(line),
            vocal: None,
            emotion: None,
            concat: false,
            notend: false,
        })
    }

    /// Remove inline comments (unescaped `;`).
    fn remove_comments(line: &str) -> String {
        let mut result = String::new();
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(&next) = chars.peek() {
                    result.push(next);
                    chars.next();
                } else {
                    result.push(c);
                }
            } else if c == ';' {
                break;
            } else {
                result.push(c);
            }
        }
        result
    }

    /// Unescape special characters.
    fn unescape(s: &str) -> String {
        s.replace("\\;", ";")
            .replace("\\{", "{")
            .replace("\\}", "}")
            .replace("\\:", ":")
            .replace("\\n", "\n")
    }

    /// Try to parse as a command line (e.g., `changeBg=path` or `-changeBg=path`).
    fn try_parse_command(line: &str) -> Option<ScriptCommand> {
        let line = line.trim();

        // changeBg
        if let Some(rest) = Self::strip_prefix(line, "changeBg") {
            let path = Self::extract_value(rest, "changeBg");
            let args = Self::parse_args(rest);
            let transition = args
                .iter()
                .find(|a| a.key == "transition" || a.key == "t")
                .and_then(|a| a.value.clone());
            return Some(ScriptCommand::ChangeBg { path, transition });
        }

        // changeFigure
        if let Some(rest) = Self::strip_prefix(line, "changeFigure") {
            let path = Self::extract_value(rest, "changeFigure");
            let args = Self::parse_args(rest);
            let position = args
                .iter()
                .find(|a| a.key == "position" || a.key == "pos")
                .and_then(|a| a.value.clone());
            let id = args
                .iter()
                .find(|a| a.key == "id")
                .and_then(|a| a.value.clone());
            let motion = args
                .iter()
                .find(|a| a.key == "motion")
                .and_then(|a| a.value.clone());
            let expression = args
                .iter()
                .find(|a| a.key == "expression" || a.key == "exp")
                .and_then(|a| a.value.clone());
            let live2d = args.iter().any(|a| a.key == "live2d");
            return Some(ScriptCommand::ChangeFigure {
                path,
                position,
                id,
                motion,
                expression,
                live2d,
            });
        }

        // bgm
        if let Some(rest) = Self::strip_prefix(line, "bgm") {
            let path = Self::extract_value(rest, "bgm");
            let path = if path.is_empty() { None } else { Some(path) };
            return Some(ScriptCommand::Bgm { path });
        }

        // video
        if let Some(rest) = Self::strip_prefix(line, "video") {
            let path = Self::extract_value(rest, "video");
            return Some(ScriptCommand::Video { path });
        }

        // intro
        if let Some(rest) = Self::strip_prefix(line, "intro") {
            let text = Self::extract_value(rest, "intro");
            return Some(ScriptCommand::Intro { text });
        }

        // choose
        if let Some(rest) = Self::strip_prefix(line, "choose") {
            let choose_arg = Self::extract_value(rest, "choose");
            let choices = Self::parse_choices(&choose_arg);
            return Some(ScriptCommand::Choose { choices });
        }

        // changeScene
        if let Some(rest) = Self::strip_prefix(line, "changeScene") {
            let scene_id = Self::extract_value(rest, "changeScene");
            return Some(ScriptCommand::ChangeScene { scene_id });
        }

        // callScene
        if let Some(rest) = Self::strip_prefix(line, "callScene") {
            let scene_id = Self::extract_value(rest, "callScene");
            return Some(ScriptCommand::CallScene { scene_id });
        }

        // setVar
        if let Some(rest) = Self::strip_prefix(line, "setVar") {
            let val = Self::extract_value(rest, "setVar");
            if let Some(eq_pos) = val.find('=') {
                return Some(ScriptCommand::SetVar {
                    name: val[..eq_pos].to_string(),
                    value: val[eq_pos + 1..].to_string(),
                });
            }
            return Some(ScriptCommand::SetVar {
                name: String::new(),
                value: String::new(),
            });
        }

        // label
        if let Some(rest) = Self::strip_prefix(line, "label") {
            let name = Self::extract_value(rest, "label");
            return Some(ScriptCommand::Label { name });
        }

        // jumpLabel
        if let Some(rest) = Self::strip_prefix(line, "jumpLabel") {
            let label = Self::extract_value(rest, "jumpLabel");
            return Some(ScriptCommand::JumpLabel { label });
        }

        // setTextbox
        if let Some(rest) = Self::strip_prefix(line, "setTextbox") {
            let value = Self::extract_value(rest, "setTextbox");
            let visible = value != "hide";
            return Some(ScriptCommand::SetTextbox { visible });
        }

        // playEffect
        if let Some(rest) = Self::strip_prefix(line, "playEffect") {
            let path = Self::extract_value(rest, "playEffect");
            return Some(ScriptCommand::PlayEffect { path });
        }

        // wait
        if let Some(rest) = Self::strip_prefix(line, "wait") {
            let value = Self::extract_value(rest, "wait");
            let duration_ms = value.parse().unwrap_or(1000);
            return Some(ScriptCommand::Wait { duration_ms });
        }

        // setAnimation
        if let Some(rest) = Self::strip_prefix(line, "setAnimation") {
            let value = Self::extract_value(rest, "setAnimation");
            let parts: Vec<&str> = value.splitn(2, ':').collect();
            if parts.len() == 2 {
                return Some(ScriptCommand::SetAnimation {
                    target: parts[0].to_string(),
                    animation: parts[1].to_string(),
                });
            }
        }

        // miniAvatar
        if let Some(rest) = Self::strip_prefix(line, "miniAvatar") {
            let path = Self::extract_value(rest, "miniAvatar");
            return Some(ScriptCommand::MiniAvatar { path });
        }

        // setTextSpeed
        if let Some(rest) = Self::strip_prefix(line, "setTextSpeed") {
            let value = Self::extract_value(rest, "setTextSpeed");
            let speed_ms = value.parse().unwrap_or(30);
            return Some(ScriptCommand::SetTextSpeed { speed_ms });
        }

        // setAutoPlaySpeed
        if let Some(rest) = Self::strip_prefix(line, "setAutoPlaySpeed") {
            let value = Self::extract_value(rest, "setAutoPlaySpeed");
            let speed_ms = value.parse().unwrap_or(3000);
            return Some(ScriptCommand::SetAutoPlaySpeed { speed_ms });
        }

        // filmMode
        if let Some(rest) = Self::strip_prefix(line, "filmMode") {
            let value = Self::extract_value(rest, "filmMode");
            let enabled = value != "off";
            return Some(ScriptCommand::FilmMode { enabled });
        }

        // playVoice
        if let Some(rest) = Self::strip_prefix(line, "playVoice") {
            let path = Self::extract_value(rest, "playVoice");
            return Some(ScriptCommand::PlayVoice { path });
        }

        // stopVoice
        if line == "stopVoice" {
            return Some(ScriptCommand::StopVoice);
        }

        // unlockCg
        if let Some(rest) = Self::strip_prefix(line, "unlockCg") {
            let id = Self::extract_value(rest, "unlockCg");
            return Some(ScriptCommand::UnlockCg { id });
        }

        // unlockBgm
        if let Some(rest) = Self::strip_prefix(line, "unlockBgm") {
            let id = Self::extract_value(rest, "unlockBgm");
            return Some(ScriptCommand::UnlockBgm { id });
        }

        // end
        if line == "end" {
            return Some(ScriptCommand::End);
        }

        None
    }

    /// Try to parse as dialogue: `speaker:text` or `speaker:emotion:text`
    fn try_parse_dialogue(line: &str) -> Option<ScriptCommand> {
        // Find the first unescaped colon
        let mut colon_pos = None;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '\\' && i + 1 < chars.len() {
                i += 2; // skip escaped char
                continue;
            }
            if chars[i] == ':' {
                colon_pos = Some(i);
                break;
            }
            i += 1;
        }

        let colon_pos = colon_pos?;

        let speaker_part: String = chars[..colon_pos].iter().collect();
        let content_part: String = chars[colon_pos + 1..].iter().collect();

        let speaker_part = speaker_part.trim();
        let content_part = content_part.trim();

        if speaker_part.is_empty() || content_part.is_empty() {
            return None;
        }

        // Parse speaker:emotion format (e.g., "Sakura:happy")
        let (speaker, emotion_from_speaker) = if speaker_part.contains(':') {
            let parts: Vec<&str> = speaker_part.splitn(2, ':').collect();
            (Some(parts[0].to_string()), Some(parts[1].to_string()))
        } else {
            (Some(speaker_part.to_string()), None)
        };

        // Check if content starts with an emotion keyword (e.g., "happy 今天天气真好！")
        let known_emotions = [
            "happy",
            "sad",
            "angry",
            "surprised",
            "neutral",
            "shy",
            "excited",
        ];
        let (emotion, content_text) = if emotion_from_speaker.is_some() {
            // Emotion already extracted from speaker part
            (emotion_from_speaker, content_part.to_string())
        } else {
            // Check if first word of content is an emotion
            let first_word = content_part.split_whitespace().next().unwrap_or("");
            if known_emotions.contains(&first_word) && content_part.len() > first_word.len() {
                let rest = content_part[first_word.len()..].trim().to_string();
                (Some(first_word.to_string()), rest)
            } else {
                (None, content_part.to_string())
            }
        };

        // Parse content and args
        let (text, args) = Self::split_content_and_args(&content_text);

        // Extract common args
        let vocal = args
            .iter()
            .find(|a| a.key == "v" || a.key == "vocal")
            .and_then(|a| a.value.clone());
        let concat = args.iter().any(|a| a.key == "concat");
        let notend = args.iter().any(|a| a.key == "notend");

        Some(ScriptCommand::Say {
            speaker,
            text: Self::unescape(&text),
            vocal,
            emotion,
            concat,
            notend,
        })
    }

    /// Split content from trailing arguments.
    fn split_content_and_args(s: &str) -> (String, Vec<CommandArg>) {
        let mut args = Vec::new();
        let mut content = String::new();
        let mut in_args = false;

        for word in s.split_whitespace() {
            if word.starts_with('-') && word.len() > 1 && word.chars().nth(1) != Some(' ') {
                if !in_args {
                    in_args = true;
                }
                if let Some(arg) = CommandArg::parse(word) {
                    args.push(arg);
                }
            } else if in_args {
                // Continue collecting arg value
                if let Some(last) = args.last_mut() {
                    if last.value.is_none() {
                        last.value = Some(word.to_string());
                    }
                }
            } else {
                if !content.is_empty() {
                    content.push(' ');
                }
                content.push_str(word);
            }
        }

        (content, args)
    }

    /// Parse all arguments from a string.
    fn parse_args(s: &str) -> Vec<CommandArg> {
        s.split_whitespace().filter_map(CommandArg::parse).collect()
    }

    /// Parse choice options from `text1:target1,text2:target2`.
    fn parse_choices(s: &str) -> Vec<ChoiceOption> {
        s.split(',')
            .filter_map(|c| {
                let parts: Vec<&str> = c.splitn(2, ':').collect();
                if parts.len() == 2 {
                    Some(ChoiceOption {
                        text: parts[0].trim().to_string(),
                        target: parts[1].trim().to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Strip a prefix and handle the separator (`=` or `:`).
    fn strip_prefix<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
        line.strip_prefix(prefix)
    }

    /// Extract value from `prefix=value` format.
    fn extract_value(rest: &str, _prefix: &str) -> String {
        let rest = rest.trim();
        if let Some(value) = rest.strip_prefix('=') {
            // Format: `=value` (from `prefix=value`)
            let value = value.trim();
            // Extract everything before any `-` args
            if let Some(arg_pos) = value.find(" -") {
                value[..arg_pos].trim().to_string()
            } else {
                value.to_string()
            }
        } else if let Some(value) = rest.strip_prefix(':') {
            // Format: `:value` (from `prefix:value`)
            let value = value.trim();
            if let Some(arg_pos) = value.find(" -") {
                value[..arg_pos].trim().to_string()
            } else {
                value.to_string()
            }
        } else {
            // Try to find value in args
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_say() {
        let cmd = ScriptParser::parse_line("Sakura:你好！").unwrap();
        match cmd {
            ScriptCommand::Say { speaker, text, .. } => {
                assert_eq!(speaker, Some("Sakura".to_string()));
                assert_eq!(text, "你好！");
            }
            _ => panic!("Expected Say"),
        }
    }

    #[test]
    fn test_parse_say_with_args() {
        let cmd = ScriptParser::parse_line("Sakura:你好 -v=v1.ogg -concat").unwrap();
        match cmd {
            ScriptCommand::Say {
                speaker,
                text,
                vocal,
                concat,
                ..
            } => {
                assert_eq!(speaker, Some("Sakura".to_string()));
                assert_eq!(text, "你好");
                assert_eq!(vocal, Some("v1.ogg".to_string()));
                assert!(concat);
            }
            _ => panic!("Expected Say"),
        }
    }

    #[test]
    fn test_parse_narrator() {
        let cmd = ScriptParser::parse_line("这是一段旁白。").unwrap();
        match cmd {
            ScriptCommand::Say { speaker, text, .. } => {
                assert_eq!(speaker, None);
                assert_eq!(text, "这是一段旁白。");
            }
            _ => panic!("Expected Say"),
        }
    }

    #[test]
    fn test_parse_comment() {
        let cmd = ScriptParser::parse_line("你好 ;这是注释").unwrap();
        match cmd {
            ScriptCommand::Say { text, .. } => {
                assert_eq!(text, "你好");
            }
            _ => panic!("Expected Say"),
        }
    }

    #[test]
    fn test_parse_change_bg() {
        let cmd = ScriptParser::parse_line("changeBg=bg1.png").unwrap();
        match cmd {
            ScriptCommand::ChangeBg { path, .. } => {
                assert_eq!(path, "bg1.png");
            }
            _ => panic!("Expected ChangeBg"),
        }
    }

    #[test]
    fn test_parse_end() {
        let cmd = ScriptParser::parse_line("end").unwrap();
        assert!(matches!(cmd, ScriptCommand::End));
    }

    #[test]
    fn test_full_script() {
        let script = r#"
; 这是一个测试脚本
Sakura:你好！ -v=v1.ogg
这是一段旁白。
changeBg=park.png
Sakura:happy 今天天气真好！
choose=去公园:park,回家:home
end
"#;
        let commands = ScriptParser::parse(script);
        assert!(commands.len() >= 5);
    }
}
