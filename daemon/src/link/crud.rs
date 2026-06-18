pub use super::crud_fns::{
    cascade_delete_entity_links, create_link, delete_link, delete_link_by_id, update_link,
};
pub use super::crud_read::{get_available_link_types, list_all_links, list_links};
#[cfg(test)]
pub use super::crud_types::LinkTypeInfo;
pub use super::crud_types::{CreateLinkOptions, DeleteLinkOptions, LinkError, UpdateLinkOptions};
#[cfg(test)]
pub use super::types::{CustomLinkTypeDefinition, TargetType};

#[cfg(test)]
#[path = "link_options_debug_tests.rs"]
mod link_options_debug_tests;
#[cfg(test)]
#[path = "link_type_availability_tests.rs"]
mod link_type_availability_tests;
