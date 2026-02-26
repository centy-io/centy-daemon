use crate::link::Link;
use serde::{Deserialize, Serialize};
pub const LINKS_FILENAME: &str = "links.json";
/// Container for links stored in links.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LinksFile {
    #[serde(default)]
    pub links: Vec<Link>,
}
impl LinksFile {
    #[must_use]
    pub fn new() -> Self { Self { links: Vec::new() } }
    pub fn add_link(&mut self, link: Link) { self.links.push(link); }
    /// Remove a link matching the target and link type.
    /// Returns `true` if a link was removed, `false` otherwise.
    pub fn remove_link(&mut self, target_id: &str, link_type: Option<&str>) -> bool {
        let initial_len = self.links.len();
        self.links.retain(|link| {
            if link.target_id != target_id { return true; }
            if let Some(lt) = link_type { link.link_type != lt } else { false }
        });
        self.links.len() < initial_len
    }
    pub fn has_link(&self, target_id: &str, link_type: &str) -> bool {
        self.links.iter().any(|link| link.target_id == target_id && link.link_type == link_type)
    }
}
