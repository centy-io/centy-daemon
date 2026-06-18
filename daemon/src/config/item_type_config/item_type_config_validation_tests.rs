use super::*;

#[test]
fn test_yaml_soft_delete_feature() {
    let yaml = "name: Bug\nidentifier: uuid\nfeatures:\n  displayNumber: false\n  status: true\n  priority: true\n  softDelete: true\n  assets: false\n  orgSync: false\n  move: false\n  duplicate: false\n";
    let config: ItemTypeConfig = serde_yaml::from_str(yaml).expect("Should deserialize");
    assert!(config.features.soft_delete);
}
