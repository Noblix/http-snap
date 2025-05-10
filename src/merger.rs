use crate::types::{
    Array, Comparison, Element, Header, Json, Object, RawInput, SnapResponse, UpdateMode, Value,
};
use itertools::Itertools;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn merge_snapshots_into_files(
    path_to_file: &PathBuf,
    request_texts: &Vec<RawInput>,
    mismatched_responses: Vec<(usize, SnapResponse)>,
    update_mode: &UpdateMode,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut imports = Vec::new();
    let mut merges = Vec::new();
    let mut mismatch_counter = 0;
    for (index, input) in request_texts.iter().enumerate() {
        if let Some(imported_path) = &input.imported_path {
            imports.push(format!("import {}", imported_path.display()));
            continue;
        }
        if mismatch_counter < mismatched_responses.len() {
            let (mismatch_index, mismatch_response) = &mismatched_responses[mismatch_counter];
            if *mismatch_index == index {
                let updated_mismatch =
                    create_content_with_snapshot(&input.text, mismatch_response, update_mode);
                merges.push(updated_mismatch);
                mismatch_counter += 1;
                continue;
            }
        }
        merges.push(input.text.to_string())
    }

    let imported = imports.join("\n");
    let merged = merges.join("\n\n###\n\n");

    let mut file = File::options()
        .write(true)
        .truncate(true)
        .open(path_to_file)?;

    if !imported.is_empty() {
        file.write_all(imported.as_bytes())
            .expect("Unable to write imports");
        file.write("\n\n".as_bytes()).unwrap();
    }

    file.write_all(&merged.as_bytes())
        .expect("Unable to write snapshot");
    file.flush()?;

    return Ok(());
}

pub fn create_content_with_snapshot(
    raw_text: &str,
    response: &SnapResponse,
    update_mode: &UpdateMode,
) -> String {
    let parts_of_file: Vec<&str> = raw_text.split("SNAPSHOT").collect();
    let snapshot = format_snapshot(response);
    if parts_of_file.len() == 1 {
        return raw_text.trim().to_owned() + "\n\nSNAPSHOT\n" + &snapshot;
    }
    if parts_of_file.len() == 2 {
        if update_mode == &UpdateMode::Overwrite {
            return parts_of_file[0].trim().to_owned() + "\n\nSNAPSHOT\n" + &snapshot;
        } else {
            return raw_text.trim().to_owned() + "\n||\n" + &snapshot;
        }
    }
    panic!("Found more than one snapshot place");
}

fn format_snapshot(response: &SnapResponse) -> String {
    let mut formatted = "status: ".to_owned() + &response.status.to_string();
    formatted += "\n\n";

    for name in response.headers.keys().sorted() {
        let header = response.headers.get(name).unwrap();
        formatted += &format_header(header);
        formatted += "\n";
    }

    formatted += "\n";
    formatted += &format_body(&response.body);

    return formatted;
}

fn format_header(header: &Header) -> String {
    let formatted = format_comparison(&header.comparison, &Value::from(header.value.to_string()))
        .unwrap_or_else(|| header.value.to_string());
    return format!("{}: {}", header.name, formatted);
}

fn format_body(body: &Option<Json>) -> String {
    return match body { 
        Some(json) => format_element(&json.element, 0),
        None => String::new()
    };
}

fn format_element(element: &Element, indent: usize) -> String {
    return format_comparison(&element.comparison, &element.value)
        .unwrap_or_else(|| format_value(&element.value, indent));
}

fn format_comparison(comparison: &Option<Comparison>, value: &Value) -> Option<String> {
    return match comparison {
        Some(Comparison::Ignore) => Some(format!("{{{{_:_}}}}")),
        Some(Comparison::TimestampFormat(pattern)) => Some(format!(
            "{{{{_:timestamp(\"{pattern}\"):{}}}}}",
            format_value(value, 0)
        )),
        Some(Comparison::Guid) => Some(format!("{{{{_:guid:{}}}}}", format_value(value, 0))),
        _ => None,
    };
}

fn format_value(value: &Value, indent: usize) -> String {
    match value {
        Value::VariableReference(name) => format!("{{{{{name}}}}}"), // Gives {{name}}
        Value::Object(object) => format_object(object, indent),
        Value::Array(array) => format_array(array, indent),
        Value::String(composite) => serde_json::to_string_pretty(&composite.to_string()).unwrap(),
        Value::Number(number) => number.to_string(),
        Value::Boolean(boolean) => boolean.to_string(),
        Value::Null() => String::from("null"),
    }
}

fn format_object(object: &Object, indent: usize) -> String {
    if object.members.is_empty() {
        return String::from("{}");
    }

    let indent_str = "  ".repeat(indent + 1);
    let members = object
        .members
        .iter()
        .map(|member| {
            let value = format_element(&member.value, indent + 1);
            format!("{}\"{}\": {}", indent_str, member.key, value)
        })
        .join(",\n");

    let closing_indent = "  ".repeat(indent);
    return format!("{{\n{members}\n{closing_indent}}}");
}

fn format_array(array: &Array, indent: usize) -> String {
    let elements = array.get_elements();
    if elements.len() == 0 {
        return String::from("[]");
    }

    let indent_str = "  ".repeat(indent + 1);
    let formatted_elements = elements
        .iter()
        .map(|element| format!("{}{}", indent_str, format_element(element, indent + 1)))
        .join(",\n");

    let closing_indent = "  ".repeat(indent);
    return format!("[\n{}\n{}]", formatted_elements, closing_indent);
}
