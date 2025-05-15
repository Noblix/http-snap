use crate::types::{
    Array, Comparison, CompositeString, Element, Header, Json, Number, Object, SnapResponse,
    Snapshot, Status, Value,
};
use chrono::{DateTime, NaiveDateTime};
use std::collections::HashMap;
use uuid::Uuid;

pub fn compare_to_snapshot(snapshot: &Snapshot, response: &SnapResponse) -> bool {
    let status_matches = match_status(&snapshot.status, &response.status);
    if !status_matches {
        log::error!("Status did not match snapshot");
        log::error!(
            "Expected: {:?} but got {:?}",
            snapshot.status,
            response.status
        );
        return false;
    }

    let headers_match = match_headers(&snapshot.headers, &response.headers);
    if !headers_match {
        return false;
    }

    let body_match = match_body(&snapshot.body, &response.body);
    if !body_match {
        log::error!("Body did not match snapshot");
        return false;
    }

    return true;
}

fn match_status(snapshot_status: &Status, response_status: &u16) -> bool {
    return match snapshot_status {
        Status::Value(Number::Int(value)) => value == &(response_status.clone() as i64),
        Status::Pattern(pattern) => {
            let code = response_status.to_string();
            for (index, number) in code.chars().enumerate() {
                let pattern_number = pattern.chars().nth(index).unwrap();
                if pattern_number != 'x' && pattern_number != 'X' {
                    if pattern_number != number {
                        return false;
                    }
                }
            }
            true
        }
        _ => false,
    };
}

fn match_headers(
    snapshot_headers: &Vec<Header>,
    response_header: &HashMap<String, Header>,
) -> bool {
    for snapshot_header in snapshot_headers {
        if matches!(snapshot_header.comparison, Some(Comparison::Ignore)) {
            continue;
        }

        let matched_snapshot = match &snapshot_header.comparison {
            Some(Comparison::Exact) => {
                let response_value_option = response_header.get(&snapshot_header.name);
                if let Some(response_value) = response_value_option {
                    response_value.value.to_string() == snapshot_header.value.to_string()
                } else {
                    false
                }
            }
            Some(Comparison::TimestampFormat(pattern)) => {
                let response_value_option = response_header.get(&snapshot_header.name);
                if let Some(response_value) = response_value_option {
                    let value = response_value.value.to_string();
                    compare_timestamp_format(pattern, &value)
                } else {
                    false
                }
            }
            Some(Comparison::Guid) => {
                let response_value_option = response_header.get(&snapshot_header.name);
                if let Some(response_value) = response_value_option {
                    let value = response_value.value.to_string();
                    compare_guid_format(&value)
                } else {
                    false
                }
            }
            _ => false,
        };

        if !matched_snapshot {
            log_header_mismatch(snapshot_header, response_header.get(&snapshot_header.name));
            return false;
        }
    }

    if response_header.len() > snapshot_headers.len() {
        log::error!("Response contains headers not present in snapshot");
        return false;
    }

    return true;
}

fn match_body(snapshot_body: &Option<Json>, response_body: &Option<Json>) -> bool {
    return match (snapshot_body, response_body) {
        (None, None) => true,
        (Some(snapshot), Some(response)) => {
            match_body_element(&snapshot.element, &response.element)
        }
        _ => false,
    };
}

fn match_body_element(expected: &Element, actual: &Element) -> bool {
    return match &expected.comparison {
        Some(Comparison::Ignore) => true,
        Some(Comparison::TimestampFormat(pattern)) => match_body_timestamp(&pattern, &actual.value),
        Some(Comparison::Guid) => match_body_guid(&actual.value),
        _ => match_body_value(&expected.value, &actual.value), // This is the same as exact
    };
}

fn match_body_timestamp(pattern: &CompositeString, actual: &Value) -> bool {
    return match actual {
        Value::String(actual_string) => {
            compare_timestamp_format(pattern, &actual_string.to_string())
        }
        _ => {
            log::error!("Value {:?} is not a string", actual);
            false
        }
    };
}

fn match_body_guid(actual: &Value) -> bool {
    return match actual {
        Value::String(actual_string) => compare_guid_format(&actual_string.to_string()),
        _ => {
            log::error!("Value {:?} is not a string", actual);
            false
        }
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

fn log_header_mismatch(snapshot_header: &Header, response_header: Option<&Header>) {
    log::error!(
        "Header named: {:?} did NOT match snapshot",
        snapshot_header.name
    );

    let response_value = response_header
        .map(|header| header.value.to_string())
        .unwrap_or_default();

    match &snapshot_header.comparison {
        Some(Comparison::Exact) => {
            log::error!(
                "Expected: {:?} but got {:?}",
                snapshot_header.value.to_string(),
                response_value
            );
        }
        Some(Comparison::TimestampFormat(pattern)) => {
            log::error!(
                "Timestamp {:?} does not match pattern {}",
                response_value,
                pattern.to_string()
            );
        }
        Some(Comparison::Guid) => {
            log::error!("Expected a guid but got {:?}", response_value);
        }
        _ => panic!(
            "Comparison type {:?} not supported for headers",
            &snapshot_header.comparison
        ),
    }
}

fn compare_timestamp_format(pattern: &CompositeString, value: &str) -> bool {
    let pattern = pattern.to_string();
    return if DateTime::parse_from_str(&value, &pattern).is_ok() {
        true
    } else {
        NaiveDateTime::parse_from_str(&value, &pattern).is_ok()
    };
}

fn compare_guid_format(value: &str) -> bool {
    return Uuid::try_parse(value).is_ok();
}
