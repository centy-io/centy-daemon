use chrono::NaiveDate;

use super::ast::{Condition, Field, Operator, Query, Value};
use crate::item::entities::issue::Issue;

/// Evaluate a query against an issue
pub fn evaluate(query: &Query, issue: &Issue) -> bool {
    match query {
        Query::And(left, right) => evaluate(left, issue) && evaluate(right, issue),
        Query::Or(left, right) => evaluate(left, issue) || evaluate(right, issue),
        Query::Not(inner) => !evaluate(inner, issue),
        Query::Condition(condition) => evaluate_condition(condition, issue),
    }
}

fn evaluate_condition(condition: &Condition, issue: &Issue) -> bool {
    let field_value = get_field_value(&condition.field, issue);

    match field_value {
        FieldValue::String(s) => {
            evaluate_string_condition(&condition.operator, &s, &condition.value)
        }
        FieldValue::Number(n) => {
            evaluate_number_condition(&condition.operator, n, &condition.value)
        }
        FieldValue::Date(d) => evaluate_date_condition(&condition.operator, &d, &condition.value),
        FieldValue::None => false,
    }
}

/// Internal enum for field values
enum FieldValue {
    String(String),
    Number(i64),
    Date(NaiveDate),
    None,
}

fn get_field_value(field: &Field, issue: &Issue) -> FieldValue {
    match field {
        Field::Title => FieldValue::String(issue.title.clone()),
        Field::Description => FieldValue::String(issue.description.clone()),
        Field::Status => FieldValue::String(issue.metadata.status.clone()),
        Field::Priority => FieldValue::Number(i64::from(issue.metadata.priority)),
        Field::DisplayNumber => FieldValue::Number(i64::from(issue.metadata.display_number)),
        Field::CreatedAt => parse_date(&issue.metadata.created_at)
            .map(FieldValue::Date)
            .unwrap_or(FieldValue::None),
        Field::UpdatedAt => parse_date(&issue.metadata.updated_at)
            .map(FieldValue::Date)
            .unwrap_or(FieldValue::None),
        Field::Custom(name) => issue
            .metadata
            .custom_fields
            .get(name)
            .map(|v| FieldValue::String(v.clone()))
            .unwrap_or(FieldValue::None),
    }
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    // Try ISO format first (from timestamps like "2024-01-15T10:30:00Z")
    if let Some(date_part) = s.split('T').next() {
        if let Ok(date) = NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
            return Some(date);
        }
    }
    // Try direct date format
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

fn evaluate_string_condition(operator: &Operator, field_value: &str, query_value: &Value) -> bool {
    let field_lower = field_value.to_lowercase();

    match query_value {
        Value::String(s) => {
            let query_lower = s.to_lowercase();
            match operator {
                Operator::Eq => field_lower == query_lower,
                Operator::NotEq => field_lower != query_lower,
                Operator::Contains => field_lower.contains(&query_lower),
                Operator::StartsWith => field_lower.starts_with(&query_lower),
                Operator::EndsWith => field_lower.ends_with(&query_lower),
                // For string comparisons, use lexicographic ordering
                Operator::Gt => field_lower > query_lower,
                Operator::Lt => field_lower < query_lower,
                Operator::Gte => field_lower >= query_lower,
                Operator::Lte => field_lower <= query_lower,
            }
        }
        Value::Pattern(pattern) => {
            // Case-insensitive pattern matching
            pattern.matches(&field_lower)
        }
        // Other value types don't match strings
        _ => false,
    }
}

fn evaluate_number_condition(operator: &Operator, field_value: i64, query_value: &Value) -> bool {
    match query_value {
        Value::Number(n) => {
            match operator {
                Operator::Eq => field_value == *n,
                Operator::NotEq => field_value != *n,
                Operator::Gt => field_value > *n,
                Operator::Lt => field_value < *n,
                Operator::Gte => field_value >= *n,
                Operator::Lte => field_value <= *n,
                // These don't make sense for numbers
                Operator::Contains | Operator::StartsWith | Operator::EndsWith => false,
            }
        }
        // Other value types don't match numbers
        _ => false,
    }
}

fn evaluate_date_condition(
    operator: &Operator,
    field_value: &NaiveDate,
    query_value: &Value,
) -> bool {
    match query_value {
        Value::Date(d) => {
            match operator {
                Operator::Eq => field_value == d,
                Operator::NotEq => field_value != d,
                Operator::Gt => field_value > d,
                Operator::Lt => field_value < d,
                Operator::Gte => field_value >= d,
                Operator::Lte => field_value <= d,
                // These don't make sense for dates
                Operator::Contains | Operator::StartsWith | Operator::EndsWith => false,
            }
        }
        // Other value types don't match dates
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::entities::issue::IssueMetadataFlat;
    use crate::search::ast::CompiledPattern;
    use std::collections::HashMap;

    #[allow(deprecated)]
    fn create_test_issue(title: &str, status: &str, priority: u32) -> Issue {
        Issue {
            id: "test-uuid".to_string(),
            issue_number: "test-uuid".to_string(),
            title: title.to_string(),
            description: "Test description".to_string(),
            metadata: IssueMetadataFlat {
                status: status.to_string(),
                priority,
                display_number: 1,
                created_at: "2024-06-15T10:30:00Z".to_string(),
                updated_at: "2024-06-15T10:30:00Z".to_string(),
                custom_fields: HashMap::new(),
                draft: false,
                deleted_at: None,
                is_org_issue: false,
                org_slug: None,
                org_display_number: None,
            },
        }
    }

    #[test]
    fn test_string_equality() {
        let issue = create_test_issue("Bug fix", "open", 1);
        let condition = Condition {
            field: Field::Status,
            operator: Operator::Eq,
            value: Value::String("open".to_string()),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_string_contains() {
        let issue = create_test_issue("Bug fix for login", "open", 1);
        let condition = Condition {
            field: Field::Title,
            operator: Operator::Contains,
            value: Value::String("login".to_string()),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_number_comparison() {
        let issue = create_test_issue("Bug fix", "open", 2);
        let condition = Condition {
            field: Field::Priority,
            operator: Operator::Lte,
            value: Value::Number(2),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_and_query() {
        let issue = create_test_issue("Bug fix", "open", 1);
        let query = Query::And(
            Box::new(Query::Condition(Condition {
                field: Field::Status,
                operator: Operator::Eq,
                value: Value::String("open".to_string()),
            })),
            Box::new(Query::Condition(Condition {
                field: Field::Priority,
                operator: Operator::Eq,
                value: Value::Number(1),
            })),
        );
        assert!(evaluate(&query, &issue));
    }

    #[test]
    fn test_or_query() {
        let issue = create_test_issue("Bug fix", "closed", 1);
        let query = Query::Or(
            Box::new(Query::Condition(Condition {
                field: Field::Status,
                operator: Operator::Eq,
                value: Value::String("open".to_string()),
            })),
            Box::new(Query::Condition(Condition {
                field: Field::Status,
                operator: Operator::Eq,
                value: Value::String("closed".to_string()),
            })),
        );
        assert!(evaluate(&query, &issue));
    }

    #[test]
    fn test_not_query() {
        let issue = create_test_issue("Bug fix", "open", 1);
        let query = Query::Not(Box::new(Query::Condition(Condition {
            field: Field::Status,
            operator: Operator::Eq,
            value: Value::String("closed".to_string()),
        })));
        assert!(evaluate(&query, &issue));
    }

    #[test]
    fn test_string_not_eq() {
        let issue = create_test_issue("Bug fix", "open", 1);
        let condition = Condition {
            field: Field::Status,
            operator: Operator::NotEq,
            value: Value::String("closed".to_string()),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_string_starts_with() {
        let issue = create_test_issue("Bug: login failure", "open", 1);
        let condition = Condition {
            field: Field::Title,
            operator: Operator::StartsWith,
            value: Value::String("bug".to_string()),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_string_ends_with() {
        let issue = create_test_issue("Login failure bug", "open", 1);
        let condition = Condition {
            field: Field::Title,
            operator: Operator::EndsWith,
            value: Value::String("bug".to_string()),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_string_gt() {
        let issue = create_test_issue("Bug fix", "open", 1);
        let condition = Condition {
            field: Field::Status,
            operator: Operator::Gt,
            value: Value::String("a".to_string()),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_number_eq() {
        let issue = create_test_issue("Bug fix", "open", 2);
        let condition = Condition {
            field: Field::Priority,
            operator: Operator::Eq,
            value: Value::Number(2),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_number_not_eq() {
        let issue = create_test_issue("Bug fix", "open", 2);
        let condition = Condition {
            field: Field::Priority,
            operator: Operator::NotEq,
            value: Value::Number(1),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_number_gt() {
        let issue = create_test_issue("Bug fix", "open", 3);
        let condition = Condition {
            field: Field::Priority,
            operator: Operator::Gt,
            value: Value::Number(2),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_number_lt() {
        let issue = create_test_issue("Bug fix", "open", 1);
        let condition = Condition {
            field: Field::Priority,
            operator: Operator::Lt,
            value: Value::Number(2),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_number_gte() {
        let issue = create_test_issue("Bug fix", "open", 2);
        let condition = Condition {
            field: Field::Priority,
            operator: Operator::Gte,
            value: Value::Number(2),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_number_contains_returns_false() {
        let issue = create_test_issue("Bug fix", "open", 2);
        let condition = Condition {
            field: Field::Priority,
            operator: Operator::Contains,
            value: Value::Number(2),
        };
        assert!(!evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_number_wrong_value_type() {
        let issue = create_test_issue("Bug fix", "open", 2);
        let condition = Condition {
            field: Field::Priority,
            operator: Operator::Eq,
            value: Value::String("2".to_string()),
        };
        assert!(!evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_date_comparison() {
        let issue = create_test_issue("Bug fix", "open", 1);
        let date = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let condition = Condition {
            field: Field::CreatedAt,
            operator: Operator::Eq,
            value: Value::Date(date),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_date_gt() {
        let issue = create_test_issue("Bug fix", "open", 1);
        let date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let condition = Condition {
            field: Field::CreatedAt,
            operator: Operator::Gt,
            value: Value::Date(date),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_date_contains_returns_false() {
        let issue = create_test_issue("Bug fix", "open", 1);
        let date = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let condition = Condition {
            field: Field::CreatedAt,
            operator: Operator::Contains,
            value: Value::Date(date),
        };
        assert!(!evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_display_number_field() {
        let issue = create_test_issue("Bug fix", "open", 1);
        let condition = Condition {
            field: Field::DisplayNumber,
            operator: Operator::Eq,
            value: Value::Number(1),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_description_field() {
        let issue = create_test_issue("Bug fix", "open", 1);
        let condition = Condition {
            field: Field::Description,
            operator: Operator::Contains,
            value: Value::String("test".to_string()),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_custom_field_not_found() {
        let issue = create_test_issue("Bug fix", "open", 1);
        let condition = Condition {
            field: Field::Custom("nonexistent".to_string()),
            operator: Operator::Eq,
            value: Value::String("value".to_string()),
        };
        assert!(!evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_case_insensitive_matching() {
        let issue = create_test_issue("Bug Fix", "Open", 1);
        let condition = Condition {
            field: Field::Status,
            operator: Operator::Eq,
            value: Value::String("open".to_string()),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_wildcard_pattern_matching() {
        let issue = create_test_issue("Bug fix for login", "open", 1);
        let pattern = CompiledPattern::from_wildcard("bug*login").unwrap();
        let condition = Condition {
            field: Field::Title,
            operator: Operator::Contains,
            value: Value::Pattern(pattern),
        };
        assert!(evaluate_condition(&condition, &issue));
    }

    #[test]
    fn test_and_query_false() {
        let issue = create_test_issue("Bug fix", "closed", 1);
        let query = Query::And(
            Box::new(Query::Condition(Condition {
                field: Field::Status,
                operator: Operator::Eq,
                value: Value::String("open".to_string()),
            })),
            Box::new(Query::Condition(Condition {
                field: Field::Priority,
                operator: Operator::Eq,
                value: Value::Number(1),
            })),
        );
        assert!(!evaluate(&query, &issue));
    }

    #[test]
    fn test_or_query_both_false() {
        let issue = create_test_issue("Bug fix", "in-progress", 3);
        let query = Query::Or(
            Box::new(Query::Condition(Condition {
                field: Field::Status,
                operator: Operator::Eq,
                value: Value::String("open".to_string()),
            })),
            Box::new(Query::Condition(Condition {
                field: Field::Status,
                operator: Operator::Eq,
                value: Value::String("closed".to_string()),
            })),
        );
        assert!(!evaluate(&query, &issue));
    }

    #[test]
    fn test_not_query_false() {
        let issue = create_test_issue("Bug fix", "open", 1);
        let query = Query::Not(Box::new(Query::Condition(Condition {
            field: Field::Status,
            operator: Operator::Eq,
            value: Value::String("open".to_string()),
        })));
        assert!(!evaluate(&query, &issue));
    }
}
