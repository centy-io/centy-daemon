use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct CentyManifest {
    pub schema_version: u32,
    pub centy_version: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Type of managed file (file or directory)
/// Used for reconciliation and file type distinction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ManagedFileType {
    File,
    Directory,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centy_manifest_default() {
        let manifest = CentyManifest::default();
        assert_eq!(manifest.schema_version, 0);
        assert!(manifest.centy_version.is_empty());
        assert!(manifest.created_at.is_empty());
        assert!(manifest.updated_at.is_empty());
    }

    #[test]
    fn test_centy_manifest_serialization() {
        let manifest = CentyManifest {
            schema_version: 1,
            centy_version: "0.1.0".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-06-15T12:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&manifest).expect("Should serialize");
        let deserialized: CentyManifest = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(manifest, deserialized);
    }

    #[test]
    fn test_centy_manifest_camel_case_json() {
        let manifest = CentyManifest {
            schema_version: 1,
            centy_version: "0.1.0".to_string(),
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&manifest).expect("Should serialize");
        assert!(json.contains("schemaVersion"));
        assert!(json.contains("centyVersion"));
        assert!(json.contains("createdAt"));
        assert!(json.contains("updatedAt"));
        assert!(!json.contains("schema_version"));
    }

    #[test]
    fn test_centy_manifest_clone() {
        let manifest = CentyManifest {
            schema_version: 2,
            centy_version: "1.0.0".to_string(),
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let cloned = manifest.clone();
        assert_eq!(manifest, cloned);
    }

    #[test]
    fn test_managed_file_type_file() {
        let ft = ManagedFileType::File;
        let json = serde_json::to_string(&ft).expect("Should serialize");
        assert_eq!(json, "\"file\"");
    }

    #[test]
    fn test_managed_file_type_directory() {
        let ft = ManagedFileType::Directory;
        let json = serde_json::to_string(&ft).expect("Should serialize");
        assert_eq!(json, "\"directory\"");
    }

    #[test]
    fn test_managed_file_type_deserialization() {
        let ft: ManagedFileType = serde_json::from_str("\"file\"").expect("Should deserialize");
        assert_eq!(ft, ManagedFileType::File);

        let ft: ManagedFileType =
            serde_json::from_str("\"directory\"").expect("Should deserialize");
        assert_eq!(ft, ManagedFileType::Directory);
    }

    #[test]
    fn test_managed_file_type_clone_and_eq() {
        let ft = ManagedFileType::File;
        let cloned = ft.clone();
        assert_eq!(ft, cloned);
    }

    #[test]
    fn test_managed_file_type_debug() {
        let debug = format!("{:?}", ManagedFileType::File);
        assert!(debug.contains("File"));
        let debug = format!("{:?}", ManagedFileType::Directory);
        assert!(debug.contains("Directory"));
    }
}
