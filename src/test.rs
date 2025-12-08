use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use expect_test::expect;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser as MarkdownParser, Tag, TagEnd};

use crate::add_links;

#[derive(Debug, Default)]
struct TestCase {
    inputs: BTreeMap<String, String>,
    empty_input_files: Vec<String>,
    outputs: BTreeMap<String, String>,
}

fn parse_test_file(path: &Path) -> Result<TestCase> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read test file {}", path.display()))?;
    parse_test(content)
}

fn parse_test(content: String) -> std::result::Result<TestCase, anyhow::Error> {
    let options = Options::empty();
    let parser = MarkdownParser::new_ext(&content, options);
    let events: Vec<_> = parser.collect();

    let mut test_case = TestCase::default();
    let mut current_section: Option<&str> = None;
    let mut current_file: Option<String> = None;
    let mut in_code_block = false;
    let mut code_block_content = String::new();
    let mut in_list = false;
    let mut in_item = false;
    let mut current_item_text = String::new();
    let mut current_item_has_code = false;
    let mut list_items = Vec::new();

    for (i, event) in events.iter().enumerate() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                if *level == HeadingLevel::H2 {
                    if i + 1 < events.len()
                        && let Event::Text(text) = &events[i + 1]
                    {
                        let section = text.as_ref();
                        if section == "Inputs" {
                            current_section = Some("inputs");
                        } else if section == "Outputs" {
                            current_section = Some("outputs");
                        }
                    }
                } else if *level == HeadingLevel::H3 {
                    let mut heading_text = String::new();
                    let mut heading_code = None;
                    for rest in events.iter().skip(i + 1) {
                        match rest {
                            Event::Text(text) => {
                                heading_text.push_str(text.as_ref());
                            }
                            Event::Code(code) => {
                                heading_code = Some(code.as_ref().to_string());
                            }
                            Event::End(TagEnd::Heading(_)) => {
                                break;
                            }
                            _ => {}
                        }
                    }
                    if let Some(code) = heading_code {
                        current_file = Some(code);
                        in_list = false;
                    } else if heading_text.trim() == "Empty files" {
                        current_file = None;
                        in_list = true;
                        list_items.clear();
                    }
                }
            }
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
                code_block_content.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                if in_code_block {
                    if let Some(file) = &current_file {
                        let content = code_block_content.trim().to_string();
                        match current_section {
                            Some("inputs") => {
                                test_case.inputs.insert(file.clone(), content);
                            }
                            Some("outputs") => {
                                test_case.outputs.insert(file.clone(), content);
                            }
                            _ => {}
                        }
                    }
                    code_block_content.clear();
                    in_code_block = false;
                }
            }
            Event::Text(text) => {
                if in_code_block {
                    code_block_content.push_str(text.as_ref());
                } else if in_item {
                    current_item_text.push_str(text.as_ref());
                }
            }
            Event::Code(code) => {
                if in_code_block {
                    code_block_content.push_str(code.as_ref());
                } else if in_item {
                    current_item_text.push_str(code.as_ref());
                    current_item_has_code = true;
                }
            }
            Event::Start(Tag::List(_)) => {
                if current_file.is_none() {
                    in_list = true;
                    list_items.clear();
                }
            }
            Event::End(TagEnd::List(_)) => {
                if in_list {
                    if let Some("inputs") = current_section {
                        test_case.empty_input_files = list_items.clone();
                    }
                    list_items.clear();
                    in_list = false;
                }
            }
            Event::Start(Tag::Item) => {
                in_item = true;
                current_item_text.clear();
                current_item_has_code = false;
            }
            Event::End(TagEnd::Item) => {
                if in_item && in_list {
                    let text = current_item_text.trim();
                    if text.starts_with('`') && text.ends_with('`') {
                        list_items.push(text[1..text.len() - 1].to_string());
                    } else if current_item_has_code && !text.is_empty() {
                        list_items.push(text.to_string());
                    }
                    current_item_text.clear();
                    current_item_has_code = false;
                    in_item = false;
                } else {
                    in_item = false;
                }
            }
            _ => {}
        }
    }

    Ok(test_case)
}

fn test_path(test: &'static str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(test)
}

#[test]
fn test_parse_test_file() {
    let test_file = test_path("boundaries.md");
    let test_case = parse_test_file(&test_file).unwrap();
    expect![[r#"
        TestCase {
            inputs: {
                "first.md": "[linking] should work at the beginning of files",
                "last.md": "linking should work at the beginning of [files]",
                "whole.md": "[linking should work for a whole file]",
            },
            empty_input_files: [
                "linking.md",
                "files.md",
                "linking-should-work-for-a-whole-file.md",
            ],
            outputs: {
                "first.md": "[linking](linking.md) should work at the beginning of files",
                "last.md": "linking should work at the beginning of [files](files.md)",
                "whole.md": "[linking should work for a whole file](linking-should-work-for-a-whole-file.md)",
            },
        }"#]]
    .assert_eq(&format!("{test_case:#?}"));
}

fn run_test_case(test_case: &TestCase) {
    let dir = Path::new("test");
    let exists = |p: &Path| -> Result<bool> {
        let file_name = p
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;
        Ok(test_case
            .empty_input_files
            .contains(&String::from(file_name))
            || test_case.inputs.contains_key(file_name))
    };

    let mut results: HashMap<&str, String> = HashMap::new();
    for (file, content) in &test_case.inputs {
        let modified = add_links(content, dir, exists).unwrap();
        results.insert(file, modified);
    }
    for (file, expected) in &test_case.outputs {
        let actual = results
            .get(file.as_str())
            .expect("Output file not found in results");
        assert_eq!(expected, actual);
    }
}

fn test(test: &'static str) {
    let test_case = parse_test_file(&test_path(test)).unwrap();
    run_test_case(&test_case);
}

#[test]
fn test_boundaries() {
    test("boundaries.md");
}

#[test]
fn test_existing() {
    test("existing.md");
}
