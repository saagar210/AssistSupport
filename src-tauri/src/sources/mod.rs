//! Source definition module for AssistSupport
//! Handles YAML source file parsing for batch content ingestion

pub mod parser;

pub use parser::{ParseError, SourceDefinition, SourceFile, SourceType};
