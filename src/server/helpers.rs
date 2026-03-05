/// Convert an empty string to `None`, non-empty to `Some`.
pub fn nonempty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Convert a protobuf `int32` to `Option<u32>`: 0 or negative means "not set".
pub fn nonzero_u32(v: i32) -> Option<u32> {
    u32::try_from(v).ok().filter(|&n| n != 0)
}
