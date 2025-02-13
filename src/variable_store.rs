use crate::types::SnapResponse;
use regex::Regex;
use std::collections::HashMap;

pub struct VariableStore {
    variables: HashMap<String, String>,
}

impl VariableStore {
    pub fn new() -> Self {
        return Self {
            variables: HashMap::new(),
        };
    }

    pub fn replace_variables(&mut self, text: &str, previous: &Option<SnapResponse>) -> String {
        let previous_replaced = replace_previous(text, &previous);
        let new_variables = extract_variables(&previous_replaced);
        self.variables.extend(new_variables.into_iter());
        let text_without_variables = self.replace_local_variables(&previous_replaced);
        return text_without_variables;
    }

    fn replace_local_variables(&self, text: &str) -> String {
        let mut result: String = text.to_string();
        for (name, value) in &self.variables {
            let replace = ["{{", name, "}}"].concat();
            result = result.replace(&replace, value);
        }
        return result;
    }
}

fn replace_previous(text: &str, previous: &Option<SnapResponse>) -> String {
    let mut result: String = text.to_string();

    if let Some(previous) = previous {
        let header_regex =
            Regex::new(r#"\{\{previous\.headers\["([a-zA-Z0-9\-\_]+)"\]\}\}"#).unwrap();
        let header_names = header_regex.captures_iter(text).map(|c| c.extract());
        for (_, [header_name]) in header_names {
            let header = previous.headers.get(header_name);
            if let Some(value) = header {
                let replace = ["{{previous.headers[\"", header_name, "\"]}}"].concat();
                result = result.replace(&replace, value.to_str().unwrap().trim())
            }
        }
    }

    return result;
}

fn extract_variables(text: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();

    for mut line in text.lines() {
        line = line.trim();
        if line.starts_with("@") {
            let parts: Vec<&str> = line.split("=").collect();
            if parts.len() >= 2 {
                let name = parts[0][1..].trim().to_string();
                let value = parts[1..].concat().trim().to_string();
                result.insert(name, value);
            }
        }

        let verb_options = ["GET", "DELETE", "PATCH", "POST", "PUT"];
        for prefix in verb_options {
            if line.starts_with(prefix) {
                break;
            }
        }
    }

    return result;
}