﻿use serde::ser::{SerializeMap, SerializeSeq, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug)]
pub struct ExecuteOptions {
    pub mode: Mode,
    pub update_options: Option<UpdateOptions>,
}

impl ExecuteOptions {
    pub fn new_test() -> Self {
        return Self {
            mode: Mode::Test,
            update_options: None,
        };
    }

    pub fn new_update(
        stop_on_failure: bool,
        update_mode: UpdateMode,
        detectors: &[Detector],
    ) -> Self {
        return Self {
            mode: Mode::Update,
            update_options: Some(UpdateOptions {
                stop_on_failure,
                update_mode,
                detectors: detectors.iter().cloned().collect(),
            }),
        };
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Mode {
    Test,
    Update,
}

#[derive(Debug)]
pub struct UpdateOptions {
    pub stop_on_failure: bool,
    pub update_mode: UpdateMode,
    pub detectors: HashSet<Detector>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UpdateMode {
    Overwrite,
    Append,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Detector {
    Timestamp,
    Guid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientOptions {
    #[serde(default)]
    pub use_cookies: Option<bool>,

    #[serde(default)]
    pub default_headers: Option<Vec<DefaultHeader>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultHeader {
    pub name: String,
    pub value: String,
}

impl Default for ClientOptions {
    fn default() -> Self {
        ClientOptions {
            use_cookies: None,
            default_headers: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RawInput {
    pub text: String,
    pub section: usize,
    pub imported_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct HttpFile {
    pub variables: HashMap<String, Variable>,
    pub verb: HttpVerb,
    pub url: CompositeString,
    pub headers: Vec<Header>,
    pub body: Option<Json>,
    pub snapshots: Vec<Snapshot>,
}

#[derive(Debug)]
pub struct ExecutedRequest {
    pub raw_input: RawInput,
    pub snapshot: Option<SnapResponse>,
}

#[derive(Debug)]
pub enum Variable {
    Value(Value),
    Generator(Generator),
}

#[derive(Debug, Clone)]
pub enum Generator {
    Guid,
}

#[derive(Debug)]
pub struct Snapshot {
    pub status: Status,
    pub headers: Vec<Header>,
    pub body: Option<Json>,
}

#[derive(Debug)]
pub enum Status {
    Value(Number),
    Pattern(String),
}

#[derive(Debug, Clone)]
pub enum Comparison {
    Exact,
    Ignore,
    TimestampFormat(CompositeString),
    Guid,
}

#[derive(Debug)]
pub struct SnapResponse {
    pub status: u16,
    pub headers: HashMap<String, Header>,
    pub body: Option<Json>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum HttpVerb {
    CONNECT,
    DELETE,
    GET,
    HEAD,
    OPTIONS,
    PATCH,
    POST,
    PUT,
    TRACE,
}

#[derive(Debug, Clone)]
pub struct CompositeString {
    pub parts: Vec<CompositeStringPart>,
}

impl CompositeString {
    pub fn new(parts: Vec<CompositeStringPart>) -> Self {
        return Self { parts };
    }
}

impl From<String> for CompositeString {
    fn from(s: String) -> Self {
        CompositeString::new(vec![CompositeStringPart::Literal(s)])
    }
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

impl CompositeStringPart {
    pub(crate) fn merge_literals(parts: Vec<CompositeStringPart>) -> Vec<CompositeStringPart> {
        let mut merged = Vec::new();
        for part in parts {
            match part {
                CompositeStringPart::Literal(s) => {
                    if let Some(CompositeStringPart::Literal(last)) = merged.last_mut() {
                        last.push_str(&s);
                    } else {
                        merged.push(CompositeStringPart::Literal(s));
                    }
                }
                other => merged.push(other),
            }
        }
        return merged;
    }
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

#[derive(Debug, Clone)]
pub struct Header {
    pub name: String,
    pub value: CompositeString,
    pub variable_store: Option<String>,
    pub comparison: Option<Comparison>,
}

#[derive(Debug, Clone)]
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
    pub comparison: Option<Comparison>,
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

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(CompositeString::from(s))
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
pub enum Array {
    VariableReference(String),
    Literal(Vec<Element>),
    StartsWith(Vec<Element>),
    Contains(Vec<Element>),
    EndsWith(Vec<Element>),
}

impl Array {
    pub(crate) fn get_known_elements(&self) -> Vec<Element> {
        return match self {
            Array::Literal(elements) => elements.clone(),
            Array::StartsWith(elements) => elements.clone(),
            Array::Contains(elements) => elements.clone(),
            Array::EndsWith(elements) => elements.clone(),
            Array::VariableReference(name) => {
                panic!("Variable named {name} has not been replaced yet")
            }
        };
    }
}

impl Serialize for Array {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.get_known_elements().len()))?;
        for element in &self.get_known_elements() {
            seq.serialize_element(element)?;
        }
        return seq.end();
    }
}

#[derive(Debug, Clone)]
pub enum Number {
    Int(i64),
    Fraction(String),
    Exponent(String),
}

impl Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Number::Int(val) => serializer.serialize_i64(val.clone()),
            Number::Fraction(val) => {
                let raw = RawValue::from_string(val.into()).map_err(serde::ser::Error::custom)?;
                raw.serialize(serializer)
            }
            Number::Exponent(val) => {
                let raw = RawValue::from_string(val.into()).map_err(serde::ser::Error::custom)?;
                raw.serialize(serializer)
            }
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Number::Int(val) => val.to_string(),
            Number::Fraction(val) => val.clone(),
            Number::Exponent(val) => val.clone(),
        };
        write!(f, "{}", str)
    }
}
