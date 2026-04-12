use mdstore::StoreError;

pub fn validate_link_type(link_type: &str) -> Result<(), StoreError> {
    if link_type.is_empty() {
        return Err(StoreError::custom("link_type must not be empty"));
    }
    Ok(())
}

pub fn validate_link_ids(source_id: &str, target_id: &str) -> Result<(), StoreError> {
    if source_id.is_empty() {
        return Err(StoreError::custom("source_id must not be empty"));
    }
    if target_id.is_empty() {
        return Err(StoreError::custom("target_id must not be empty"));
    }
    Ok(())
}
