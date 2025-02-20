use http_snap::parser::parse_file;
use indoc::indoc;

#[test]
fn get_without_snapshot() {
    let input = indoc! {r#"
        GET https://localhost:5000/today
        Accept: application/json
    "#};

    let result = parse_file(input).unwrap();

    insta::assert_debug_snapshot!(result);
}

#[test]
fn get_with_options_without_snapshot() {
    let input = indoc! {r#"
        include-headers: false

        GET https://localhost:5000/today
        Accept: application/json
    "#};

    let result = parse_file(input).unwrap();

    insta::assert_debug_snapshot!(result);
}

#[test]
fn post_without_body_and_without_snapshot() {
    let input = indoc! {r#"
        POST https://localhost:5000/items
        Accept: application/json
    "#};

    let result = parse_file(input).unwrap();

    insta::assert_debug_snapshot!(result);
}

#[test]
fn post_with_body_without_snapshot() {
    let input = indoc! {r#"
        POST https://localhost:5000/items
        Accept: application/json

        {
            "id": 5,
            "fields": {
                "text": "Hello!",
                "authors": [
                    "James Hughes",
                    "Eric McBrunch"
                ]
            }
        }
    "#};

    let result = parse_file(input).unwrap();

    insta::assert_debug_snapshot!(result);
}

#[test]
fn post_with_body_and_snapshot() {
    let input = indoc! {r#"
        POST https://localhost:5000/items
        Accept: application/json

        {
            "id": 5,
            "fields": {
                "text": "Hello!",
                "authors": [
                    "James Hughes",
                    "Eric McBrunch"
                ]
            }
        }
        
        SNAPSHOT:
        status: 201
        
        content-type: application/json; charset=utf-8
        date: Wed, 29 Jan 2025 21:08:47 GMT
        server: Kestrel
        location: _
        transfer-encoding: chunked

        {
          "id": 5
        }
    "#};

    let result = parse_file(input).unwrap();

    insta::assert_debug_snapshot!(result);
}
