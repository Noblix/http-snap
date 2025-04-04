use crate::types::{Comparison, CompositeString, Element, Header, Value};
use chrono::{DateTime, NaiveDateTime};
use crate::detector::detector_trait::Detector;

pub struct TimestampDetector;

impl Detector for TimestampDetector {
    fn detect_in_header(&self, header: &Header) -> Option<Header> {
        if let Some(comparison) = try_to_match_patterns(&header.value.to_string()) {
            return Some(Header {
                name: header.name.clone(),
                value: header.value.clone(),
                variable_store: header.variable_store.clone(),
                comparison: Some(comparison),
            });
        }

        return None;
    }

    fn detect_in_element(&self, header: &Element) -> Option<Element> {
        if let Value::String(value) = &header.value {
            if let Some(comparison) = try_to_match_patterns(&value.to_string()) {
                return Some(Element {
                    value: header.value.clone(),
                    variable_store: header.variable_store.clone(),
                    comparison: Some(comparison),
                });
            }
        }

        return None;
    }
}


fn try_to_match_patterns(value: &String) -> Option<Comparison> {
    let patterns = [
        "%a, %d %b %Y %H:%M:%S %Z",
        "%m/%d/%Y %I:%M:%S %p",
        "%Y%m%dT%H%M%SZ",
        "%Y-%m-%dT%H:%M:%SZ",
    ];
    for pattern in patterns {
        if DateTime::parse_from_str(&value, &pattern).is_ok() {
            return Some(Comparison::TimestampFormat(CompositeString::from(
                String::from(pattern),
            )));
        }

        if NaiveDateTime::parse_from_str(&value, &pattern).is_ok() {
            return Some(Comparison::TimestampFormat(CompositeString::from(
                String::from(pattern),
            )));
        }
    }

    return None;
}