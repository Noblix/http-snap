use crate::types::{CompositeString, CompositeStringPart, HttpFile, SnapResponse, Value};
use std::collections::HashMap;

pub struct VariableStore {
    variables: HashMap<String, Value>,
}

impl VariableStore {
    pub fn new() -> Self {
        return Self {
            variables: HashMap::new(),
        };
    }

    pub fn replace_variables(&mut self, input: HttpFile) -> HttpFile {
        self.variables.extend(input.variables.into_iter());
        let url_replaced = self.replace_in_url(input.url);
        /*        let previous_headers_replaced = replace_previous_headers(text, &previous);
                let previous_body_replaced = replace_previous_body(&previous_headers_replaced, &previous);
                let new_variables = extract_variables(&previous_body_replaced);
                self.variables.extend(new_variables.into_iter());
                let text_without_variables = self.replace_local_variables(&previous_body_replaced);
        */
        return HttpFile {
            options: input.options,
            variables: HashMap::new(),
            verb: input.verb,
            url: url_replaced,
            headers: input.headers,
            body: input.body,
            snapshot: input.snapshot,
        };
    }

    fn replace_in_url(&self, url: CompositeString) -> CompositeString {
        let mut replaced_url = Vec::new();
        for part in url.parts {
            match part {
                CompositeStringPart::Literal(_) => replaced_url.push(part),
                CompositeStringPart::VariableName(name) => {
                    if self.variables.contains_key(&name) {
                        let variable_value = self.variables.get(&name).unwrap();
                        let value_as_string = match variable_value {
                            Value::String(val) => val.clone(),
                            Value::Number(val) => val.to_string(),
                            Value::Boolean(val) => val.to_string(),
                            _ => panic!("Variable named {name} cannot be used in URL"),
                        };
                        replaced_url.push(CompositeStringPart::Literal(value_as_string))
                    } else {
                        panic!("Unknown variable named: {name}");
                    }
                }
            }
        }

        return CompositeString {
            parts: replaced_url,
        };
    }
}
