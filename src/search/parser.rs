use pest::Parser;
use pest_derive::Parser;

use super::ast::{CompiledPattern, Condition, Field, Operator, Query, Value};
use super::error::SearchError;

#[derive(Parser)]
#[grammar = "search/grammar.pest"]
struct QueryParser;

/// Parse a query string into a Query AST
pub fn parse_query(input: &str) -> Result<Option<Query>, SearchError> {
    let input = input.trim();

    // Empty query matches all
    if input.is_empty() {
        return Ok(None);
    }

    let pairs = QueryParser::parse(Rule::query, input)
        .map_err(|e| SearchError::ParseError(format!("{e}")))?;

    let query_pair = pairs
        .into_iter()
        .next()
        .ok_or_else(|| SearchError::ParseError("Empty parse result".to_string()))?;

    parse_query_inner(query_pair).map(Some)
}

fn parse_query_inner(pair: pest::iterators::Pair<Rule>) -> Result<Query, SearchError> {
    // Navigate through the parse tree
    match pair.as_rule() {
        Rule::query => {
            // query -> expr -> or_expr -> ...
            let inner = pair.into_inner().next()
                .ok_or_else(|| SearchError::ParseError("Empty query".to_string()))?;
            parse_query_inner(inner)
        }
        Rule::expr => {
            let inner = pair.into_inner().next()
                .ok_or_else(|| SearchError::ParseError("Empty expr".to_string()))?;
            parse_query_inner(inner)
        }
        Rule::or_expr => parse_or_expr(pair),
        Rule::and_expr => parse_and_expr(pair),
        Rule::not_expr => parse_not_expr(pair),
        Rule::primary => parse_primary(pair),
        Rule::condition => parse_condition(pair),
        _ => Err(SearchError::ParseError(format!(
            "Unexpected rule: {:?}",
            pair.as_rule()
        ))),
    }
}

fn parse_or_expr(pair: pest::iterators::Pair<Rule>) -> Result<Query, SearchError> {
    let mut inner = pair.into_inner().peekable();

    let first = inner
        .next()
        .ok_or_else(|| SearchError::ParseError("Expected expression in OR".to_string()))?;
    let mut result = parse_query_inner(first)?;

    while let Some(next) = inner.next() {
        // Skip the OR operator token
        if next.as_rule() == Rule::or_op {
            let right_expr = inner
                .next()
                .ok_or_else(|| SearchError::ParseError("Expected expression after OR".to_string()))?;
            let right = parse_query_inner(right_expr)?;
            result = Query::Or(Box::new(result), Box::new(right));
        } else {
            // Shouldn't happen but handle it
            let right = parse_query_inner(next)?;
            result = Query::Or(Box::new(result), Box::new(right));
        }
    }

    Ok(result)
}

fn parse_and_expr(pair: pest::iterators::Pair<Rule>) -> Result<Query, SearchError> {
    let mut inner = pair.into_inner().peekable();

    let first = inner
        .next()
        .ok_or_else(|| SearchError::ParseError("Expected expression in AND".to_string()))?;
    let mut result = parse_query_inner(first)?;

    while let Some(next) = inner.next() {
        // Skip the AND operator token
        if next.as_rule() == Rule::and_op {
            let right_expr = inner
                .next()
                .ok_or_else(|| SearchError::ParseError("Expected expression after AND".to_string()))?;
            let right = parse_query_inner(right_expr)?;
            result = Query::And(Box::new(result), Box::new(right));
        } else {
            // Shouldn't happen but handle it
            let right = parse_query_inner(next)?;
            result = Query::And(Box::new(result), Box::new(right));
        }
    }

    Ok(result)
}

fn parse_not_expr(pair: pest::iterators::Pair<Rule>) -> Result<Query, SearchError> {
    let mut inner = pair.into_inner().peekable();

    // Check if this is a NOT expression
    if let Some(first) = inner.peek() {
        if first.as_rule() == Rule::not_op {
            inner.next(); // consume NOT
            let operand = inner
                .next()
                .ok_or_else(|| SearchError::ParseError("Expected expression after NOT".to_string()))?;
            let inner_query = parse_query_inner(operand)?;
            return Ok(Query::Not(Box::new(inner_query)));
        }
    }

    // Otherwise parse the child (primary)
    let child = inner
        .next()
        .ok_or_else(|| SearchError::ParseError("Expected expression in NOT".to_string()))?;
    parse_query_inner(child)
}

fn parse_primary(pair: pest::iterators::Pair<Rule>) -> Result<Query, SearchError> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| SearchError::ParseError("Empty primary".to_string()))?;

    match inner.as_rule() {
        Rule::expr => parse_query_inner(inner),
        Rule::condition => parse_condition(inner),
        _ => Err(SearchError::ParseError(format!(
            "Unexpected rule in primary: {:?}",
            inner.as_rule()
        ))),
    }
}

fn parse_condition(pair: pest::iterators::Pair<Rule>) -> Result<Query, SearchError> {
    let mut inner = pair.into_inner();

    let field_pair = inner
        .next()
        .ok_or_else(|| SearchError::ParseError("Expected field in condition".to_string()))?;
    let field_str = field_pair.as_str();
    let field = Field::from_str(field_str);

    // Collect remaining pairs
    let remaining: Vec<_> = inner.collect();

    let (operator, value_str) = match remaining.len() {
        1 => {
            // Just value, no explicit operator - use default
            let value_pair = &remaining[0];
            (default_operator(&field), value_pair.clone())
        }
        2 => {
            // Operator and value
            let op_pair = &remaining[0];
            let value_pair = &remaining[1];
            let op_str = op_pair.as_str();
            let operator = Operator::from_str(op_str).ok_or_else(|| {
                SearchError::InvalidOperator(op_str.to_string(), field_str.to_string())
            })?;
            (operator, value_pair.clone())
        }
        _ => {
            return Err(SearchError::ParseError(format!(
                "Unexpected number of parts in condition: {}",
                remaining.len()
            )))
        }
    };

    let value = parse_value(value_str, &field, &operator)?;

    Ok(Query::Condition(Condition {
        field,
        operator,
        value,
    }))
}

fn default_operator(field: &Field) -> Operator {
    if field.is_numeric() || field.is_date() || field.is_boolean() {
        Operator::Eq
    } else {
        // For text fields, default to contains
        Operator::Contains
    }
}

fn parse_value(
    pair: pest::iterators::Pair<Rule>,
    field: &Field,
    _operator: &Operator,
) -> Result<Value, SearchError> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| SearchError::ParseError("Empty value".to_string()))?;

    match inner.as_rule() {
        Rule::quoted_string => {
            // Remove surrounding quotes and handle escapes
            let raw = inner.as_str();
            let unquoted = &raw[1..raw.len() - 1];
            let unescaped = unescape_string(unquoted);
            Ok(Value::String(unescaped))
        }
        Rule::boolean_value => {
            let s = inner.as_str().to_lowercase();
            let b = s == "true";
            Ok(Value::Boolean(b))
        }
        Rule::date_value => {
            let date_str = inner.as_str();
            let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|_| SearchError::InvalidDateFormat(date_str.to_string()))?;
            Ok(Value::Date(date))
        }
        Rule::number => {
            let num_str = inner.as_str();
            let num: i64 = num_str.parse().map_err(|_| {
                SearchError::InvalidValue(num_str.to_string(), "Invalid number".to_string())
            })?;
            Ok(Value::Number(num))
        }
        Rule::wildcard_or_word => {
            let text = inner.as_str();

            // Check if it's a wildcard pattern
            if text.contains('*') || text.contains('?') {
                // For wildcard operator or if pattern has wildcards
                let pattern = CompiledPattern::from_wildcard(text)
                    .map_err(|e| SearchError::InvalidRegex(text.to_string(), e.to_string()))?;
                Ok(Value::Pattern(pattern))
            } else if field.is_numeric() {
                // Try to parse as number for numeric fields
                let num: i64 = text.parse().map_err(|_| {
                    SearchError::InvalidValue(
                        text.to_string(),
                        format!("Expected number for field {field:?}"),
                    )
                })?;
                Ok(Value::Number(num))
            } else if field.is_boolean() {
                // Try to parse as boolean
                let b = match text.to_lowercase().as_str() {
                    "true" | "yes" | "1" => true,
                    "false" | "no" | "0" => false,
                    _ => {
                        return Err(SearchError::InvalidValue(
                            text.to_string(),
                            "Expected boolean (true/false)".to_string(),
                        ))
                    }
                };
                Ok(Value::Boolean(b))
            } else if field.is_date() {
                // Try to parse as date
                let date = chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d")
                    .map_err(|_| SearchError::InvalidDateFormat(text.to_string()))?;
                Ok(Value::Date(date))
            } else {
                // Plain string
                Ok(Value::String(text.to_string()))
            }
        }
        _ => Err(SearchError::ParseError(format!(
            "Unexpected value rule: {:?}",
            inner.as_rule()
        ))),
    }
}

fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                match next {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    _ => {
                        result.push('\\');
                        result.push(next);
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Format a parsed query back to a string representation (for debugging)
pub fn format_query(query: &Query) -> String {
    match query {
        Query::And(left, right) => format!("({} AND {})", format_query(left), format_query(right)),
        Query::Or(left, right) => format!("({} OR {})", format_query(left), format_query(right)),
        Query::Not(inner) => format!("NOT {}", format_query(inner)),
        Query::Condition(cond) => format_condition(cond),
    }
}

fn format_condition(cond: &Condition) -> String {
    let field = format_field(&cond.field);
    let op = format_operator(&cond.operator);
    let value = format_value(&cond.value);
    format!("{field}{op}{value}")
}

fn format_field(field: &Field) -> String {
    match field {
        Field::Title => "title".to_string(),
        Field::Description => "description".to_string(),
        Field::Status => "status".to_string(),
        Field::Priority => "priority".to_string(),
        Field::DisplayNumber => "displayNumber".to_string(),
        Field::CreatedAt => "createdAt".to_string(),
        Field::UpdatedAt => "updatedAt".to_string(),
        Field::Compacted => "compacted".to_string(),
        Field::Custom(name) => name.clone(),
    }
}

fn format_operator(op: &Operator) -> String {
    match op {
        Operator::Eq => ":".to_string(),
        Operator::NotEq => "!=".to_string(),
        Operator::Contains => "~".to_string(),
        Operator::StartsWith => "^".to_string(),
        Operator::EndsWith => "$".to_string(),
        Operator::Gt => ">".to_string(),
        Operator::Lt => "<".to_string(),
        Operator::Gte => ">=".to_string(),
        Operator::Lte => "<=".to_string(),
    }
}

fn format_value(value: &Value) -> String {
    match value {
        Value::String(s) => {
            if s.contains(' ') {
                format!("\"{s}\"")
            } else {
                s.clone()
            }
        }
        Value::Number(n) => n.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Date(d) => d.format("%Y-%m-%d").to_string(),
        Value::Pattern(p) => p.original.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_condition() {
        let result = parse_query("status:open").unwrap();
        assert!(result.is_some());
        let query = result.unwrap();
        if let Query::Condition(cond) = query {
            assert_eq!(cond.field, Field::Status);
            assert_eq!(cond.operator, Operator::Eq);
        } else {
            panic!("Expected condition");
        }
    }

    #[test]
    fn test_quoted_string() {
        let result = parse_query("title:\"bug fix\"").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_and_expression() {
        let result = parse_query("status:open AND priority:1").unwrap();
        assert!(result.is_some());
        let query = result.unwrap();
        if let Query::And(_, _) = query {
            // OK
        } else {
            panic!("Expected AND expression");
        }
    }

    #[test]
    fn test_or_expression() {
        let result = parse_query("status:open OR status:planning").unwrap();
        assert!(result.is_some());
        let query = result.unwrap();
        if let Query::Or(_, _) = query {
            // OK
        } else {
            panic!("Expected OR expression");
        }
    }

    #[test]
    fn test_not_expression() {
        let result = parse_query("NOT status:closed").unwrap();
        assert!(result.is_some());
        let query = result.unwrap();
        if let Query::Not(_) = query {
            // OK
        } else {
            panic!("Expected NOT expression");
        }
    }

    #[test]
    fn test_grouped_expression() {
        let result = parse_query("(status:open OR status:planning) AND priority:1").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_date_comparison() {
        let result = parse_query("createdAt>2024-01-01").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_wildcard_pattern() {
        let result = parse_query("title:bug*").unwrap();
        assert!(result.is_some());
        let query = result.unwrap();
        if let Query::Condition(cond) = query {
            if let Value::Pattern(_) = cond.value {
                // OK
            } else {
                panic!("Expected pattern value");
            }
        } else {
            panic!("Expected condition");
        }
    }

    #[test]
    fn test_empty_query() {
        let result = parse_query("").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_complex_query() {
        let result = parse_query(
            "(status:open OR status:planning) AND priority<=2"
        ).unwrap();
        assert!(result.is_some());
    }
}
