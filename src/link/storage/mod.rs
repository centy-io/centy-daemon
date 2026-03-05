mod io;
mod links_file;
pub use io::{read_links, write_links};
pub use links_file::LinksFile;
#[cfg(test)]
pub use crate::link::Link;
#[cfg(test)]
pub use links_file::LINKS_FILENAME;
#[cfg(test)]
pub use tokio::fs;
#[cfg(test)]
#[path = "../storage_tests_1.rs"]
mod storage_tests_1;
#[cfg(test)]
#[path = "../storage_tests_2.rs"]
mod storage_tests_2;
#[cfg(test)]
#[path = "../storage_tests_3.rs"]
mod storage_tests_3;
#[cfg(test)]
#[path = "../storage_tests_4.rs"]
mod storage_tests_4;
