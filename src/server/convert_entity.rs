use crate::config::item_type_config::ItemTypeConfig;

use super::proto::{
    GenericItem as ProtoGenericItem, GenericItemMetadata, ItemTypeConfigProto, User as ProtoUser,
};

pub fn generic_item_to_proto(item: &mdstore::Item, item_type: &str) -> ProtoGenericItem {
    ProtoGenericItem {
        id: item.id.clone(),
        item_type: item_type.to_string(),
        title: item.title.clone(),
        body: item.body.clone(),
        metadata: Some(GenericItemMetadata {
            display_number: item.frontmatter.display_number.unwrap_or(0),
            status: item.frontmatter.status.clone().unwrap_or_default(),
            priority: item.frontmatter.priority.unwrap_or(0),
            created_at: item.frontmatter.created_at.clone(),
            updated_at: item.frontmatter.updated_at.clone(),
            deleted_at: item.frontmatter.deleted_at.clone().unwrap_or_default(),
            custom_fields: item
                .frontmatter
                .custom_fields
                .iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect(),
        }),
    }
}

pub fn user_to_proto(user: &crate::user::User) -> ProtoUser {
    ProtoUser {
        id: user.id.clone(),
        name: user.name.clone(),
        email: user.email.clone().unwrap_or_default(),
        git_usernames: user.git_usernames.clone(),
        created_at: user.created_at.clone(),
        updated_at: user.updated_at.clone(),
        deleted_at: user.deleted_at.clone().unwrap_or_default(),
    }
}

pub fn user_to_generic_item_proto(user: &crate::user::User) -> ProtoGenericItem {
    let status = if user.deleted_at.is_some() {
        "deleted".to_string()
    } else {
        "active".to_string()
    };
    ProtoGenericItem {
        id: user.id.clone(),
        item_type: "user".to_string(),
        title: user.name.clone(),
        body: String::new(),
        metadata: Some(GenericItemMetadata {
            display_number: 0,
            status,
            priority: 0,
            created_at: user.created_at.clone(),
            updated_at: user.updated_at.clone(),
            deleted_at: user.deleted_at.clone().unwrap_or_default(),
            custom_fields: std::collections::HashMap::new(),
        }),
    }
}

pub fn config_to_proto(folder: &str, config: &ItemTypeConfig) -> ItemTypeConfigProto {
    ItemTypeConfigProto {
        name: config.name.clone(),
        plural: folder.to_string(),
        identifier: config.identifier.to_string(),
        features: Some(super::proto::ItemTypeFeatures {
            display_number: config.features.display_number,
            status: config.features.status,
            priority: config.features.priority,
            soft_delete: config.features.soft_delete,
            assets: config.features.assets,
            org_sync: config.features.org_sync,
            r#move: config.features.move_item,
            duplicate: config.features.duplicate,
        }),
        statuses: config.statuses.clone(),
        default_status: config.default_status.clone().unwrap_or_default(),
        priority_levels: config.priority_levels.unwrap_or(0),
        custom_fields: config
            .custom_fields
            .iter()
            .map(|f| super::proto::CustomFieldDefinition {
                name: f.name.clone(),
                field_type: f.field_type.clone(),
                required: f.required,
                default_value: f.default_value.clone().unwrap_or_default(),
                enum_values: f.enum_values.clone(),
            })
            .collect(),
        icon: config.icon.clone().unwrap_or_default(),
        template: config.template.clone().unwrap_or_default(),
    }
}
