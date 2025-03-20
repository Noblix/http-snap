use crate::types::{
    Array, Comparison, Element, Header, Json, Number, Object, SnapResponse, Snapshot, Value,
};
use reqwest::header::HeaderMap;
use std::collections::HashMap;

pub fn compare_to_snapshot(snapshot: &Snapshot, response: &SnapResponse) -> bool {
    let status_matches = match_status(&snapshot.status, &response.status);
    if !status_matches {
        log::error!("Status did not match snapshot");
        log::error!(
            "Expected: {:?} but got {:?}",
            snapshot.status, response.status
        );
        return false;
    }

    if response.options.include_headers {
        let headers_match = match_headers(&snapshot.headers, &response.headers);
        if !headers_match {
            return false;
        }
    }

    let body_match = match_body(&snapshot.body, &response.body);
    if !body_match {
        log::error!("Body did not match snapshot");
        return false;
    }

    return true;
}

fn match_status(snapshot_status: &Number, response_status: &u16) -> bool {
    return match snapshot_status {
        Number::Int(value) => value == &(response_status.clone() as i64),
        _ => false,
    };
}

fn match_headers(snapshot_headers: &Vec<Header>, response_header: &HeaderMap) -> bool {
    for snapshot_header in snapshot_headers {
        if matches!(snapshot_header.comparison, Some(Comparison::Ignore)) {
            continue;
        }

        let matched_snapshot = match &snapshot_header.comparison {
            Some(Comparison::Exact) => {
                let response_value_option = response_header.get(&snapshot_header.name);
                if let Some(response_value) = response_value_option {
                    response_value.to_str().unwrap() == snapshot_header.value
                } else {
                    false
                }
            }
            _ => false,
        };

        if !matched_snapshot {
            log::error!(
                "Header named: {:?} did NOT match snapshot",
                snapshot_header.name
            );
            log::error!(
                "Expected: {:?} but got {:?}",
                snapshot_header.value,
                response_header.get(&snapshot_header.name).unwrap()
            );
            return false;
        }
    }

    if response_header.len() > snapshot_headers.len() {
        log::error!("Response contains headers not present in snapshot");
        return false;
    }

    return true;
}

fn match_body(snapshot_body: &Json, response_body: &Json) -> bool {
    return match_body_element(&snapshot_body.element, &response_body.element);
}

fn match_body_element(expected: &Element, actual: &Element) -> bool {
    return match expected.comparison {
        Some(Comparison::Ignore) => true,
        _ => match_body_value(&expected.value, &actual.value), // This is the same as exact
    };
}

fn match_body_value(expected: &Value, actual: &Value) -> bool {
    return match (expected, actual) {
        (Value::Object(expected_object), Value::Object(actual_object)) => {
            match_body_object(expected_object, actual_object)
        }
        (Value::Array(expected_array), Value::Array(actual_array)) => {
            match_body_array(expected_array, actual_array)
        }
        (Value::String(expected_string), Value::String(actual_string)) => {
            expected_string.to_string() == actual_string.to_string()
        }
        (Value::Number(expected_number), Value::Number(actual_number)) => {
            match_body_number(expected_number, actual_number)
        }
        (Value::Boolean(expected_bool), Value::Boolean(actual_bool)) => {
            expected_bool == actual_bool
        }
        (Value::Null(), Value::Null()) => true,
        _ => false,
    };
}

fn match_body_object(expected: &Object, actual: &Object) -> bool {
    if expected.members.len() != actual.members.len() {
        return false;
    }

    let actual_members = actual
        .members
        .iter()
        .map(|member| (&member.key, member))
        .collect::<HashMap<_, _>>();

    for member in &expected.members {
        let actual_member = actual_members.get(&member.key);
        if actual_member.is_none() {
            log::error!("Could not find expected member named {:?}", member.key);
            return false;
        }

        let matched_member = match_body_element(&member.value, &actual_member.unwrap().value);
        if !matched_member {
            log::error!("Member named: {:?} did NOT match snapshot", member.key);
            return false;
        }
    }

    return true;
}

fn match_body_array(expected: &Array, actual: &Array) -> bool {
    if expected.get_elements().len() != actual.get_elements().len() {
        return false;
    }

    let zipped = expected
        .get_elements()
        .into_iter()
        .zip(actual.get_elements().into_iter());
    for (expected_element, actual_element) in zipped {
        let matches_expected = match_body_element(&expected_element, &actual_element);
        if !matches_expected {
            return false;
        }
    }

    return true;
}

fn match_body_number(expected: &Number, actual: &Number) -> bool {
    return match (expected, actual) {
        (Number::Int(expected_int), Number::Int(actual_int)) => expected_int == actual_int,
        (Number::Fraction(expected_faction), Number::Fraction(actual_fraction)) => {
            expected_faction == actual_fraction
        }
        (Number::Exponent(expected_exponent), Number::Exponent(actual_exponent)) => {
            expected_exponent == actual_exponent
        }
        _ => false,
    };
}
