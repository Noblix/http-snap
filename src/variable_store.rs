use crate::types::{
    Array, CompositeString, CompositeStringPart, Element, HttpFile, Member, Number, Object,
    SnapResponse, Value,
};
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

    pub fn replace_variables(
        &mut self,
        input: HttpFile,
        previous: &Option<SnapResponse>,
    ) -> HttpFile {
        self.extend_variables(&input.variables, previous);
        let url_replaced = self.replace_in_composite_string(&input.url, previous);
        /*        let previous_headers_replaced = replace_previous_headers(text, &previous);
                let previous_body_replaced = replace_previous_body(&previous_headers_replaced, &previous);
                let new_variables = extract_variables(&previous_body_replaced);
                self.variables.extend(new_variables.into_iter());
                let text_without_variables = self.replace_local_variables(&previous_body_replaced);
        */
        return HttpFile {
            options: input.options,
            variables: input.variables,
            verb: input.verb,
            url: url_replaced,
            headers: input.headers,
            body: input.body,
            snapshot: input.snapshot,
        };
    }

    fn replace_in_composite_string(
        &self,
        url: &CompositeString,
        previous: &Option<SnapResponse>,
    ) -> CompositeString {
        let mut replaced_url = Vec::new();
        for part in &url.parts {
            match part {
                CompositeStringPart::Literal(_) => replaced_url.push(part.clone()),
                CompositeStringPart::VariableName(name) => {
                    let value = self.look_up_variable(name, previous);
                    let value_as_string = match value {
                        Value::String(val) => val.to_string(),
                        Value::Number(val) => val.to_string(),
                        Value::Boolean(val) => val.to_string(),
                        _ => panic!("Variable named {name} cannot be used in string"),
                    };
                    replaced_url.push(CompositeStringPart::Literal(value_as_string))
                }
            }
        }

        return CompositeString {
            parts: replaced_url,
        };
    }

    fn extend_variables(
        &mut self,
        new_variables: &HashMap<String, Value>,
        previous: &Option<SnapResponse>,
    ) {
        for (new_var_name, new_var_value) in new_variables {
            let value = self.replace_in_value(new_var_value, previous);
            self.variables.insert(new_var_name.clone(), value);
        }
    }

    fn replace_in_value(&self, value: &Value, previous: &Option<SnapResponse>) -> Value {
        return match value {
            Value::Boolean(_) | Value::Null() | Value::Number(_) => value.clone(),
            Value::String(val) => Value::String(self.replace_in_composite_string(val, previous)),
            Value::VariableReference(name) => self.look_up_variable(name, previous),
            Value::Array(array) => self.replace_in_array(array, previous),
            Value::Object(object) => self.replace_in_object(object, previous),
        };
    }

    fn look_up_variable(&self, name: &str, previous: &Option<SnapResponse>) -> Value {
        if self.variables.contains_key(name) {
            return self.variables.get(name).unwrap().clone();
        }

        if let Some(previous) = previous {
            let previous_header_match = "previous.headers[\"";
            if name.starts_with(previous_header_match) {
                let header_name = name
                    .strip_prefix(previous_header_match)
                    .unwrap()
                    .strip_suffix("\"]")
                    .unwrap();
                let header_value = previous.headers.get(header_name).unwrap().to_str().unwrap();
                if let Ok(number) = header_value.parse::<i64>() {
                    return Value::Number(Number::Int(number));
                }
                if let Ok(number) = header_value.parse::<f64>() {
                    return Value::Number(Number::Fraction(number));
                }
                return Value::String(CompositeString {
                    parts: [CompositeStringPart::Literal(header_value.to_string())].to_vec(),
                });
            }
        }

        panic!("Variable not found!");
    }

    fn replace_in_array(&self, array: &Array, previous: &Option<SnapResponse>) -> Value {
        let mut replaced = Vec::new();
        for element in &array.elements {
            replaced.push(Element {
                value: self.replace_in_value(&element.value, previous),
            });
        }
        return Value::Array(Array { elements: replaced });
    }

    fn replace_in_object(&self, object: &Object, previous: &Option<SnapResponse>) -> Value {
        let mut replaced = Vec::new();
        for member in &object.members {
            replaced.push(Member {
                key: member.key.clone(),
                value: Element {
                    value: self.replace_in_value(&member.value.value, previous),
                },
            });
        }
        return Value::Object(Object { members: replaced });
    }
}
