use crate::types::{Element, Header};

pub trait Detector {
    fn detect_in_header(&self, header: &Header) -> Option<Header>;
    fn detect_in_element(&self, element: &Element) -> Option<Element>;
}