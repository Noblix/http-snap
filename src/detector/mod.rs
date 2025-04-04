use crate::types::{
    Array, Detector, Element, Header, Json, Member, Object, SnapResponse, UpdateOptions, Value,
};
use std::collections::{HashMap};

mod detector_trait;
mod guid_detector;
mod timestamp_detector;

pub struct Replacer {
    pub detectors: Vec<Box<dyn detector_trait::Detector>>
}

impl Replacer {
    pub(crate) fn new(options: &Option<UpdateOptions>) -> Self {
        return if let Some(UpdateOptions { detectors, .. }) = options {
            let mut selected_detectors: Vec<Box<dyn detector_trait::Detector>> = Vec::new();
            if detectors.contains(&Detector::Timestamp) {
                selected_detectors.push(Box::new(timestamp_detector::TimestampDetector));
            }
            if detectors.contains(&Detector::Guid) {
                selected_detectors.push(Box::new(guid_detector::GuidDetector));
            }
            Self {
                detectors: selected_detectors,
            }
        } else {
            Self { detectors: Vec::new() }
        };
    }

    pub(crate) fn detect_types(&self, response: SnapResponse) -> SnapResponse {
        if self.detectors.len() == 0 {
            return response;
        }

        return SnapResponse {
            options: response.options,
            status: response.status,
            headers: self.detect_in_headers(response.headers),
            body: self.detect_in_json(response.body),
        };
    }

    fn detect_in_headers(&self, headers: HashMap<String, Header>) -> HashMap<String, Header> {
        let mut updated_headers = HashMap::new();
        for header in headers.values() {
            updated_headers.insert(header.name.clone(), self.detect_in_header(header));
        }
        return updated_headers;
    }

    fn detect_in_header(&self, header: &Header) -> Header {
        for detector in &self.detectors {
            if let Some(detected) = detector.detect_in_header(header) {
                return detected;
            }
        }

        return header.clone();
    }

    fn detect_in_json(&self, json: Json) -> Json {
        return Json {
            element: self.detect_in_element(json.element),
        };
    }

    fn detect_in_element(&self, element: Element) -> Element {
        for detector in &self.detectors {
            if let Some(detected) = detector.detect_in_element(&element) {
                return detected;
            }
        }

        return Element {
            value: self.detect_in_value(element.value),
            variable_store: element.variable_store,
            comparison: element.comparison,
        };
    }

    fn detect_in_value(&self, value: Value) -> Value {
        return match value {
            Value::Array(array) => {
                let mut elements = Vec::new();
                for element in array.get_elements() {
                    elements.push(self.detect_in_element(element));
                }
                Value::Array(Array::Literal(elements))
            }
            Value::Object(object) => {
                let mut members = Vec::new();
                for member in object.members {
                    members.push(self.detect_in_member(member))
                }
                Value::Object(Object { members })
            }
            _ => value,
        };
    }

    fn detect_in_member(&self, member: Member) -> Member {
        return Member {
            key: member.key,
            value: self.detect_in_element(member.value),
        };
    }
}



