mod io;
mod links_file;
#[allow(unused_imports)]
pub use crate::link::Link;
pub use io::{read_links, write_links};
#[allow(unused_imports)]
pub use links_file::{LinksFile, LINKS_FILENAME};
#[allow(unused_imports)]
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
