use crate::types::{SnapResponse, Value};
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
        let previous_headers_replaced = replace_previous_headers(text, &previous);
        let previous_body_replaced = replace_previous_body(&previous_headers_replaced, &previous);
        let new_variables = extract_variables(&previous_body_replaced);
        self.variables.extend(new_variables.into_iter());
        let text_without_variables = self.replace_local_variables(&previous_body_replaced);
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

fn replace_previous_body(text: &str, previous: &Option<SnapResponse>) -> String {
    let mut result: String = text.to_string();

    if let Some(previous) = previous {
        let body_regex = Regex::new(r#"\{\{previous\.body\.([^\n]+)\}\}"#).unwrap();
        let body_properties = body_regex.captures_iter(text).map(|c| c.extract());
        for (_, [path_to_property]) in body_properties {
            let parts: Vec<&str> = path_to_property.split(".").collect();
            let mut current_value = &previous.body.element.value;
            for part in parts {
                match current_value {
                    Value::Object(object) => {
                        current_value = if let Some(member) =
                            object.members.iter().find(|member| member.key == part)
                        {
                            &member.value.value
                        } else {
                            panic!("Unknown member")
                        }
                    }
                    _ => panic!("Unsupported field identification"),
                }
            }

            let replace = ["{{previous.body.", path_to_property, "}}"].concat();
            result = result.replace(&replace, &current_value.to_insertion_string(false));
        }
    }

    return result;
}

fn replace_previous_headers(text: &str, previous: &Option<SnapResponse>) -> String {
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
