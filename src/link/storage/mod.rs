mod io;
mod links_file;
#[cfg(test)]
pub use crate::link::Link;
pub use io::{read_links, write_links};
pub use links_file::LinksFile;
#[cfg(test)]
pub use links_file::LINKS_FILENAME;
#[cfg(test)]
pub use tokio::fs;
#[cfg(test)]
#[path = "../links_file_basic_operations.rs"]
mod links_file_basic_operations;
#[cfg(test)]
#[path = "../links_file_edge_cases.rs"]
mod links_file_edge_cases;
#[cfg(test)]
#[path = "../links_file_multiple_links.rs"]
mod links_file_multiple_links;
#[cfg(test)]
#[path = "../links_file_persistence.rs"]
mod links_file_persistence;
