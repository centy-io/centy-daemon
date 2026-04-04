mod crud;
mod crud_fns;
mod crud_helpers;
mod crud_read;
mod crud_types;
mod link_types;
mod storage;
mod types;

pub use crud::{
    create_link, delete_link, delete_link_by_id, get_available_link_types, list_links, update_link,
    CreateLinkOptions, DeleteLinkOptions, LinkError, UpdateLinkOptions,
};
pub use link_types::{is_valid_link_type, BUILTIN_LINK_TYPES};
pub use types::{CustomLinkTypeDefinition, LinkDirection, LinkRecord, LinkView, TargetType};

#[cfg(test)]
#[path = "link_create_tests.rs"]
mod link_create_tests;
#[cfg(test)]
#[path = "link_creation_and_serialization_tests.rs"]
mod link_creation_and_serialization_tests;
#[cfg(test)]
#[path = "target_type_conversion_tests.rs"]
mod target_type_conversion_tests;
#[cfg(test)]
#[path = "target_type_serialization_tests.rs"]
mod target_type_serialization_tests;
