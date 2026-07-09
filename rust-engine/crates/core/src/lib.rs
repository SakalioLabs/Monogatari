//! # LLM Galgame Engine - Core
//!
//! Foundation layer for the LLM-powered visual novel engine.
//!
//! ## Architecture
//!
//! The core module provides the fundamental building blocks:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      Core Module                            │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
//! │  │ ServiceLocator│  │  EventBus    │  │  GameClock   │      │
//! │  │  (依赖管理)   │  │  (事件总线)   │  │  (游戏时钟)  │      │
//! │  └──────────────┘  └──────────────┘  └──────────────┘      │
//! │  ┌──────────────┐  ┌──────────────┐                        │
//! │  │ GameService   │  │  Events      │                        │
//! │  │  (服务接口)   │  │  (事件类型)   │                        │
//! │  └──────────────┘  └──────────────┘                        │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key Components
//!
//! ### ServiceLocator
//! Thread-safe service registry using `Arc<RwLock<T>>` for shared ownership.
//! Services are registered by type and can be retrieved by type.
//!
//! ### EventBus
//! Type-erased publish/subscribe event system. Subscribers register callbacks
//! for specific event types and receive references to event data.
//!
//! ### GameClock
//! High-precision game clock tracking delta time, total time, FPS, and
//! supporting fixed update intervals for physics/logic updates.
//!
//! ### GameService Trait
//! Common interface for all engine services with lifecycle methods:
//! - `initialize()` - Called once when registered
//! - `update(delta_time)` - Called every frame
//! - `shutdown()` - Called on engine exit
//!
//! ## Usage
//!
//! ```rust
//! use llm_core::{ServiceLocator, EventBus, GameClock};
//!
//! // Create service locator
//! let locator = ServiceLocator::new();
//!
//! // Create event bus
//! let event_bus = EventBus::new();
//!
//! // Create game clock (60 FPS)
//! let mut clock = GameClock::new(60.0);
//!
//! // Game loop
//! clock.tick();
//! let dt = clock.delta_time();
//! ```

pub mod error;
pub mod event_bus;
pub mod events;
pub mod game_clock;
pub mod service_locator;
pub mod state_key;
pub mod traits;

pub use error::{EngineError, Result};
pub use event_bus::EventBus;
pub use events::*;
pub use game_clock::GameClock;
pub use service_locator::ServiceLocator;
pub use state_key::{
    normalize_script_state_key, normalize_script_state_map, SCRIPT_STATE_KEY_MAX_CHARS,
};
pub use traits::GameService;
