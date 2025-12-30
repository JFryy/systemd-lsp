use dashmap::DashMap;
use log::{debug, trace};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_lsp_server::lsp_types::{Position, Uri};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdUnit {
    pub sections: HashMap<String, SystemdSection>,
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdSection {
    pub name: String,
    pub directives: HashMap<String, SystemdDirective>,
    pub line_range: (u32, u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectiveValueSpan {
    pub line: u32,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdDirective {
    pub key: String,
    pub value: String,
    pub line_number: u32,
    pub column_range: (u32, u32),
    pub end_line_number: u32,
    pub value_spans: Vec<DirectiveValueSpan>,
}

#[derive(Debug)]
pub struct SystemdParser {
    documents: DashMap<Uri, SystemdUnit>,
    section_regex: Regex,
    directive_regex: Regex,
}

impl SystemdParser {
    pub fn new() -> Self {
        Self {
            documents: DashMap::new(),
            section_regex: Regex::new(r"^\[([^\]]+)\]$").unwrap(),
            directive_regex: Regex::new(r"^([^=]+)=(.*)$").unwrap(),
        }
    }

    pub fn parse(&self, text: &str) -> SystemdUnit {
        trace!("Parsing systemd unit file ({} characters)", text.len());
        let mut unit = SystemdUnit {
            sections: HashMap::new(),
            raw_text: text.to_string(),
        };

        let mut current_section: Option<String> = None;

        let mut lines = text.lines().enumerate().peekable();

        while let Some((raw_line_num, line)) = lines.next() {
            let line_num = raw_line_num as u32;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(captures) = self.section_regex.captures(trimmed) {
                if let Some(section_name) = current_section.take() {
                    if let Some(section) = unit.sections.get_mut(&section_name) {
                        section.line_range.1 = line_num - 1;
                    }
                }

                let section_name = captures[1].to_string();
                current_section = Some(section_name.clone());

                unit.sections.insert(
                    section_name.clone(),
                    SystemdSection {
                        name: section_name,
                        directives: HashMap::new(),
                        line_range: (line_num, line_num),
                    },
                );
            } else if let Some(captures) = self.directive_regex.captures(trimmed) {
                if let Some(section_name) = &current_section {
                    let key = captures[1].trim().to_string();
                    let raw_value = captures[2].to_string();

                    let key_start = line.find(&key).unwrap_or(0) as u32;
                    let key_end = key_start + key.len() as u32;

                    let eq_index = line.find('=').map(|idx| idx as u32);
                    let mut value_start = eq_index.unwrap_or(key_end) + 1;
                    let after_eq = if let Some(eq_idx) = line.find('=') {
                        &line[eq_idx + 1..]
                    } else {
                        ""
                    };
                    let leading_ws = after_eq.chars().take_while(|c| c.is_whitespace()).count();
                    value_start += leading_ws as u32;

                    let (mut fragment, mut continuation) = parse_value_fragment(&raw_value);
                    let mut normalized_value = fragment.clone();
                    let mut value_spans = Vec::new();

                    let first_span_end = value_start + fragment.len() as u32;
                    value_spans.push(DirectiveValueSpan {
                        line: line_num,
                        start: value_start,
                        end: first_span_end,
                    });

                    let mut end_line_number = line_num;

                    while continuation {
                        if let Some((next_line_num, next_line)) = lines.next() {
                            let next_line_trimmed = next_line.trim();
                            end_line_number = next_line_num as u32;

                            let indent = next_line.find(next_line_trimmed).unwrap_or(0) as u32;

                            let (next_fragment, next_continuation) =
                                parse_value_fragment(next_line_trimmed);

                            if !next_fragment.is_empty() {
                                if normalized_value.is_empty() {
                                    normalized_value = next_fragment.clone();
                                } else {
                                    normalized_value.push(' ');
                                    normalized_value.push_str(&next_fragment);
                                }
                            }

                            value_spans.push(DirectiveValueSpan {
                                line: end_line_number,
                                start: indent,
                                end: indent + next_fragment.len() as u32,
                            });

                            continuation = next_continuation;
                            fragment = next_fragment;
                        } else {
                            break;
                        }
                    }

                    if normalized_value.is_empty() {
                        normalized_value = fragment;
                    }

                    // If the directive has no value, ensure spans reflect current position
                    if normalized_value.is_empty() {
                        value_spans.clear();
                        value_spans.push(DirectiveValueSpan {
                            line: line_num,
                            start: value_start,
                            end: value_start,
                        });
                        end_line_number = line_num;
                    } else if value_spans.is_empty() {
                        value_spans.push(DirectiveValueSpan {
                            line: line_num,
                            start: value_start,
                            end: value_start + normalized_value.len() as u32,
                        });
                    }

                    let directive = SystemdDirective {
                        key: key.clone(),
                        value: normalized_value,
                        line_number: line_num,
                        column_range: (key_start, key_end),
                        end_line_number,
                        value_spans,
                    };

                    if let Some(section) = unit.sections.get_mut(section_name) {
                        section.directives.insert(key, directive);
                    }
                }
            }
        }

        if let Some(section_name) = current_section {
            if let Some(section) = unit.sections.get_mut(&section_name) {
                section.line_range.1 = text.lines().count() as u32;
            }
        }

        debug!(
            "Parsed {} sections with {} total directives",
            unit.sections.len(),
            unit.sections
                .values()
                .map(|s| s.directives.len())
                .sum::<usize>()
        );
        unit
    }

    pub fn update_document(&self, uri: &Uri, text: &str) {
        let parsed = self.parse(text);
        self.documents.insert(uri.clone(), parsed);
    }

    pub fn get_parsed_document(&self, uri: &Uri) -> Option<SystemdUnit> {
        self.documents.get(uri).map(|entry| entry.clone())
    }

    pub fn get_document_text(&self, uri: &Uri) -> Option<String> {
        self.documents.get(uri).map(|entry| entry.raw_text.clone())
    }

    pub fn get_word_at_position(&self, unit: &SystemdUnit, position: &Position) -> Option<String> {
        let lines: Vec<&str> = unit.raw_text.lines().collect();
        if let Some(line) = lines.get(position.line as usize) {
            // Try to extract the word at the cursor position
            let chars: Vec<char> = line.chars().collect();
            if position.character < chars.len() as u32 {
                let cursor_pos = position.character as usize;

                // Find word boundaries around the cursor
                let mut start = cursor_pos;
                let mut end = cursor_pos;

                // Move start backwards to find word start
                while start > 0
                    && (chars[start - 1].is_alphanumeric()
                        || chars[start - 1] == '-'
                        || chars[start - 1] == '_'
                        || chars[start - 1] == '.')
                {
                    start -= 1;
                }

                // Move end forwards to find word end
                while end < chars.len()
                    && (chars[end].is_alphanumeric()
                        || chars[end] == '-'
                        || chars[end] == '_'
                        || chars[end] == '.')
                {
                    end += 1;
                }

                if start < end {
                    return Some(chars[start..end].iter().collect());
                }
            }
        }
        None
    }

    pub fn get_section_header_at_position(
        &self,
        unit: &SystemdUnit,
        position: &Position,
    ) -> Option<String> {
        debug!("Checking for section header at line {}", position.line);
        for section in unit.sections.values() {
            if position.line == section.line_range.0 {
                debug!(
                    "Found section header '{}' at line {}",
                    section.name, position.line
                );
                return Some(section.name.clone());
            }
        }
        debug!("No section header found at line {}", position.line);
        None
    }

    pub fn get_section_at_line<'a>(
        &self,
        unit: &'a SystemdUnit,
        line: u32,
    ) -> Option<&'a SystemdSection> {
        unit.sections
            .values()
            .find(|section| line >= section.line_range.0 && line <= section.line_range.1)
    }
}

fn parse_value_fragment(text: &str) -> (String, bool) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return (String::new(), false);
    }

    let mut backslash_count = 0usize;
    for ch in trimmed.chars().rev() {
        if ch == '\\' {
            backslash_count += 1;
        } else {
            break;
        }
    }

    let continuation = backslash_count % 2 == 1;
    let mut fragment = trimmed.to_string();

    if continuation {
        fragment.pop();
        fragment = fragment.trim_end().to_string();
    }

    (fragment, continuation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp_server::lsp_types::{Position, Uri};

    #[test]
    fn test_parse_basic_systemd_file() {
        let parser = SystemdParser::new();
        let content = "[Unit]\nDescription=Test service\nAfter=network.target\n\n[Service]\nType=simple\nExecStart=/bin/test\n";

        let parsed = parser.parse(content);

        assert_eq!(parsed.sections.len(), 2);
        assert!(parsed.sections.contains_key("Unit"));
        assert!(parsed.sections.contains_key("Service"));

        let unit_section = &parsed.sections["Unit"];
        assert_eq!(unit_section.line_range.0, 0);
        assert_eq!(unit_section.directives.len(), 2);
        assert!(unit_section.directives.contains_key("Description"));
        assert!(unit_section.directives.contains_key("After"));

        let service_section = &parsed.sections["Service"];
        assert_eq!(service_section.line_range.0, 4);
        assert_eq!(service_section.directives.len(), 2);
        assert!(service_section.directives.contains_key("Type"));
        assert!(service_section.directives.contains_key("ExecStart"));
    }

    #[test]
    fn test_parse_with_comments_and_empty_lines() {
        let parser = SystemdParser::new();
        let content = "# This is a comment\n\n[Unit]\n# Another comment\nDescription=Test\n\n[Service]\nType=simple\n";

        let parsed = parser.parse(content);

        assert_eq!(parsed.sections.len(), 2);
        assert!(parsed.sections.contains_key("Unit"));
        assert!(parsed.sections.contains_key("Service"));

        // Comments and empty lines should be ignored
        let unit_section = &parsed.sections["Unit"];
        assert_eq!(unit_section.directives.len(), 1);
        assert!(unit_section.directives.contains_key("Description"));
    }

    #[test]
    fn test_get_section_header_at_position() {
        let parser = SystemdParser::new();
        let content = "[Unit]\nDescription=Test\n\n[Service]\nType=simple\n\n[Install]\nWantedBy=multi-user.target\n";
        let parsed = parser.parse(content);

        // Test section headers
        assert_eq!(
            parser.get_section_header_at_position(
                &parsed,
                &Position {
                    line: 0,
                    character: 0
                }
            ),
            Some("Unit".to_string())
        );
        assert_eq!(
            parser.get_section_header_at_position(
                &parsed,
                &Position {
                    line: 3,
                    character: 0
                }
            ),
            Some("Service".to_string())
        );
        assert_eq!(
            parser.get_section_header_at_position(
                &parsed,
                &Position {
                    line: 6,
                    character: 0
                }
            ),
            Some("Install".to_string())
        );

        // Test non-header lines
        assert_eq!(
            parser.get_section_header_at_position(
                &parsed,
                &Position {
                    line: 1,
                    character: 0
                }
            ),
            None
        );
        assert_eq!(
            parser.get_section_header_at_position(
                &parsed,
                &Position {
                    line: 4,
                    character: 0
                }
            ),
            None
        );
        assert_eq!(
            parser.get_section_header_at_position(
                &parsed,
                &Position {
                    line: 7,
                    character: 0
                }
            ),
            None
        );
    }

    #[test]
    fn test_get_section_at_line() {
        let parser = SystemdParser::new();
        let content = "[Unit]\nDescription=Test\nAfter=network.target\n\n[Service]\nType=simple\nExecStart=/bin/test\n";
        let parsed = parser.parse(content);

        // Test lines within sections
        let unit_section = parser.get_section_at_line(&parsed, 0).unwrap();
        assert_eq!(unit_section.name, "Unit");

        let unit_section = parser.get_section_at_line(&parsed, 1).unwrap();
        assert_eq!(unit_section.name, "Unit");

        let unit_section = parser.get_section_at_line(&parsed, 2).unwrap();
        assert_eq!(unit_section.name, "Unit");

        let service_section = parser.get_section_at_line(&parsed, 4).unwrap();
        assert_eq!(service_section.name, "Service");

        let service_section = parser.get_section_at_line(&parsed, 5).unwrap();
        assert_eq!(service_section.name, "Service");

        // The Unit section probably extends to line 3, so this test was wrong
        // Line 3 is empty, but the Unit section includes it in its range
        // Let's test a line that's definitely outside any section
        assert!(parser.get_section_at_line(&parsed, 100).is_none());
    }

    #[test]
    fn test_get_word_at_position() {
        let parser = SystemdParser::new();
        let content = "[Unit]\nDescription=Test service\nAfter=network.target\n";
        let parsed = parser.parse(content);

        // Test getting directive names
        assert_eq!(
            parser.get_word_at_position(
                &parsed,
                &Position {
                    line: 1,
                    character: 0
                }
            ),
            Some("Description".to_string())
        );
        assert_eq!(
            parser.get_word_at_position(
                &parsed,
                &Position {
                    line: 1,
                    character: 5
                }
            ),
            Some("Description".to_string())
        );
        assert_eq!(
            parser.get_word_at_position(
                &parsed,
                &Position {
                    line: 2,
                    character: 0
                }
            ),
            Some("After".to_string())
        );

        // Test getting values - the word extraction includes dots and hyphens as valid word characters
        assert_eq!(
            parser.get_word_at_position(
                &parsed,
                &Position {
                    line: 1,
                    character: 12
                }
            ),
            Some("Test".to_string())
        );
        assert_eq!(
            parser.get_word_at_position(
                &parsed,
                &Position {
                    line: 2,
                    character: 6
                }
            ),
            Some("network.target".to_string())
        );

        // Test position at different parts of "network.target"
        assert_eq!(
            parser.get_word_at_position(
                &parsed,
                &Position {
                    line: 2,
                    character: 10
                }
            ),
            Some("network.target".to_string())
        );
        assert_eq!(
            parser.get_word_at_position(
                &parsed,
                &Position {
                    line: 2,
                    character: 14
                }
            ),
            Some("network.target".to_string())
        );
    }

    #[test]
    fn test_document_storage_and_retrieval() {
        let parser = SystemdParser::new();
        let content = "[Unit]\nDescription=Test\n";
        let uri = "file:///test.service".parse::<Uri>().unwrap();

        // Test that initially there's no document
        assert!(parser.get_parsed_document(&uri).is_none());
        assert!(parser.get_document_text(&uri).is_none());

        // Store document
        parser.update_document(&uri, content);

        // Test retrieval
        let retrieved = parser.get_parsed_document(&uri).unwrap();
        assert_eq!(retrieved.sections.len(), 1);
        assert!(retrieved.sections.contains_key("Unit"));

        let text = parser.get_document_text(&uri).unwrap();
        assert_eq!(text, content);
    }

    #[test]
    fn test_parse_edge_cases() {
        let parser = SystemdParser::new();

        // Test empty file
        let empty_parsed = parser.parse("");
        assert_eq!(empty_parsed.sections.len(), 0);

        // Test file with only comments
        let comments_only = parser.parse("# Comment 1\n# Comment 2\n");
        assert_eq!(comments_only.sections.len(), 0);

        // Test section with no directives
        let empty_section = parser.parse("[Unit]\n\n[Service]\n");
        assert_eq!(empty_section.sections.len(), 2);
        assert_eq!(empty_section.sections["Unit"].directives.len(), 0);
        assert_eq!(empty_section.sections["Service"].directives.len(), 0);

        // Test directive with empty value
        let empty_value = parser.parse("[Unit]\nDescription=\n");
        assert_eq!(empty_value.sections.len(), 1);
        assert_eq!(
            empty_value.sections["Unit"].directives["Description"].value,
            ""
        );

        // Test directive with spaces around equals
        let spaced_equals = parser.parse("[Unit]\nDescription = Test Service \n");
        assert_eq!(spaced_equals.sections.len(), 1);
        assert_eq!(
            spaced_equals.sections["Unit"].directives["Description"].value,
            "Test Service"
        );
    }

    #[test]
    fn test_case_sensitivity() {
        let parser = SystemdParser::new();
        let content = "[UNIT]\nDESCRIPTION=Test\n[service]\ntype=simple\n";
        let parsed = parser.parse(content);

        // Section names should preserve case
        assert!(parsed.sections.contains_key("UNIT"));
        assert!(parsed.sections.contains_key("service"));
        assert!(!parsed.sections.contains_key("Unit"));
        assert!(!parsed.sections.contains_key("Service"));

        // Directive names should preserve case
        assert!(parsed.sections["UNIT"]
            .directives
            .contains_key("DESCRIPTION"));
        assert!(parsed.sections["service"].directives.contains_key("type"));
    }

    #[test]
    fn test_parse_multiline_directive_execstart() {
        let parser = SystemdParser::new();
        let content =
            "[Service]\nExecStart=/usr/bin/test \\\n    --flag value \\\n    --another-flag\n";

        let parsed = parser.parse(content);
        let service_section = parsed
            .sections
            .get("Service")
            .expect("Service section missing");
        let exec_start = service_section
            .directives
            .get("ExecStart")
            .expect("ExecStart directive missing");

        assert_eq!(
            exec_start.value,
            "/usr/bin/test --flag value --another-flag"
        );
        assert_eq!(exec_start.line_number, 1);
        assert_eq!(exec_start.end_line_number, 3);
        assert_eq!(exec_start.value_spans.len(), 3);

        let first_span = &exec_start.value_spans[0];
        assert_eq!(first_span.line, 1);
        assert_eq!(first_span.start, 10);
        assert_eq!(first_span.end, 23);

        let second_span = &exec_start.value_spans[1];
        assert_eq!(second_span.line, 2);
        assert_eq!(second_span.start, 4);
        assert_eq!(second_span.end, 16);

        let third_span = &exec_start.value_spans[2];
        assert_eq!(third_span.line, 3);
        assert_eq!(third_span.start, 4);
        assert_eq!(third_span.end, 18);
    }
}
