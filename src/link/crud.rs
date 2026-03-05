pub use super::crud_fns::{create_link, delete_link, get_available_link_types, list_links};
#[cfg(test)]
pub use super::crud_types::LinkTypeInfo;
pub use super::crud_types::{CreateLinkOptions, DeleteLinkOptions, LinkError};
#[cfg(test)]
pub use super::{CustomLinkTypeDefinition, TargetType};
#[cfg(test)]
#[path = "crud_tests_1.rs"]
mod tests_1;
#[cfg(test)]
#[path = "crud_tests_2.rs"]
mod tests_2;
