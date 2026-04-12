//! Unified Item domain following Domain-Driven Design principles.
//!
//! This module consolidates Issues and Docs into a unified "item" concept
//! with shared traits for common operations while preserving entity-specific behavior.

// that are prepared for future use but not yet fully integrated

pub mod core;
pub mod entities;
pub mod generic;
pub mod lifecycle;
pub mod operations;

// Re-export core types
pub use core::error::ItemError;
pub use core::metadata::ItemMetadata;
