use crate::snapshot_types::{
    ArrayComparer, Comparison, ElementComparer, HeaderComparer, JsonComparer, Number,
    ObjectComparer, Snapshot, ValueComparer,
};
use crate::types;
use crate::types::{Array, Element, Json, Object, SnapResponse, Value};
use reqwest::header::HeaderMap;
use std::collections::HashMap;

pub fn compare_to_snapshot(snapshot: &Snapshot, response: &SnapResponse) -> bool {
    let status_matches = match_status(&snapshot.status, &response.status);
    if !status_matches {
        println!("Status did not match snapshot");
        println!(
            "Expected: {:?} but got {:?}",
            snapshot.status, response.status
        );
        return false;
    }

    if response.options.include_headers {
        let headers_match = match_headers(&snapshot.headers, &response.headers);
        if !headers_match {
            println!("Headers did not match snapshot");
            return false;
        }
    }

    let body_match = match_body(&snapshot.body, &response.body);
    if !body_match {
        println!("Body did not match snapshot");
        return false;
    }

    return true;
}

fn match_status(snapshot_status: &Comparison, response_status: &u16) -> bool {
    return match snapshot_status {
        Comparison::Ignore => true,
        Comparison::Exact(ValueComparer::Number(Number::Int(value))) => {
            value == &(response_status.clone() as i64)
        }
        _ => false,
    };
}

fn match_headers(snapshot_headers: &Vec<HeaderComparer>, response_header: &HeaderMap) -> bool {
    for snapshot_header in snapshot_headers {
        if matches!(snapshot_header.value, Comparison::Ignore) {
            println!("Ignored header called: {:?}", snapshot_header.name);
            continue;
        }

        let matched_snapshot = match &snapshot_header.value {
            Comparison::Exact(ValueComparer::String(value)) => {
                let response_value_option = response_header.get(&snapshot_header.name);
                if let Some(response_value) = response_value_option {
                    response_value.to_str().unwrap() == value
                } else {
                    false
                }
            }
            _ => false,
        };

        if !matched_snapshot {
            println!(
                "Header named: {:?} did NOT match snapshot",
                snapshot_header.name
            );
            println!(
                "Expected: {:?} but got {:?}",
                snapshot_header.value,
                response_header.get(&snapshot_header.name).unwrap()
            );
            return false;
        }
    }
    
    if response_header.len() > snapshot_headers.len() {
        println!("Response contains headers not present in snapshot");
        return false;
    }

    return true;
}

fn match_body(snapshot_body: &JsonComparer, response_body: &Json) -> bool {
    return match_body_element(&snapshot_body.element, &response_body.element);
}

fn match_body_element(expected: &ElementComparer, actual: &Element) -> bool {
    return match_body_comparison(&expected.value, &actual.value);
}

fn match_body_comparison(expected: &Comparison, actual: &Value) -> bool {
    return match expected {
        Comparison::Ignore => true,
        Comparison::Exact(expected_value) => match_body_value(expected_value, actual),
    };
}

fn match_body_value(expected: &ValueComparer, actual: &Value) -> bool {
    return match (expected, actual) {
        (ValueComparer::Object(expected_object), Value::Object(actual_object)) => {
            match_body_object(expected_object, actual_object)
        }
        (ValueComparer::Array(expected_array), Value::Array(actual_array)) => {
            match_body_array(expected_array, actual_array)
        }
        (ValueComparer::String(expected_string), Value::String(actual_string)) => {
            expected_string == actual_string
        }
        (ValueComparer::Number(expected_number), Value::Number(actual_number)) => {
            match_body_number(expected_number, actual_number)
        }
        (ValueComparer::Boolean(expected_bool), Value::Boolean(actual_bool)) => {
            expected_bool == actual_bool
        }
        (ValueComparer::Null(), Value::Null()) => true,
        _ => false,
    };
}

fn match_body_object(expected: &ObjectComparer, actual: &Object) -> bool {
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
            println!("Could not find expected member named {:?}", member.key);
            return false;
        }

        let matched_member = match_body_element(&member.value, &actual_member.unwrap().value);
        if !matched_member {
            println!("Member named: {:?} did NOT match snapshot", member.key);
            return false;
        }
    }

    return true;
}

fn match_body_array(expected: &ArrayComparer, actual: &Array) -> bool {
    if expected.elements.len() != actual.elements.len() {
        return false;
    }

    let zipped = expected.elements.iter().zip(actual.elements.iter());
    for (expected_element, actual_element) in zipped {
        let matches_expected = match_body_element(expected_element, actual_element);
        if !matches_expected {
            return false;
        }
    }

    return true;
}

fn match_body_number(expected: &Number, actual: &types::Number) -> bool {
    return match (expected, actual) {
        (Number::Int(expected_int), types::Number::Int(actual_int)) => expected_int == actual_int,
        (Number::Fraction(expected_faction), types::Number::Fraction(actual_fraction)) => {
            expected_faction == actual_fraction
        }
        (Number::Exponent(expected_exponent), types::Number::Exponent(actual_exponent)) => {
            expected_exponent == actual_exponent
        }
        _ => false,
    };
}
