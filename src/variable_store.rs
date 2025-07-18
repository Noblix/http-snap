﻿use crate::types::{
    Array, CompositeString, CompositeStringPart, Element, Header, HttpFile, Json, Member, Object,
    SnapResponse, Snapshot, Value, Variable,
};
use crate::variable_generator;
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
        if let Some(snapshot_body) = &snapshot.body {
            if let Some(response_body) = &response.body {
                self.extract_variables_from_body(&snapshot_body.element, &response_body.element);
            }
        }
    }

    fn extract_variables_from_headers(
        &mut self,
        snapshot_headers: &Vec<Header>,
        response_headers: &HashMap<String, Header>,
    ) {
        for header in snapshot_headers {
            if let Some(variable_name) = &header.variable_store {
                self.variables.insert(
                    variable_name.to_string(),
                    Value::String(response_headers.get(&header.name).unwrap().value.clone()),
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
                for (index, element) in snapshot_array.get_known_elements().iter().enumerate() {
                    self.extract_variables_from_body(
                        element,
                        response_array.get_known_elements().get(index).unwrap(),
                    )
                }
            }
            _ => (),
        }
    }

    pub(crate) fn replace_variables(&mut self, input: HttpFile) -> HttpFile {
        let variables = variable_generator::generate_variables(input.variables);
        self.extend_variables(&variables);
        let url_replaced = self.replace_in_composite_string(&input.url);
        let header_replaced = self.replace_in_headers(&input.headers);
        let body_replaced = self.replace_in_body(&input.body);
        let snapshot_replaced = self.replace_in_snapshots(input.snapshots);
        return HttpFile {
            variables: variables
                .into_iter()
                .map(|(k, v)| (k, Variable::Value(v)))
                .collect(),
            verb: input.verb,
            url: url_replaced,
            headers: header_replaced,
            body: body_replaced,
            snapshots: snapshot_replaced,
        };
    }

    pub(crate) fn extend_variables(&mut self, new_variables: &HashMap<String, Value>) {
        for (new_var_name, new_var_value) in new_variables {
            let value = self.replace_in_value(new_var_value);
            self.variables.insert(new_var_name.clone(), value);
        }
    }

    fn replace_in_headers(&self, headers: &Vec<Header>) -> Vec<Header> {
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

        log::error!("Variable named \"{}\" was not found!", name);
        panic!("Variable named \"{}\" was not found!", name);
    }

    fn replace_in_body(&self, body: &Option<Json>) -> Option<Json> {
        if let Some(json) = body {
            return Some(Json {
                element: Element {
                    value: self.replace_in_value(&json.element.value),
                    variable_store: json.element.variable_store.clone(),
                    comparison: json.element.comparison.clone(),
                },
            });
        }

        return None;
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
                return Array::Literal(self.replace_in_elements(elements));
            }
            Array::StartsWith(elements) => {
                return Array::StartsWith(self.replace_in_elements(elements));
            }
            Array::Contains(elements) => {
                return Array::Contains(self.replace_in_elements(elements));
            }
            Array::EndsWith(elements) => {
                return Array::EndsWith(self.replace_in_elements(elements));
            }
        }
    }
    
    fn replace_in_elements(&self, elements: &Vec<Element>) -> Vec<Element> {
        let mut replaced = Vec::new();
        for element in elements {
            replaced.push(Element {
                value: self.replace_in_value(&element.value),
                variable_store: element.variable_store.clone(),
                comparison: element.comparison.clone(),
            });
        }
        return replaced;
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

    fn replace_in_snapshots(&self, snapshots: Vec<Snapshot>) -> Vec<Snapshot> {
        let mut result = Vec::new();
        for snapshot in snapshots {
            let headers = self.replace_in_headers(&snapshot.headers);
            let body = self.replace_in_body(&snapshot.body);
            result.push(Snapshot {
                status: snapshot.status,
                headers,
                body,
            });
        }
        return result;
    }
}
