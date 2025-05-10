use crate::types::RawInput;
use itertools::Itertools;
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use std::ffi::OsStr;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

pub fn extract_requests(path_to_file: &PathBuf) -> Vec<RawInput> {
    let extension = path_to_file.extension().unwrap_or(OsStr::new(""));
    if extension == "http" {
        return extract_from_http_file(path_to_file);
    } else if extension == "md" {
        return extract_from_markdown(path_to_file);
    } else {
        panic!("Unknown to handle file {:?}", path_to_file);
    }
}

fn extract_from_http_file(path_to_file: &PathBuf) -> Vec<RawInput> {
    let raw_text = read_to_string(path_to_file).unwrap();
    let text = raw_text.trim_start_matches("\u{feff}");
    return extract_from_text(path_to_file, 0, text);
}

fn extract_from_markdown(path_to_file: &PathBuf) -> Vec<RawInput> {
    let raw_text = read_to_string(path_to_file).unwrap();
    let text = raw_text.trim_start_matches("\u{feff}");
    let sections = extract_http_section_from_markdown(text);

    let mut request_texts = Vec::new();
    for (index, section) in sections.iter().enumerate() {
        let mut section_requests = extract_from_text(&path_to_file, index, section);
        request_texts.append(&mut section_requests);
    }
    return request_texts;
}

fn extract_http_section_from_markdown(text: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let mut in_http = false;
    let mut content = String::new();

    for event in Parser::new(text) {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang)))
                if lang.eq_ignore_ascii_case("http") =>
            {
                in_http = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                in_http = false;
                sections.push(content.clone());
                content.clear();
            }
            Event::Text(text) if in_http => {
                content.push_str(&text);
            }
            _ => {}
        }
    }

    return sections;
}

fn extract_from_text(path_to_file: &PathBuf, section_number: usize, text: &str) -> Vec<RawInput> {
    let mut request_texts = Vec::new();
    let (files_to_import, text_without_imports) = extract_imports(&text);
    for file in files_to_import {
        let base_dir = path_to_file.parent().unwrap_or_else(|| Path::new(""));
        let full_path = base_dir.join(&file);

        let imported_requests = extract_requests(&full_path);

        for request in imported_requests {
            request_texts.push(RawInput {
                text: request.text,
                section: request.section,
                imported_path: Some(PathBuf::from(&file)),
            })
        }
    }

    for request in text_without_imports.split("###") {
        request_texts.push(RawInput {
            text: request.trim().to_string(),
            section: section_number,
            imported_path: None,
        })
    }

    return request_texts;
}

fn extract_imports(text: &str) -> (Vec<String>, String) {
    let mut imports = Vec::new();
    let mut index = 0;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(path) = trimmed.strip_prefix("import ") {
            imports.push(path.to_string());
        } else if !trimmed.is_empty() {
            // once we hit a non-import, non-blank line, stop
            break;
        }
        index += 1;
    }
    return (imports, text.lines().skip(index).join("\n"));
}
