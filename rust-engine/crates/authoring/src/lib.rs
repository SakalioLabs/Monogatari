//! Headless project authoring services shared by desktop and agent transports.

#![forbid(unsafe_code)]

pub mod agent_transaction;
pub mod filesystem;
pub mod json_catalog;
pub mod paths;
pub mod project;
pub mod quality_suite_validation;
pub mod runtime_validation;
pub mod story_content_validation;
pub mod story_events;
pub mod workflow_validation;
