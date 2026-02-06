use chrono::NaiveDate;
use regex::Regex;

/// Represents a parsed search query
#[derive(Debug, Clone)]
pub enum Query {
    And(Box<Query>, Box<Query>),
    Or(Box<Query>, Box<Query>),
    Not(Box<Query>),
    Condition(Condition),
}

/// A single search condition
#[derive(Debug, Clone)]
pub struct Condition {
    pub field: Field,
    pub operator: Operator,
    pub value: Value,
}

/// Fields that can be searched
#[derive(Debug, Clone, PartialEq)]
pub enum Field {
    Title,
    Description,
    Status,
    Priority,
    DisplayNumber,
    CreatedAt,
    UpdatedAt,
    Custom(String),
}

impl Field {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "title" => Field::Title,
            "description" | "desc" => Field::Description,
            "status" => Field::Status,
            "priority" | "prio" | "p" => Field::Priority,
            "displaynumber" | "number" | "num" | "n" => Field::DisplayNumber,
            "createdat" | "created" => Field::CreatedAt,
            "updatedat" | "updated" => Field::UpdatedAt,
            other => Field::Custom(other.to_string()),
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Field::Priority | Field::DisplayNumber)
    }

    pub fn is_date(&self) -> bool {
        matches!(self, Field::CreatedAt | Field::UpdatedAt)
    }

    pub fn is_boolean(&self) -> bool {
        false
    }
}

/// Comparison operators
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    /// Equality (`:` or `=`)
    Eq,
    /// Not equal (`!=`)
    NotEq,
    /// Contains (`~`)
    Contains,
    /// Starts with (`^`)
    StartsWith,
    /// Ends with (`$`)
    EndsWith,
    /// Greater than (`>`)
    Gt,
    /// Less than (`<`)
    Lt,
    /// Greater than or equal (`>=`)
    Gte,
    /// Less than or equal (`<=`)
    Lte,
}

impl Operator {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            ":" | "=" => Some(Operator::Eq),
            "!=" => Some(Operator::NotEq),
            "~" => Some(Operator::Contains),
            "^" => Some(Operator::StartsWith),
            "$" => Some(Operator::EndsWith),
            ">" => Some(Operator::Gt),
            "<" => Some(Operator::Lt),
            ">=" => Some(Operator::Gte),
            "<=" => Some(Operator::Lte),
            _ => None,
        }
    }
}

/// A compiled pattern for wildcard matching
#[derive(Debug, Clone)]
pub struct CompiledPattern {
    pub regex: Regex,
    pub original: String,
}

impl CompiledPattern {
    pub fn from_wildcard(pattern: &str) -> Result<Self, regex::Error> {
        // Convert wildcard pattern to regex
        let mut regex_str = String::from("^");
        for ch in pattern.chars() {
            match ch {
                '*' => regex_str.push_str(".*"),
                '?' => regex_str.push('.'),
                '.' | '+' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '\\' => {
                    regex_str.push('\\');
                    regex_str.push(ch);
                }
                _ => regex_str.push(ch),
            }
        }
        regex_str.push('$');

        let regex = Regex::new(&regex_str)?;
        Ok(CompiledPattern {
            regex,
            original: pattern.to_string(),
        })
    }

    pub fn matches(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }
}

/// Value types for comparisons
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(i64),
    Boolean(bool),
    Date(NaiveDate),
    Pattern(CompiledPattern),
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Field tests ---

    #[test]
    fn test_field_from_str_title() {
        assert_eq!(Field::from_str("title"), Field::Title);
        assert_eq!(Field::from_str("TITLE"), Field::Title);
    }

    #[test]
    fn test_field_from_str_description() {
        assert_eq!(Field::from_str("description"), Field::Description);
        assert_eq!(Field::from_str("desc"), Field::Description);
    }

    #[test]
    fn test_field_from_str_status() {
        assert_eq!(Field::from_str("status"), Field::Status);
        assert_eq!(Field::from_str("STATUS"), Field::Status);
    }

    #[test]
    fn test_field_from_str_priority() {
        assert_eq!(Field::from_str("priority"), Field::Priority);
        assert_eq!(Field::from_str("prio"), Field::Priority);
        assert_eq!(Field::from_str("p"), Field::Priority);
    }

    #[test]
    fn test_field_from_str_display_number() {
        assert_eq!(Field::from_str("displaynumber"), Field::DisplayNumber);
        assert_eq!(Field::from_str("number"), Field::DisplayNumber);
        assert_eq!(Field::from_str("num"), Field::DisplayNumber);
        assert_eq!(Field::from_str("n"), Field::DisplayNumber);
    }

    #[test]
    fn test_field_from_str_dates() {
        assert_eq!(Field::from_str("createdat"), Field::CreatedAt);
        assert_eq!(Field::from_str("created"), Field::CreatedAt);
        assert_eq!(Field::from_str("updatedat"), Field::UpdatedAt);
        assert_eq!(Field::from_str("updated"), Field::UpdatedAt);
    }

    #[test]
    fn test_field_from_str_custom() {
        assert_eq!(
            Field::from_str("environment"),
            Field::Custom("environment".to_string())
        );
    }

    #[test]
    fn test_field_is_numeric() {
        assert!(Field::Priority.is_numeric());
        assert!(Field::DisplayNumber.is_numeric());
        assert!(!Field::Title.is_numeric());
        assert!(!Field::Status.is_numeric());
        assert!(!Field::CreatedAt.is_numeric());
    }

    #[test]
    fn test_field_is_date() {
        assert!(Field::CreatedAt.is_date());
        assert!(Field::UpdatedAt.is_date());
        assert!(!Field::Title.is_date());
        assert!(!Field::Priority.is_date());
    }

    #[test]
    fn test_field_is_boolean() {
        assert!(!Field::Title.is_boolean());
        assert!(!Field::Priority.is_boolean());
        assert!(!Field::Custom("flag".to_string()).is_boolean());
    }

    // --- Operator tests ---

    #[test]
    fn test_operator_from_str_eq() {
        assert_eq!(Operator::from_str(":"), Some(Operator::Eq));
        assert_eq!(Operator::from_str("="), Some(Operator::Eq));
    }

    #[test]
    fn test_operator_from_str_not_eq() {
        assert_eq!(Operator::from_str("!="), Some(Operator::NotEq));
    }

    #[test]
    fn test_operator_from_str_contains() {
        assert_eq!(Operator::from_str("~"), Some(Operator::Contains));
    }

    #[test]
    fn test_operator_from_str_starts_ends_with() {
        assert_eq!(Operator::from_str("^"), Some(Operator::StartsWith));
        assert_eq!(Operator::from_str("$"), Some(Operator::EndsWith));
    }

    #[test]
    fn test_operator_from_str_comparison() {
        assert_eq!(Operator::from_str(">"), Some(Operator::Gt));
        assert_eq!(Operator::from_str("<"), Some(Operator::Lt));
        assert_eq!(Operator::from_str(">="), Some(Operator::Gte));
        assert_eq!(Operator::from_str("<="), Some(Operator::Lte));
    }

    #[test]
    fn test_operator_from_str_invalid() {
        assert_eq!(Operator::from_str("??"), None);
        assert_eq!(Operator::from_str(""), None);
        assert_eq!(Operator::from_str("=="), None);
    }

    // --- CompiledPattern tests ---

    #[test]
    fn test_compiled_pattern_exact() {
        let pattern = CompiledPattern::from_wildcard("hello").unwrap();
        assert!(pattern.matches("hello"));
        assert!(!pattern.matches("hello world"));
        assert!(!pattern.matches("say hello"));
    }

    #[test]
    fn test_compiled_pattern_star_wildcard() {
        let pattern = CompiledPattern::from_wildcard("bug*").unwrap();
        assert!(pattern.matches("bug"));
        assert!(pattern.matches("bug-fix"));
        assert!(pattern.matches("bug report"));
        assert!(!pattern.matches("debug"));
    }

    #[test]
    fn test_compiled_pattern_question_wildcard() {
        let pattern = CompiledPattern::from_wildcard("v?.0").unwrap();
        assert!(pattern.matches("v1.0"));
        assert!(pattern.matches("v2.0"));
        assert!(!pattern.matches("v10.0"));
    }

    #[test]
    fn test_compiled_pattern_star_middle() {
        let pattern = CompiledPattern::from_wildcard("open*end").unwrap();
        assert!(pattern.matches("open-end"));
        assert!(pattern.matches("open-the-end"));
        assert!(!pattern.matches("opened"));
    }

    #[test]
    fn test_compiled_pattern_escapes_special_chars() {
        let pattern = CompiledPattern::from_wildcard("v1.0+build").unwrap();
        assert!(pattern.matches("v1.0+build"));
        assert!(!pattern.matches("v100build"));
    }

    #[test]
    fn test_compiled_pattern_original_preserved() {
        let pattern = CompiledPattern::from_wildcard("test*").unwrap();
        assert_eq!(pattern.original, "test*");
    }

    // --- Value tests ---

    #[test]
    fn test_value_string() {
        let val = Value::String("hello".to_string());
        let debug = format!("{val:?}");
        assert!(debug.contains("String"));
        assert!(debug.contains("hello"));
    }

    #[test]
    fn test_value_number() {
        let val = Value::Number(42);
        let debug = format!("{val:?}");
        assert!(debug.contains("Number"));
        assert!(debug.contains("42"));
    }

    #[test]
    fn test_value_boolean() {
        let val = Value::Boolean(true);
        let debug = format!("{val:?}");
        assert!(debug.contains("Boolean"));
        assert!(debug.contains("true"));
    }

    #[test]
    fn test_value_date() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let val = Value::Date(date);
        let debug = format!("{val:?}");
        assert!(debug.contains("Date"));
        assert!(debug.contains("2024-06-15"));
    }

    // --- Query tests ---

    #[test]
    fn test_query_condition_debug() {
        let query = Query::Condition(Condition {
            field: Field::Title,
            operator: Operator::Contains,
            value: Value::String("bug".to_string()),
        });
        let debug = format!("{query:?}");
        assert!(debug.contains("Condition"));
        assert!(debug.contains("Title"));
        assert!(debug.contains("Contains"));
    }

    #[test]
    fn test_query_and() {
        let q1 = Query::Condition(Condition {
            field: Field::Status,
            operator: Operator::Eq,
            value: Value::String("open".to_string()),
        });
        let q2 = Query::Condition(Condition {
            field: Field::Priority,
            operator: Operator::Eq,
            value: Value::Number(1),
        });
        let query = Query::And(Box::new(q1), Box::new(q2));
        let debug = format!("{query:?}");
        assert!(debug.contains("And"));
    }

    #[test]
    fn test_query_or() {
        let q1 = Query::Condition(Condition {
            field: Field::Status,
            operator: Operator::Eq,
            value: Value::String("open".to_string()),
        });
        let q2 = Query::Condition(Condition {
            field: Field::Status,
            operator: Operator::Eq,
            value: Value::String("closed".to_string()),
        });
        let query = Query::Or(Box::new(q1), Box::new(q2));
        let debug = format!("{query:?}");
        assert!(debug.contains("Or"));
    }

    #[test]
    fn test_query_not() {
        let inner = Query::Condition(Condition {
            field: Field::Status,
            operator: Operator::Eq,
            value: Value::String("closed".to_string()),
        });
        let query = Query::Not(Box::new(inner));
        let debug = format!("{query:?}");
        assert!(debug.contains("Not"));
    }

    // --- Condition tests ---

    #[test]
    fn test_condition_clone() {
        let cond = Condition {
            field: Field::Title,
            operator: Operator::Contains,
            value: Value::String("test".to_string()),
        };
        let cloned = cond.clone();
        assert_eq!(cloned.field, Field::Title);
        assert_eq!(cloned.operator, Operator::Contains);
    }
}
