use reqwest::header::HeaderMap;
use serde::ser::{SerializeMap, SerializeSeq, Serializer};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct HttpFile {
    pub options: SnapOptions,
    pub variables: HashMap<String, Value>,
    pub verb: HttpVerb,
    pub url: CompositeString,
    pub headers: Vec<Header>,
    pub body: Json,
    pub snapshot: Snapshot,
}

#[derive(Debug)]
pub struct Snapshot {
    pub status: Number,
    pub headers: Vec<Header>,
    pub body: Json
}

#[derive(Debug, Clone)]
pub enum Comparison {
    Exact,
    Ignore
}

#[derive(Debug)]
pub struct SnapResponse {
    pub options: SnapOptions,
    pub status: u16,
    pub headers: HeaderMap,
    pub body: Json,
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

#[derive(Debug, Clone)]
pub struct CompositeString {
    pub parts: Vec<CompositeStringPart>,
}

impl Display for CompositeString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = self
            .parts
            .iter()
            .map(|part| part.to_string())
            .collect::<Vec<String>>()
            .join("");
        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone)]
pub enum CompositeStringPart {
    Literal(String),
    VariableName(String),
}

impl Display for CompositeStringPart {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            CompositeStringPart::Literal(val) => val,
            CompositeStringPart::VariableName(name) => &["{{", name, "}}"].concat(),
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug)]
pub struct Header {
    pub name: String,
    pub value: String,
    pub variable_store: Option<String>,
    pub comparison: Option<Comparison>
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

#[derive(Debug, Clone)]
pub struct Element {
    pub value: Value,
    pub variable_store: Option<String>,
    pub comparison: Option<Comparison>
}

impl Serialize for Element {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        return self.value.serialize(serializer);
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    VariableReference(String),
    Object(Object),
    Array(Array),
    String(CompositeString),
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
            Value::VariableReference(name) => panic!("Variable name {name} is unknown"),
            Value::Object(val) => val.serialize(serializer),
            Value::Array(val) => val.serialize(serializer),
            Value::String(val) => serializer.serialize_str(&val.to_string()),
            Value::Number(val) => val.serialize(serializer),
            Value::Boolean(val) => serializer.serialize_bool(val.clone()),
            Value::Null() => serializer.serialize_none(),
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Member {
    pub key: String,
    pub value: Element,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Number::Int(val) => val.to_string(),
            Number::Fraction(val) => val.to_string(),
            Number::Exponent(val) => val.clone(),
        };
        write!(f, "{}", str)
    }
}
