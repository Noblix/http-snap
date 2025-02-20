use http_snap::parser::parse_file;
use http_snap::types::{
    Array, Element, Json, Member, Number, Object, SnapOptions, SnapResponse, Value,
};
use http_snap::variable_store::VariableStore;
use indoc::indoc;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

#[test]
fn replace_previous_header_variable() {
    let previous = SnapResponse {
        options: SnapOptions {
            include_headers: true,
        },
        status: 200,
        headers: HeaderMap::from_iter([(
            HeaderName::from_static("next-page"),
            HeaderValue::from(2),
        )]),
        body: Json {
            element: Element {
                value: Value::Null(),
            },
        },
    };
    let input = indoc! {r#"
        GET https://localhost:5000/items?page={{previous.headers["next-page"]}}
        Accept: application/json
    "#};
    let mut sut = VariableStore::new();

    let result = sut.replace_variables(input, &Some(previous));

    let parsed = parse_file(&result).unwrap();
    insta::assert_debug_snapshot!(parsed);
}

#[test]
fn replace_previous_body_variable_number() {
    let previous = SnapResponse {
        options: SnapOptions {
            include_headers: true,
        },
        status: 200,
        headers: HeaderMap::new(),
        body: Json {
            element: Element {
                value: Value::Object(Object {
                    members: vec![Member {
                        key: "id".to_string(),
                        value: Element {
                            value: Value::Number(Number::Int(123)),
                        },
                    }],
                }),
            },
        },
    };
    let input = indoc! {r#"
        GET https://localhost:5000/items/{{previous.body.id}}
        Accept: application/json
    "#};
    let mut sut = VariableStore::new();

    let result = sut.replace_variables(input, &Some(previous));

    let parsed = parse_file(&result).unwrap();
    insta::assert_debug_snapshot!(parsed);
}

#[test]
fn replace_previous_body_variable_string() {
    let previous = SnapResponse {
        options: SnapOptions {
            include_headers: true,
        },
        status: 200,
        headers: HeaderMap::new(),
        body: Json {
            element: Element {
                value: Value::Object(Object {
                    members: vec![Member {
                        key: "id".to_string(),
                        value: Element {
                            value: Value::String(
                                "97b9b76e-dff0-4400-a974-b59fde32724c".to_string(),
                            ),
                        },
                    }],
                }),
            },
        },
    };
    let input = indoc! {r#"
        GET https://localhost:5000/items/{{previous.body.id}}
        Accept: application/json
    "#};
    let mut sut = VariableStore::new();

    let result = sut.replace_variables(input, &Some(previous));

    let parsed = parse_file(&result).unwrap();
    insta::assert_debug_snapshot!(parsed);
}

#[test]
fn replace_previous_body_variable_bool() {
    let previous = SnapResponse {
        options: SnapOptions {
            include_headers: true,
        },
        status: 200,
        headers: HeaderMap::new(),
        body: Json {
            element: Element {
                value: Value::Object(Object {
                    members: vec![Member {
                        key: "is_anonymous".to_string(),
                        value: Element {
                            value: Value::Boolean(true),
                        },
                    }],
                }),
            },
        },
    };
    let input = indoc! {r#"
        GET https://localhost:5000/items
        Accept: application/json
        
        {
            "anonymous": {{previous.body.is_anonymous}}
        }
    "#};
    let mut sut = VariableStore::new();

    let result = sut.replace_variables(input, &Some(previous));

    let parsed = parse_file(&result).unwrap();
    insta::assert_debug_snapshot!(parsed);
}

#[test]
fn replace_previous_body_variable_null() {
    let previous = SnapResponse {
        options: SnapOptions {
            include_headers: true,
        },
        status: 200,
        headers: HeaderMap::new(),
        body: Json {
            element: Element {
                value: Value::Object(Object {
                    members: vec![Member {
                        key: "filters".to_string(),
                        value: Element {
                            value: Value::Null(),
                        },
                    }],
                }),
            },
        },
    };
    let input = indoc! {r#"
        GET https://localhost:5000/items
        Accept: application/json
        
        {
            "filters": {{previous.body.filters}}
        }
    "#};
    let mut sut = VariableStore::new();

    let result = sut.replace_variables(input, &Some(previous));

    let parsed = parse_file(&result).unwrap();
    insta::assert_debug_snapshot!(parsed);
}

#[test]
fn replace_previous_body_variable_object() {
    let previous = SnapResponse {
        options: SnapOptions {
            include_headers: true,
        },
        status: 200,
        headers: HeaderMap::new(),
        body: Json {
            element: Element {
                value: Value::Object(Object {
                    members: vec![Member {
                        key: "nested".to_string(),
                        value: Element {
                            value: Value::Object(Object {
                                members: vec![
                                    Member {
                                        key: "inside_string".to_string(),
                                        value: Element {
                                            value: Value::String("should be in quotes".to_string()),
                                        },
                                    },
                                    Member {
                                        key: "inside_number".to_string(),
                                        value: Element {
                                            value: Value::Number(Number::Int(82)),
                                        },
                                    },
                                ],
                            }),
                        },
                    }],
                }),
            },
        },
    };
    let input = indoc! {r#"
        GET https://localhost:5000/items
        Accept: application/json
        
        {
            "nested": {{previous.body.nested}}
        }
    "#};
    let mut sut = VariableStore::new();

    let result = sut.replace_variables(input, &Some(previous));

    let parsed = parse_file(&result).unwrap();
    insta::assert_debug_snapshot!(parsed);
}

#[test]
fn replace_previous_body_variable_array() {
    let previous = SnapResponse {
        options: SnapOptions {
            include_headers: true,
        },
        status: 200,
        headers: HeaderMap::new(),
        body: Json {
            element: Element {
                value: Value::Object(Object {
                    members: vec![Member {
                        key: "mixed_array".to_string(),
                        value: Element {
                            value: Value::Array(Array {
                                elements: vec![
                                    Element {
                                        value: Value::Object(Object {
                                            members: vec![Member {
                                                key: "inside".to_string(),
                                                value: Element {
                                                    value: Value::String(
                                                        "should be in quotes".to_string(),
                                                    ),
                                                },
                                            }],
                                        }),
                                    },
                                    Element {
                                        value: Value::String("Hello".to_string()),
                                    },
                                    Element {
                                        value: Value::Number(Number::Int(42)),
                                    },
                                ],
                            }),
                        },
                    }],
                }),
            },
        },
    };
    let input = indoc! {r#"
        GET https://localhost:5000/items
        Accept: application/json
        
        {
            "array": {{previous.body.mixed_array}}
        }
    "#};
    let mut sut = VariableStore::new();

    let result = sut.replace_variables(input, &Some(previous));

    let parsed = parse_file(&result).unwrap();
    insta::assert_debug_snapshot!(parsed);
}
