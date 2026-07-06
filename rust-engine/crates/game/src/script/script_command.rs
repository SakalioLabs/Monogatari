//! Script command types for the DSL parser.

use serde::{Deserialize, Serialize};

/// A parsed argument: `-key=value` or `-flag`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandArg {
    pub key: String,
    pub value: Option<String>,
}

/// All possible script commands, inspired by WebGAL's command set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptCommand {
    /// Dialogue: `speaker:text`
    Say {
        speaker: Option<String>,
        text: String,
        vocal: Option<String>,
        emotion: Option<String>,
        concat: bool,
        notend: bool,
    },
    /// Change background: `-changeBg=path`
    ChangeBg {
        path: String,
        transition: Option<String>,
    },
    /// Change character figure: `-changeFigure=path`
    ChangeFigure {
        path: String,
        position: Option<String>,
        id: Option<String>,
        motion: Option<String>,
        expression: Option<String>,
        live2d: bool,
    },
    /// Play background music: `-bgm=path`
    Bgm { path: Option<String> },
    /// Play video: `-video=path`
    Video { path: String },
    /// Show intro text: `-intro=text`
    Intro { text: String },
    /// Show choices: `-choose=text1:target1,text2:target2`
    Choose {
        choices: Vec<ChoiceOption>,
    },
    /// Change scene: `-changeScene=sceneId`
    ChangeScene { scene_id: String },
    /// Call scene (push current): `-callScene=sceneId`
    CallScene { scene_id: String },
    /// Return from called scene
    Return,
    /// Set variable: `-setVar=name=value`
    SetVar { name: String, value: String },
    /// Conditional: `-if=condition`
    If { condition: String, then_cmd: Box<ScriptCommand>, else_cmd: Option<Box<ScriptCommand>> },
    /// Jump to label: `-jumpLabel=label`
    JumpLabel { label: String },
    /// Define label: `-label=name`
    Label { name: String },
    /// Set textbox visibility: `-setTextbox=show|hide`
    SetTextbox { visible: bool },
    /// Play effect: `-playEffect=path`
    PlayEffect { path: String },
    /// Wait: `-wait=ms`
    Wait { duration_ms: u64 },
    /// Set animation on figure: `-setAnimation=name`
    SetAnimation { target: String, animation: String },
    /// Set transform on figure: `-setTransform=x,y,scale,rotation,alpha`
    SetTransform { target: String, x: f32, y: f32, scale: f32, rotation: f32, alpha: f32 },
    /// Set filter effect: `-setFilter=blur,brightness,contrast`
    SetFilter { target: String, filter_type: String, value: f32 },
    /// Unlock CG in gallery: `-unlockCg=id`
    UnlockCg { id: String },
    /// Unlock BGM in gallery: `-unlockBgm=id`
    UnlockBgm { id: String },
    /// Show mini avatar: `-miniAvatar=path`
    MiniAvatar { path: String },
    /// Set text speed: `-setTextSpeed=ms`
    SetTextSpeed { speed_ms: u32 },
    /// Set auto play speed: `-setAutoPlaySpeed=ms`
    SetAutoPlaySpeed { speed_ms: u32 },
    /// Toggle film mode: `-filmMode=on|off`
    FilmMode { enabled: bool },
    /// Play voice: `-playVoice=path`
    PlayVoice { path: String },
    /// Stop voice: `-stopVoice`
    StopVoice,
    /// End dialogue
    End,
    /// Comment (ignored)
    Comment { text: String },
    /// Unknown command (pass through)
    Unknown { command: String, args: Vec<CommandArg> },
}

/// A choice option with text and target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceOption {
    pub text: String,
    pub target: String,
}

impl CommandArg {
    /// Parse a `-key=value` or `-flag` argument string.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim_start_matches('-');
        if let Some(eq_pos) = s.find('=') {
            let key = s[..eq_pos].to_string();
            let value = s[eq_pos + 1..].to_string();
            Some(CommandArg { key, value: Some(value) })
        } else if !s.is_empty() {
            Some(CommandArg { key: s.to_string(), value: None })
        } else {
            None
        }
    }
}
