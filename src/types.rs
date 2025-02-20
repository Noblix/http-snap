use crate::snapshot_types::Snapshot;
use reqwest::header::HeaderMap;
use serde::ser::{SerializeMap, SerializeSeq, Serializer};
use serde::Serialize;

#[derive(Debug)]
pub struct HttpFile {
    pub options: SnapOptions,
    pub verb: HttpVerb,
    pub url: String,
    pub headers: Vec<Header>,
    pub body: Json,
    pub snapshot: Snapshot,
}

#[derive(Debug)]
pub struct SnapResponse {
    pub options: SnapOptions,
    pub status: u16,
    pub headers: HeaderMap,
    pub body: Json
}

#[derive(Debug, Clone)]
pub struct SnapOptions {
    pub include_headers: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum HttpVerb {
    GET,
    DELETE,
    PATCH,
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

impl Element {
    pub fn to_insertion_string(&self, nested: bool) -> String {
        return self.value.to_insertion_string(nested);
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

impl Value {
    pub fn to_insertion_string(&self, nested: bool) -> String {
        match self {
            Value::Object(val) => val.to_insertion_string(),
            Value::Array(val) => val.to_insertion_string(),
            Value::String(val) => {
                if nested {
                    ["\"", val, "\""].concat()
                } else {
                    val.clone()
                }
            }
            Value::Number(val) => val.to_insertion_string(),
            Value::Boolean(val) => val.to_string(),
            Value::Null() => "null".to_string(),
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

impl Object {
    pub fn to_insertion_string(&self) -> String {
        let mut result = "{".to_string();
        let mut member_strings = Vec::new();
        for member in &self.members {
            member_strings.push(member.to_insertion_string(true))
        }
        result += &member_strings.join("\n,");
        result += "}";
        return result;
    }
}

#[derive(Debug)]
pub struct Member {
    pub key: String,
    pub value: Element,
}

impl Member {
    pub fn to_insertion_string(&self, nested: bool) -> String {
        return [
            "\"",
            &self.key,
            "\": ",
            &self.value.to_insertion_string(nested),
        ]
        .concat();
    }
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

impl Array {
    pub fn to_insertion_string(&self) -> String {
        let mut result = "[".to_string();
        let mut element_strings = Vec::new();
        for element in &self.elements {
            element_strings.push(element.to_insertion_string(true))
        }
        result += &element_strings.join("\n,");
        result += "]";
        return result;
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

impl Number {
    pub fn to_insertion_string(&self) -> String {
        match self {
            Number::Int(val) => val.to_string(),
            Number::Fraction(val) => val.to_string(),
            Number::Exponent(val) => val.clone(),
        }
    }
}
