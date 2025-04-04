use crate::types::{Comparison, Element, Header, Value};
use uuid::Uuid;
use crate::detector::detector_trait::Detector;

pub struct GuidDetector;
impl Detector for GuidDetector {
    fn detect_in_header(&self, header: &Header) -> Option<Header> {
        if Uuid::try_parse(&header.value.to_string()).is_ok() {
            return Some(Header {
                name: header.name.clone(),
                value: header.value.clone(),
                variable_store: header.variable_store.clone(),
                comparison: Some(Comparison::Guid),
            });
        }

        return None;
    }

    fn detect_in_element(&self, element: &Element) -> Option<Element> {
        if let Value::String(value) = &element.value {
            if Uuid::try_parse(&value.to_string()).is_ok() {
                return Some(Element {
                    value: element.value.clone(),
                    variable_store: element.variable_store.clone(),
                    comparison: Some(Comparison::Guid),
                });
            }
        }

        return None;
    }
}