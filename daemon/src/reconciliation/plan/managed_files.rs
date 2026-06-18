use super::hashing::{actual_file_hash, template_hash};
use super::types::{FileInfo, ReconciliationPlan};
use crate::manifest::ManagedFileType;
use crate::reconciliation::managed_files::ManagedFileTemplate;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;

pub async fn process_managed_files(
    plan: &mut ReconciliationPlan,
    managed_templates: &HashMap<String, ManagedFileTemplate>,
    files_on_disk: &HashSet<String>,
    centy_path: &Path,
) {
    for (path, template) in managed_templates {
        let full_path = centy_path.join(path.trim_end_matches('/'));
        let exists_on_disk = files_on_disk.contains(path);
        let file_info = FileInfo {
            path: path.clone(),
            file_type: template.file_type.clone(),
            hash: template
                .content
                .as_ref()
                .map(|c| template_hash(c))
                .unwrap_or_default(),
            content_preview: template
                .content
                .as_ref()
                .map(|c| c.chars().take(100).collect::<String>()),
        };
        if exists_on_disk {
            match &template.file_type {
                ManagedFileType::Directory => {
                    plan.up_to_date.push(file_info);
                }
                ManagedFileType::File => {
                    if let Some(expected_content) = &template.content {
                        let expected_hash = template_hash(expected_content);
                        let actual_hash = actual_file_hash(&full_path).await;
                        if actual_hash == expected_hash {
                            plan.up_to_date.push(file_info);
                        } else {
                            plan.to_reset.push(FileInfo {
                                hash: actual_hash,
                                ..file_info
                            });
                        }
                    } else {
                        plan.up_to_date.push(file_info);
                    }
                }
            }
        } else {
            plan.to_create.push(file_info);
        }
    }
}
