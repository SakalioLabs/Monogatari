//! Headless project authoring services shared by desktop and agent transports.

#![forbid(unsafe_code)]

pub mod agent_transaction;
pub mod conversation_quality;
pub mod delivery_validation;
pub mod filesystem;
pub mod json_catalog;
pub mod paths;
pub mod project;
pub mod project_package;
pub mod prompt_guard;
pub mod quality_suite_execution;
pub mod quality_suite_validation;
pub mod runtime_validation;
pub mod story_content_validation;
pub mod story_events;
pub mod workflow_execution_policy;
pub mod workflow_preview;
pub mod workflow_validation;
