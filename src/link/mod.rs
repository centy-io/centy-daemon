mod crud;
mod link_types;
mod storage;
mod types;

pub use crud::{
    create_link, delete_link, get_available_link_types, list_links, CreateLinkOptions,
    DeleteLinkOptions, LinkError,
};
#[allow(unused_imports)]
pub use crud::{CreateLinkResult, DeleteLinkResult, LinkTypeInfo};
pub use link_types::{get_inverse_link_type, is_valid_link_type, BUILTIN_LINK_TYPES};
pub use storage::{read_links, write_links, LinksFile};
pub use types::{CustomLinkTypeDefinition, Link, TargetType};

#[cfg(test)]
#[path = "link_tests_1.rs"]
mod tests_1;
#[cfg(test)]
#[path = "link_tests_2.rs"]
mod tests_2;
#[cfg(test)]
#[path = "link_tests_3.rs"]
mod tests_3;
