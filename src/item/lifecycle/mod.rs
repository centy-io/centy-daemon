//! Item lifecycle management (soft delete, restore).

pub mod soft_delete;

pub use soft_delete::{Restorable, SoftDeletable};
