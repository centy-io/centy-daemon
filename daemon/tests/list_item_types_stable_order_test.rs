#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

mod common;

use centy_daemon::config::item_type_config::BUILTIN_ITEM_TYPE_FOLDERS;
use centy_daemon::server::handlers::item_type_create::create_item_type;
use centy_daemon::server::handlers::item_type_list::list_item_types;
use centy_daemon::server::proto::{
    CreateItemTypeRequest, ItemTypeFeatures as ProtoFeatures, ListItemTypesRequest,
};
use common::{create_test_dir, init_centy_project};

fn custom_type_request(pp: &str, name: &str, plural: &str) -> CreateItemTypeRequest {
    CreateItemTypeRequest {
        project_path: pp.to_string(),
        name: name.to_string(),
        plural: plural.to_string(),
        identifier: "uuid".to_string(),
        features: Some(ProtoFeatures {
            display_number: true,
            status: true,
            priority: true,
            soft_delete: false,
            assets: false,
            org_sync: false,
            r#move: true,
            duplicate: true,
        }),
        statuses: vec!["open".to_string(), "closed".to_string()],
        default_status: "open".to_string(),
        priority_levels: 2,
        custom_fields: vec![],
    }
}

/// Repeated calls return the same order: listed built-ins in const order,
/// then custom types sorted by folder name.
#[tokio::test]
async fn test_list_item_types_order_is_stable() {
    let temp = create_test_dir();
    let path = temp.path();
    init_centy_project(path).await;
    let pp = path.to_str().unwrap();

    // Created out of alphabetical order on purpose.
    for (name, plural) in [("Zeta", "zeta"), ("Alpha", "alpha")] {
        let resp = create_item_type(custom_type_request(pp, name, plural))
            .await
            .unwrap()
            .into_inner();
        assert!(resp.success, "create {plural} failed: {}", resp.error);
    }

    let mut first: Option<Vec<String>> = None;
    for _ in 0..10u32 {
        let resp = list_item_types(ListItemTypesRequest {
            project_path: pp.to_string(),
        })
        .await
        .unwrap()
        .into_inner();
        assert!(resp.success, "list failed: {}", resp.error);
        let plurals: Vec<String> = resp.item_types.iter().map(|t| t.plural.clone()).collect();

        match &first {
            None => {
                let mut expected: Vec<String> = BUILTIN_ITEM_TYPE_FOLDERS
                    .iter()
                    .filter(|f| plurals.iter().any(|p| p == *f))
                    .map(ToString::to_string)
                    .collect();
                expected.push("alpha".to_string());
                expected.push("zeta".to_string());
                assert_eq!(plurals, expected, "unexpected order on first call");
                first = Some(plurals);
            }
            Some(order) => assert_eq!(&plurals, order, "order changed between calls"),
        }
    }
}
