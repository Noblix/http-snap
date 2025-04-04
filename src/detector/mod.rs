use crate::types::{Array, Element, Header, Json, Member, Object, Value};
use std::collections::HashMap;

mod guid_detector;
mod timestamp_detector;

pub(crate) fn detect_in_headers(headers: HashMap<String, Header>) -> HashMap<String, Header> {
    let mut updated_headers = HashMap::new();
    for header in headers.values() {
        updated_headers.insert(header.name.clone(), detect_in_header(header));
    }
    return updated_headers;
}

fn detect_in_header(header: &Header) -> Header {
    return if let Some(detected) = guid_detector::detect_in_header(header) {
        detected
    } else {
        if let Some(detected) = timestamp_detector::detect_in_header(header) {
            detected
        } else {
            header.clone()
        }
    };
}

pub fn detect_in_json(json: Json) -> Json {
    return Json {
        element: detect_in_element(json.element),
    };
}

fn detect_in_element(element: Element) -> Element {
    let guid_element = guid_detector::detect_in_element(&element);
    return guid_element.unwrap_or_else(|| {
        timestamp_detector::detect_in_element(&element).unwrap_or_else(|| Element {
            value: detect_in_value(element.value),
            variable_store: element.variable_store,
            comparison: element.comparison,
        })
    });
}

fn detect_in_value(value: Value) -> Value {
    return match value {
        Value::Array(array) => {
            let mut elements = Vec::new();
            for element in array.get_elements() {
                elements.push(detect_in_element(element));
            }
            Value::Array(Array::Literal(elements))
        }
        Value::Object(object) => {
            let mut members = Vec::new();
            for member in object.members {
                members.push(detect_in_member(member))
            }
            Value::Object(Object { members })
        }
        _ => value,
    };
}

fn detect_in_member(member: Member) -> Member {
    return Member {
        key: member.key,
        value: detect_in_element(member.value),
    };
}
