pub use super::crud_fns::{create_link, delete_link};
pub use super::crud_read::{get_available_link_types, list_links};
#[cfg(test)]
pub use super::crud_types::LinkTypeInfo;
pub use super::crud_types::{CreateLinkOptions, DeleteLinkOptions, LinkError};
#[cfg(test)]
pub use super::types::{CustomLinkTypeDefinition, TargetType};

#[cfg(test)]
#[path = "link_options_debug.rs"]
mod link_options_debug;
#[cfg(test)]
#[path = "link_type_availability.rs"]
mod link_type_availability;
