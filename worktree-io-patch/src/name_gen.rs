//! Random human-friendly workspace name generator (e.g. `"bold_turing"`).
use uuid::Uuid;

const ADJECTIVES: &[&str] = &[
    "bold", "bright", "brave", "calm", "clever", "eager", "fair", "gentle", "happy", "keen",
    "kind", "lively", "merry", "noble", "proud", "quiet", "swift", "warm", "wise", "free",
];

const NOUNS: &[&str] = &[
    "turing",
    "neumann",
    "lovelace",
    "hopper",
    "curie",
    "darwin",
    "euler",
    "gauss",
    "newton",
    "pascal",
    "ramanujan",
    "knuth",
    "dijkstra",
    "boole",
    "babbage",
    "shannon",
    "hawking",
    "einstein",
    "feynman",
    "tesla",
];

/// Generate a random workspace name in `adjective_noun` format.
///
/// Uses UUID v4 entropy so each call produces a unique name with high
/// probability. Example outputs: `"bold_turing"`, `"calm_lovelace"`.
#[must_use]
pub fn generate_name() -> String {
    let id = Uuid::new_v4();
    let b = id.as_bytes();
    let adj = ADJECTIVES[b[0] as usize % ADJECTIVES.len()];
    let noun = NOUNS[b[1] as usize % NOUNS.len()];
    format!("{adj}_{noun}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_is_adjective_underscore_noun() {
        let name = generate_name();
        let mut parts = name.splitn(2, '_');
        let adj = parts.next().expect("adjective part");
        let noun = parts.next().expect("noun part");
        assert!(ADJECTIVES.contains(&adj), "unknown adjective: {adj}");
        assert!(NOUNS.contains(&noun), "unknown noun: {noun}");
    }

    #[test]
    fn generates_distinct_names() {
        // 20×20 = 400 possible names; P(all 5 identical) ≈ (1/400)^4 ≈ 0
        let names: std::collections::HashSet<_> = (0..5).map(|_| generate_name()).collect();
        assert!(names.len() > 1, "5 calls all returned the same name");
    }
}
