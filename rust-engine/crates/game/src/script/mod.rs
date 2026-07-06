//! # Script DSL Parser
//!
//! This module provides a WebGAL-inspired line-oriented script DSL parser for
//! defining visual novel dialogue, scenes, and game logic.
//!
//! ## Format
//!
//! The script format is simple and human-readable:
//!
//! ```text
//! ; Comments start with semicolons
//! 
//! ; Narrator dialogue (no speaker)
//! 这是一段旁白。
//!
//! ; Character dialogue
//! Sakura:你好！
//!
//! ; Character dialogue with emotion
//! Sakura:happy 今天天气真好！
//!
//! ; Character dialogue with voice
//! Sakura:你好 -v=v1.ogg
//!
//! ; Commands
//! changeBg=park.png
//! changeFigure=sakura.png -position=center -live2d
//! bgm=bgm1.mp3
//! choose=去公园:park,回家:home
//! end
//! ```
//!
//! ## Supported Commands
//!
//! | Command | Description | Example |
//! |---------|-------------|---------|
//! | `speaker:text` | Character dialogue | `Sakura:你好！` |
//! | `speaker:emotion:text` | Dialogue with emotion | `Sakura:happy 你好！` |
//! | `changeBg=path` | Change background | `changeBg=park.png` |
//! | `changeFigure=path` | Change character figure | `changeFigure=sakura.png` |
//! | `bgm=path` | Play background music | `bgm=bgm1.mp3` |
//! | `video=path` | Play video | `video=opening.mp4` |
//! | `intro=text` | Show intro text | `intro=第一章 相遇` |
//! | `choose=text:target,...` | Show choices | `choose=是:yes,否:no` |
//! | `changeScene=id` | Change scene | `changeScene=chapter2` |
//! | `callScene=id` | Call sub-scene | `callScene=sub_dialogue` |
//! | `setVar=name=value` | Set variable | `setVar=x=10` |
//! | `label=name` | Define label | `label=start` |
//! | `jumpLabel=name` | Jump to label | `jumpLabel=end` |
//! | `setTextbox=show\|hide` | Toggle textbox | `setTextbox=hide` |
//! | `playEffect=path` | Play sound effect | `playEffect=explosion.mp3` |
//! | `wait=ms` | Wait milliseconds | `wait=1000` |
//! | `end` | End dialogue | `end` |
//!
//! ## Arguments
//!
//! Commands can have additional arguments in `-key=value` format:
//!
//! ```text
//! Sakura:你好 -v=v1.ogg -concat -notend
//! changeBg=park.png -fadeIn
//! changeFigure=sakura.png -position=center -motion=idle -live2d
//! ```

pub mod parser;
pub mod script_command;

pub use parser::ScriptParser;
pub use script_command::{ChoiceOption, CommandArg, ScriptCommand};
