//! Token parsing and reordering for FST output
//!
//! This module parses the tagged token output from tagger FST and reorders
//! fields according to predefined orders for each token type.

use std::collections::HashMap;

use crate::config::{Language, Operator};
use crate::error::{Result, WeTextError};

/// Token structure representing a tagged entity
#[derive(Debug, Clone)]
pub struct Token {
    /// Token type name (e.g., "date", "money", "time")
    pub name: String,
    /// Order of fields as they were parsed
    pub order: Vec<String>,
    /// Field key-value pairs
    pub members: HashMap<String, String>,
}

impl Token {
    /// Create a new token with the given name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            order: Vec::new(),
            members: HashMap::new(),
        }
    }

    /// Append a key-value pair to the token
    pub fn append(&mut self, key: &str, value: &str) {
        self.order.push(key.to_string());
        self.members.insert(key.to_string(), value.to_string());
    }

    /// Convert token to string with specified field order
    pub fn to_string_with_order(&self, orders: &HashMap<String, Vec<String>>) -> String {
        let mut output = format!("{} {{", self.name);

        // Determine field order
        let order = if let Some(defined_order) = orders.get(&self.name) {
            if self.members.get("preserve_order") != Some(&"true".to_string()) {
                defined_order.clone()
            } else {
                self.order.clone()
            }
        } else {
            self.order.clone()
        };

        for key in &order {
            if let Some(value) = self.members.get(key) {
                output.push_str(&format!(" {}: \"{}\"", key, value));
            }
        }

        output.push_str(" }");
        output
    }
}

/// Token parser for reordering FST output fields
///
/// Parses tagger FST output and reorders fields according to predefined orders
pub struct TokenParser {
    orders: HashMap<String, Vec<String>>,
}

impl TokenParser {
    /// Field contract of the frozen official Chinese WeTextProcessing 1.2.0 grammar.
    pub fn chinese_itn_v120() -> Self {
        let mut orders = Self::itn_orders();
        for (name, fields) in [
            ("date", vec!["year", "month", "day", "preserve_order"]),
            (
                "measure",
                vec!["numerator", "denominator", "value", "units"],
            ),
            ("money", vec!["currency", "value", "decimal", "quantity"]),
            ("time", vec!["hour", "minute", "second", "noon", "zone"]),
            ("telephone", vec!["country_code", "number_part"]),
            ("electronic", vec!["username", "domain", "protocol"]),
        ] {
            orders.insert(name.into(), fields.into_iter().map(String::from).collect());
        }
        Self { orders }
    }

    /// Parse tagged output without the legacy pass-through fallback.
    pub fn reorder_strict(&self, input: &str) -> Result<String> {
        if input.is_empty() {
            return Ok(String::new());
        }
        let tokens = self.parse(input)?;
        if tokens.is_empty() {
            return Err(WeTextError::TokenParseError("empty token plan".into()));
        }
        Ok(tokens
            .iter()
            .map(|token| token.to_string_with_order(&self.orders))
            .collect::<Vec<_>>()
            .join(" "))
    }

    /// Create a new token parser for the given language and operator
    ///
    /// # Arguments
    /// * `lang` - Language type
    /// * `operator` - Operation type (TN or ITN)
    pub fn new(lang: Language, operator: Operator) -> Self {
        // Note: Japanese uses the same orders as Chinese (matching Python behavior)
        // English ITN is not supported in Python (raises NotImplementedError),
        // so we return empty HashMap which means fields keep original order
        let orders = match (lang, operator) {
            (Language::En, Operator::Tn) => Self::en_tn_orders(),
            (Language::Zh | Language::Ja, Operator::Tn) => Self::tn_orders(),
            (Language::Zh | Language::Ja, Operator::Itn) => Self::itn_orders(),
            _ => HashMap::new(), // English ITN: not supported, use original order
        };

        Self { orders }
    }

    /// Chinese/Japanese TN field orders
    fn tn_orders() -> HashMap<String, Vec<String>> {
        let mut m = HashMap::new();
        m.insert(
            "date".to_string(),
            vec!["year", "month", "day"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        m.insert(
            "fraction".to_string(),
            vec!["denominator", "numerator"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        m.insert(
            "measure".to_string(),
            vec!["denominator", "numerator", "value"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        m.insert(
            "money".to_string(),
            vec!["value", "currency"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        m.insert(
            "time".to_string(),
            vec!["noon", "hour", "minute", "second"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        m
    }

    /// English TN field orders
    fn en_tn_orders() -> HashMap<String, Vec<String>> {
        let mut m = HashMap::new();
        m.insert(
            "date".to_string(),
            vec!["preserve_order", "text", "day", "month", "year"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        m.insert(
            "money".to_string(),
            vec![
                "integer_part",
                "fractional_part",
                "quantity",
                "currency_maj",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        m
    }

    /// ITN field orders
    fn itn_orders() -> HashMap<String, Vec<String>> {
        let mut m = HashMap::new();
        m.insert(
            "date".to_string(),
            vec!["year", "month", "day"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        m.insert(
            "fraction".to_string(),
            vec!["sign", "numerator", "denominator"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        m.insert(
            "measure".to_string(),
            vec!["numerator", "denominator", "value"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        m.insert(
            "money".to_string(),
            vec!["currency", "value", "decimal"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        m.insert(
            "time".to_string(),
            vec!["hour", "minute", "second", "noon"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        m
    }

    /// Reorder token string fields according to predefined orders
    ///
    /// # Arguments
    /// * `input` - Tagged token string from tagger FST
    ///
    /// # Returns
    /// Reordered token string, or original input if not in token format
    pub fn reorder(&self, input: &str) -> Result<String> {
        // Handle empty input gracefully
        if input.trim().is_empty() {
            return Ok(String::new());
        }

        // Check if input looks like token format (contains '{')
        // If not, return as-is (non-token output from FST)
        if !input.contains('{') {
            return Ok(input.to_string());
        }

        match self.parse(input) {
            Ok(tokens) => {
                let output: Vec<String> = tokens
                    .iter()
                    .map(|t| t.to_string_with_order(&self.orders))
                    .collect();
                Ok(output.join(" "))
            }
            Err(_) => {
                // If parsing fails, return original input
                Ok(input.to_string())
            }
        }
    }

    /// Parse token string into structured tokens
    ///
    /// Expected format: `token_name { key1: "value1" key2: "value2" }`
    fn parse(&self, input: &str) -> Result<Vec<Token>> {
        let chars: Vec<char> = input.chars().collect();
        let mut pos = 0;
        let mut tokens = Vec::new();
        let error = || WeTextError::TokenParseError("malformed tagged output".into());
        let skip = |pos: &mut usize| {
            while *pos < chars.len() && chars[*pos].is_whitespace() {
                *pos += 1;
            }
        };
        while pos < chars.len() {
            skip(&mut pos);
            if pos == chars.len() {
                break;
            }
            let start = pos;
            while pos < chars.len() && (chars[pos].is_ascii_alphabetic() || chars[pos] == '_') {
                pos += 1;
            }
            if start == pos {
                return Err(error());
            }
            let name: String = chars[start..pos].iter().collect();
            skip(&mut pos);
            if chars.get(pos) != Some(&'{') {
                return Err(error());
            }
            pos += 1;
            let mut token = Token::new(&name);
            loop {
                skip(&mut pos);
                if chars.get(pos) == Some(&'}') {
                    pos += 1;
                    break;
                }
                let start = pos;
                while pos < chars.len() && (chars[pos].is_ascii_alphabetic() || chars[pos] == '_') {
                    pos += 1;
                }
                if start == pos {
                    return Err(error());
                }
                let key: String = chars[start..pos].iter().collect();
                skip(&mut pos);
                if chars.get(pos) != Some(&':') {
                    return Err(error());
                }
                pos += 1;
                skip(&mut pos);
                if chars.get(pos) != Some(&'"') {
                    return Err(error());
                }
                pos += 1;
                let start = pos;
                loop {
                    match chars.get(pos) {
                        None => return Err(error()),
                        Some('"') => break,
                        Some('\\') => {
                            pos += 1;
                            if pos == chars.len() {
                                return Err(error());
                            }
                        }
                        _ => {}
                    }
                    pos += 1;
                }
                let value: String = chars[start..pos].iter().collect();
                pos += 1;
                token.append(&key, &value);
            }
            tokens.push(token);
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_token() {
        let parser = TokenParser::new(Language::Zh, Operator::Tn);
        let input = r#"date { year: "2024" month: "1" day: "15" }"#;
        let result = parser.reorder(input).unwrap();
        // Should be ordered as year, month, day
        assert!(result.contains("year: \"2024\""));
    }

    #[test]
    fn test_empty_input() {
        let parser = TokenParser::new(Language::Zh, Operator::Tn);
        assert_eq!(parser.reorder("").unwrap(), "");
        assert_eq!(parser.reorder("   ").unwrap(), "");
    }

    #[test]
    fn v120_fields_and_wire_escapes_are_preserved() {
        let parser = TokenParser::chinese_itn_v120();
        assert_eq!(
            parser
                .reorder_strict(r#"measure { units: "千克" value: "3" }"#)
                .unwrap(),
            r#"measure { value: "3" units: "千克" }"#
        );
        let escaped = r#"char { value: "a\"b\\c" }"#;
        assert_eq!(parser.reorder_strict(escaped).unwrap(), escaped);
        assert!(parser.reorder_strict("not tagged output").is_err());
    }
}
