use reqwest::header::HeaderMap;
use serde::ser::{SerializeMap, SerializeSeq, Serializer};
use serde::Serialize;
use crate::snapshot_types::Snapshot;

#[derive(Debug)]
pub struct HttpFile {
    pub verb: HttpVerb,
    pub url: String,
    pub headers: Vec<Header>,
    pub body: Json,
    pub snapshot: Snapshot,
}

#[derive(Debug)]
pub struct SnapResponse {
    pub status: u16,
    pub headers: HeaderMap,
    pub body: Json
}

#[derive(Debug, Eq, PartialEq)]
pub enum HttpVerb {
    GET,
    POST,
    PUT,
}

#[derive(Debug)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct Json {
    pub element: Element,
}

impl Serialize for Json {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        return self.element.serialize(serializer);
    }
}

#[derive(Debug)]
pub struct Element {
    pub value: Value,
}

impl Serialize for Element {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        return self.value.serialize(serializer);
    }
}

#[derive(Debug)]
pub enum Value {
    Object(Object),
    Array(Array),
    String(String),
    Number(Number),
    Boolean(bool),
    Null(),
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Object(val) => val.serialize(serializer),
            Value::Array(val) => val.serialize(serializer),
            Value::String(val) => serializer.serialize_str(val),
            Value::Number(val) => val.serialize(serializer),
            Value::Boolean(val) => serializer.serialize_bool(val.clone()),
            Value::Null() => serializer.serialize_none(),
        }
    }
}

#[derive(Debug)]
pub struct Object {
    pub members: Vec<Member>,
}

impl Serialize for Object {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.members.len()))?;
        for member in &self.members {
            map.serialize_entry(&member.key, &member.value)?;
        }
        return map.end();
    }
}

#[derive(Debug)]
pub struct Member {
    pub key: String,
    pub value: Element,
}

#[derive(Debug)]
pub struct Array {
    pub elements: Vec<Element>,
}

impl Serialize for Array {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.elements.len()))?;
        for element in &self.elements {
            seq.serialize_element(element)?;
        }
        return seq.end();
    }
}

#[derive(Debug)]
pub enum Number {
    Int(i64),
    Fraction(f64),
    Exponent(String),
}

impl Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Number::Int(val) => serializer.serialize_i64(val.clone()),
            Number::Fraction(val) => serializer.serialize_f64(val.clone()),
            Number::Exponent(val) => serializer.serialize_f64(val.parse::<f64>().unwrap()),
        }
    }
}
