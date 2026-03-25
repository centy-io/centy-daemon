mod crud;
mod crud_fns;
mod crud_helpers;
mod crud_read;
mod crud_types;
mod link_types;
mod storage;
mod types;

pub use crud::{
    create_link, delete_link, get_available_link_types, list_links, CreateLinkOptions,
    DeleteLinkOptions, LinkError,
};
pub use link_types::{get_inverse_link_type, is_valid_link_type, BUILTIN_LINK_TYPES};
pub use storage::{read_links, write_links, LinksFile};
pub use types::{CustomLinkTypeDefinition, Link, TargetType};

#[cfg(test)]
#[path = "link_creation_and_serialization.rs"]
mod link_creation_and_serialization;
#[cfg(test)]
#[path = "target_type_conversion.rs"]
mod target_type_conversion;
#[cfg(test)]
#[path = "target_type_serialization.rs"]
mod target_type_serialization;
