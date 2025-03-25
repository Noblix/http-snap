use crate::types::{
    Array, CompositeString, CompositeStringPart, Element, Header, HttpFile, Json, Member, Object,
    SnapResponse, Snapshot, Value,
};
use reqwest::header::HeaderMap;
use std::collections::HashMap;

pub(crate) struct VariableStore {
    pub(crate) variables: HashMap<String, Value>,
}

impl VariableStore {
    pub fn new() -> Self {
        return Self {
            variables: HashMap::new(),
        };
    }

    pub(crate) fn update_variables(&mut self, snapshot: &Snapshot, response: &SnapResponse) {
        self.extract_variables_from_headers(&snapshot.headers, &response.headers);
        self.extract_variables_from_body(&snapshot.body.element, &response.body.element);
    }

    fn extract_variables_from_headers(
        &mut self,
        snapshot_headers: &Vec<Header>,
        response_headers: &HeaderMap,
    ) {
        for header in snapshot_headers {
            if let Some(variable_name) = &header.variable_store {
                self.variables.insert(
                    variable_name.to_string(),
                    Value::String(CompositeString {
                        parts: [CompositeStringPart::Literal(
                            response_headers
                                .get(&header.name)
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string(),
                        )]
                        .to_vec(),
                    }),
                );
            }
        }
    }

    fn extract_variables_from_body(
        &mut self,
        snapshot_element: &Element,
        response_element: &Element,
    ) {
        if let Some(name) = &snapshot_element.variable_store {
            self.variables
                .insert(name.clone(), response_element.value.clone());
        }

        match (&snapshot_element.value, &response_element.value) {
            (Value::Object(snapshot_object), Value::Object(response_object)) => {
                for (index, member) in snapshot_object.members.iter().enumerate() {
                    self.extract_variables_from_body(
                        &member.value,
                        &response_object.members.get(index).unwrap().value,
                    )
                }
            }
            (Value::Array(snapshot_array), Value::Array(response_array)) => {
                for (index, element) in snapshot_array.get_elements().iter().enumerate() {
                    self.extract_variables_from_body(
                        element,
                        response_array.get_elements().get(index).unwrap(),
                    )
                }
            }
            _ => (),
        }
    }

    pub(crate) fn replace_variables(&mut self, input: HttpFile) -> HttpFile {
        self.extend_variables(&input.variables);
        let url_replaced = self.replace_in_composite_string(&input.url);
        let header_replaced = self.replace_in_headers(&input.headers);
        let body_replaced = self.replace_in_body(&input.body);
        let snapshot_replaced = self.replace_in_snapshot(input.snapshot);
        return HttpFile {
            options: input.options,
            variables: input.variables,
            verb: input.verb,
            url: url_replaced,
            headers: header_replaced,
            body: body_replaced,
            snapshot: snapshot_replaced,
        };
    }

    fn extend_variables(&mut self, new_variables: &HashMap<String, Value>) {
        for (new_var_name, new_var_value) in new_variables {
            let value = self.replace_in_value(new_var_value);
            self.variables.insert(new_var_name.clone(), value);
        }
    }
    
    fn replace_in_headers(&mut self, headers: &Vec<Header>) -> Vec<Header> {
        let mut result = Vec::new();
        for header in headers {
            result.push(Header {
                name: header.name.clone(),
                comparison: header.comparison.clone(),
                value: self.replace_in_composite_string(&header.value),
                variable_store: header.variable_store.clone(),
            });
        }
        return result;
    }

    fn replace_in_value(&self, value: &Value) -> Value {
        return match value {
            Value::VariableReference(name) => self.look_up_variable(&name),
            Value::Boolean(_) | Value::Null() | Value::Number(_) => value.clone(),
            Value::String(val) => Value::String(self.replace_in_composite_string(val)),
            Value::Array(array) => Value::Array(self.replace_in_array(array)),
            Value::Object(object) => Value::Object(self.replace_in_object(object)),
        };
    }

    fn look_up_variable(&self, name: &str) -> Value {
        if self.variables.contains_key(name) {
            return self.variables.get(name).unwrap().clone();
        }

        panic!("Variable not found!");
    }

    fn replace_in_body(&self, body: &Json) -> Json {
        return Json {
            element: Element {
                value: self.replace_in_value(&body.element.value),
                variable_store: body.element.variable_store.clone(),
                comparison: body.element.comparison.clone(),
            },
        };
    }

    fn replace_in_composite_string(&self, url: &CompositeString) -> CompositeString {
        let mut replaced_url = Vec::new();
        for part in &url.parts {
            match part {
                CompositeStringPart::Literal(_) => replaced_url.push(part.clone()),
                CompositeStringPart::VariableName(name) => {
                    let value = self.look_up_variable(name);
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

    fn replace_in_array(&self, array: &Array) -> Array {
        match &array {
            Array::VariableReference(name) => {
                let variable_value = self.look_up_variable(&name);
                match variable_value {
                    Value::Array(value) => value,
                    _ => panic!("Variable {name} is not of type array"),
                }
            }
            Array::Literal(elements) => {
                let mut replaced = Vec::new();
                for element in elements {
                    replaced.push(Element {
                        value: self.replace_in_value(&element.value),
                        variable_store: element.variable_store.clone(),
                        comparison: element.comparison.clone(),
                    });
                }
                return Array::Literal(replaced);
            }
            Array::Composite(parts) => {
                let mut replaced = Vec::new();
                for part in parts {
                    replaced.push(self.replace_in_array(&part));
                }
                return Array::Composite(replaced);
            }
        }
    }

    fn replace_in_object(&self, object: &Object) -> Object {
        let mut replaced = Vec::new();
        for member in &object.members {
            replaced.push(Member {
                key: member.key.clone(),
                value: Element {
                    value: self.replace_in_value(&member.value.value),
                    variable_store: member.value.variable_store.clone(),
                    comparison: member.value.comparison.clone(),
                },
            });
        }
        return Object { members: replaced };
    }

    fn replace_in_snapshot(&self, snapshot: Snapshot) -> Snapshot {
        let body = self.replace_in_body(&snapshot.body);

        return Snapshot {
            status: snapshot.status,
            headers: snapshot.headers,
            body,
        };
    }
}
