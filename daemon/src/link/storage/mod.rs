mod io;
pub mod serialization;
pub mod validation;
pub use io::{
    create_link_file, delete_link_file, delete_links_for_entity, list_all_link_records,
    update_link_file,
};

#[cfg(test)]
#[path = "../links_file_basic_operations_tests.rs"]
mod links_file_basic_operations_tests;
#[cfg(test)]
#[path = "../links_file_persistence_tests.rs"]
mod links_file_persistence_tests;
#[cfg(test)]
#[path = "../storage_io_tests.rs"]
mod storage_io_tests;
#[cfg(test)]
#[path = "../storage_serialization_tests.rs"]
mod storage_serialization_tests;
#[cfg(test)]
#[path = "../storage_validation_tests.rs"]
mod storage_validation_tests;
