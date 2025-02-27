use crate::types::CompositeString;

#[derive(Debug)]
pub struct Snapshot {
    pub status: Comparison,
    pub headers: Vec<HeaderComparer>,
    pub body: JsonComparer
}

#[derive(Debug)]
pub struct HeaderComparer {
    pub name: String,
    pub value: Comparison,
}

#[derive(Debug)]
pub struct JsonComparer {
    pub element: ElementComparer,
}

#[derive(Debug)]
pub enum Comparison {
    Exact(ValueComparer),
    Ignore
}

#[derive(Debug)]
pub struct ElementComparer {
    pub value: Comparison,
}

#[derive(Debug)]
pub enum ValueComparer {
    Object(ObjectComparer),
    Array(ArrayComparer),
    String(CompositeString),
    Number(Number),
    Boolean(bool),
    Null(),
}

#[derive(Debug)]
pub struct ObjectComparer {
    pub members: Vec<MemberComparer>,
}

#[derive(Debug)]
pub struct MemberComparer {
    pub key: String,
    pub value: ElementComparer,
}

#[derive(Debug)]
pub struct ArrayComparer {
    pub elements: Vec<ElementComparer>,
}

#[derive(Debug)]
pub enum Number {
    Int(i64),
    Fraction(f64),
    Exponent(String),
}
