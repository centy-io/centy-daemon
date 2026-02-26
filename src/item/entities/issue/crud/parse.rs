use super::super::planning::remove_planning_note;

pub fn parse_issue_md(content: &str) -> (String, String) {
    let content = remove_planning_note(content);
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return (String::new(), String::new());
    }
    let mut title_idx = 0;
    for (idx, line) in lines.iter().enumerate() {
        if line.starts_with('#') {
            title_idx = idx;
            break;
        }
    }
    let title = lines
        .get(title_idx)
        .map(|line| line.strip_prefix('#').map_or(*line, str::trim))
        .unwrap_or("")
        .to_string();
    let description = lines
        .get(title_idx.saturating_add(1)..)
        .unwrap_or(&[])
        .iter()
        .skip_while(|line| line.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string();
    (title, description)
}

pub fn generate_issue_md(title: &str, description: &str) -> String {
    if description.is_empty() {
        format!("# {title}\n")
    } else {
        format!("# {title}\n\n{description}\n")
    }
}
