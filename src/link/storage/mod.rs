mod io;
pub use io::{create_link_file, delete_link_file, list_all_link_records};

#[cfg(test)]
#[path = "../links_file_basic_operations.rs"]
mod links_file_basic_operations;
#[cfg(test)]
#[path = "../links_file_persistence.rs"]
mod links_file_persistence;
#[cfg(test)]
#[path = "../storage_io_tests.rs"]
mod storage_io_tests;
