use dashmap::DashMap;
use log::{debug, trace};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tower_lsp_server::ls_types::{Position, Uri};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdUnit {
    pub sections: HashMap<String, SystemdSection>,
    pub raw_text: String,
}

/// A section and the directives from every occurrence of its header. systemd merges
/// repeated sections, so each occurrence keeps its own line range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdSection {
    pub name: String,
    pub directives: Vec<SystemdDirective>,
    pub line_ranges: Vec<(u32, u32)>,
}

impl SystemdSection {
    pub fn contains_line(&self, line: u32) -> bool {
        self.line_ranges
            .iter()
            .any(|&(start, end)| line >= start && line <= end)
    }

    pub fn is_header_line(&self, line: u32) -> bool {
        self.line_ranges.iter().any(|&(start, _)| start == line)
    }
}

/// Number of UTF-16 code units in `text`, the column unit LSP positions use by default.
pub fn utf16_len(text: &str) -> u32 {
    text.encode_utf16().count() as u32
}

/// Byte index in `line` for a UTF-16 column, clamped to the line end and to char boundaries.
pub fn utf16_to_byte(line: &str, column: u32) -> usize {
    let mut units = 0u32;
    for (byte_index, ch) in line.char_indices() {
        if units >= column {
            return byte_index;
        }
        units += ch.len_utf16() as u32;
    }
    line.len()
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
    documents: DashMap<Uri, Arc<SystemdUnit>>,
    section_regex: Regex,
    directive_regex: Regex,
}

impl SystemdParser {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for SystemdParser {
    fn default() -> Self {
        Self {
            documents: DashMap::new(),
            section_regex: Regex::new(r"^\[([^\]]+)\]$").unwrap(),
            directive_regex: Regex::new(r"^([^=]+)=(.*)$").unwrap(),
        }
    }
}

impl SystemdParser {
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
                    if let Some(range) = unit
                        .sections
                        .get_mut(&section_name)
                        .and_then(|section| section.line_ranges.last_mut())
                    {
                        range.1 = line_num - 1;
                    }
                }

                let section_name = captures[1].to_string();
                current_section = Some(section_name.clone());

                unit.sections
                    .entry(section_name.clone())
                    .or_insert_with(|| SystemdSection {
                        name: section_name,
                        directives: Vec::new(),
                        line_ranges: Vec::new(),
                    })
                    .line_ranges
                    .push((line_num, line_num));
            } else if let Some(captures) = self.directive_regex.captures(trimmed) {
                if let Some(section_name) = &current_section {
                    let key = captures[1].trim().to_string();
                    let raw_value = captures[2].to_string();

                    // Byte offsets into `line`, converted to UTF-16 columns for the protocol.
                    let key_start_byte = line.find(&key).unwrap_or(0);
                    let key_end_byte = key_start_byte + key.len();
                    let key_start = utf16_len(&line[..key_start_byte]);
                    let key_end = utf16_len(&line[..key_end_byte]);

                    let after_eq_byte = line.find('=').map_or(key_end_byte, |idx| idx + 1);
                    let after_eq = &line[after_eq_byte..];
                    let value_start_byte =
                        after_eq_byte + (after_eq.len() - after_eq.trim_start().len());
                    let value_start = utf16_len(&line[..value_start_byte]);

                    let (mut fragment, mut continuation) = parse_value_fragment(&raw_value);
                    let mut normalized_value = fragment.clone();
                    let mut value_spans = Vec::new();

                    value_spans.push(DirectiveValueSpan {
                        line: line_num,
                        start: value_start,
                        end: value_start + utf16_len(&fragment),
                    });

                    let mut end_line_number = line_num;

                    while continuation {
                        if let Some((next_line_num, next_line)) = lines.next() {
                            let next_line_trimmed = next_line.trim();
                            end_line_number = next_line_num as u32;

                            let indent_byte = next_line.find(next_line_trimmed).unwrap_or(0);
                            let indent = utf16_len(&next_line[..indent_byte]);

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
                                end: indent + utf16_len(&next_fragment),
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
                            end: value_start + utf16_len(&normalized_value),
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
                        section.directives.push(directive);
                    }
                }
            }
        }

        if let Some(section_name) = current_section {
            if let Some(range) = unit
                .sections
                .get_mut(&section_name)
                .and_then(|section| section.line_ranges.last_mut())
            {
                range.1 = text.lines().count() as u32;
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

    /// Parses `text`, stores it for `uri` and returns the shared parse result.
    pub fn update_document(&self, uri: &Uri, text: &str) -> Arc<SystemdUnit> {
        let parsed = Arc::new(self.parse(text));
        self.documents.insert(uri.clone(), Arc::clone(&parsed));
        parsed
    }

    /// Cheap handle to the stored parse result; the unit itself is not copied.
    pub fn get_parsed_document(&self, uri: &Uri) -> Option<Arc<SystemdUnit>> {
        self.documents.get(uri).map(|entry| Arc::clone(&entry))
    }

    pub fn get_document_text(&self, uri: &Uri) -> Option<String> {
        self.documents.get(uri).map(|entry| entry.raw_text.clone())
    }

    pub fn get_word_at_position(&self, unit: &SystemdUnit, position: &Position) -> Option<String> {
        fn is_word_char(ch: char) -> bool {
            ch.is_alphanumeric() || matches!(ch, '-' | '_' | '.')
        }

        let line = unit.raw_text.lines().nth(position.line as usize)?;
        let cursor = utf16_to_byte(line, position.character);
        if cursor >= line.len() {
            return None;
        }

        let start = line[..cursor]
            .char_indices()
            .rev()
            .take_while(|&(_, ch)| is_word_char(ch))
            .last()
            .map_or(cursor, |(index, _)| index);
        let end = line[cursor..]
            .find(|ch| !is_word_char(ch))
            .map_or(line.len(), |offset| cursor + offset);

        (start < end).then(|| line[start..end].to_string())
    }

    pub fn get_section_header_at_position(
        &self,
        unit: &SystemdUnit,
        position: &Position,
    ) -> Option<String> {
        debug!("Checking for section header at line {}", position.line);
        for section in unit.sections.values() {
            if section.is_header_line(position.line) {
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
            .find(|section| section.contains_line(line))
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
    use tower_lsp_server::ls_types::{Position, Uri};

    #[test]
    fn test_parse_basic_systemd_file() {
        let parser = SystemdParser::new();
        let content = "[Unit]\nDescription=Test service\nAfter=network.target\n\n[Service]\nType=simple\nExecStart=/bin/test\n";

        let parsed = parser.parse(content);

        assert_eq!(parsed.sections.len(), 2);
        assert!(parsed.sections.contains_key("Unit"));
        assert!(parsed.sections.contains_key("Service"));

        let unit_section = &parsed.sections["Unit"];
        assert_eq!(unit_section.line_ranges[0].0, 0);
        assert_eq!(unit_section.directives.len(), 2);
        assert!(unit_section
            .directives
            .iter()
            .find(|directive| directive.key == "Description")
            .is_some());
        assert!(unit_section
            .directives
            .iter()
            .find(|directive| directive.key == "After")
            .is_some());

        let service_section = &parsed.sections["Service"];
        assert_eq!(service_section.line_ranges[0].0, 4);
        assert_eq!(service_section.directives.len(), 2);
        assert!(service_section
            .directives
            .iter()
            .find(|directive| directive.key == "Type")
            .is_some());
        assert!(service_section
            .directives
            .iter()
            .find(|directive| directive.key == "ExecStart")
            .is_some());
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
        assert!(unit_section
            .directives
            .iter()
            .find(|directive| directive.key == "Description")
            .is_some());
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
            empty_value.sections["Unit"]
                .directives
                .iter()
                .find(|directive| directive.key == "Description")
                .unwrap()
                .value,
            ""
        );

        // Test directive with spaces around equals
        let spaced_equals = parser.parse("[Unit]\nDescription = Test Service \n");
        assert_eq!(spaced_equals.sections.len(), 1);
        assert_eq!(
            spaced_equals.sections["Unit"]
                .directives
                .iter()
                .find(|directive| directive.key == "Description")
                .unwrap()
                .value,
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
            .iter()
            .find(|directive| directive.key == "DESCRIPTION")
            .is_some());
        assert!(parsed.sections["service"]
            .directives
            .iter()
            .find(|directive| directive.key == "type")
            .is_some());
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
            .iter()
            .find(|directive| directive.key == "ExecStart")
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

    #[test]
    fn test_repeated_sections_are_merged() {
        let parser = SystemdParser::new();
        let content = "[Service]\nExecStart=\n\n[Unit]\nDescription=x\n\n[Service]\nType=simple\n";
        let parsed = parser.parse(content);

        let service = &parsed.sections["Service"];
        let keys: Vec<&str> = service.directives.iter().map(|d| d.key.as_str()).collect();
        assert_eq!(keys, ["ExecStart", "Type"]);
        assert_eq!(service.line_ranges, [(0, 2), (6, 8)]);

        assert_eq!(
            parser.get_section_at_line(&parsed, 1).unwrap().name,
            "Service"
        );
        assert_eq!(parser.get_section_at_line(&parsed, 4).unwrap().name, "Unit");
        assert_eq!(
            parser.get_section_at_line(&parsed, 7).unwrap().name,
            "Service"
        );
        assert_eq!(
            parser.get_section_header_at_position(&parsed, &Position::new(6, 0)),
            Some("Service".to_string())
        );
    }

    #[test]
    fn test_columns_are_utf16() {
        let parser = SystemdParser::new();
        // "é" is 2 bytes / 1 UTF-16 unit; "😀" is 4 bytes / 2 UTF-16 units.
        let content = "[Unit]\nDescription=é 😀 x\n[Service]\nExecStart=/bin/a \\\n  ü \\\n  b\n";
        let parsed = parser.parse(content);

        let description = &parsed.sections["Unit"].directives[0];
        assert_eq!(description.column_range, (0, 11));
        assert_eq!(description.value_spans[0].start, 12);
        assert_eq!(description.value_spans[0].end, 12 + 6);

        let exec_start = &parsed.sections["Service"].directives[0];
        let spans: Vec<(u32, u32, u32)> = exec_start
            .value_spans
            .iter()
            .map(|span| (span.line, span.start, span.end))
            .collect();
        assert_eq!(spans, [(3, 10, 16), (4, 2, 3), (5, 2, 3)]);
    }

    #[test]
    fn test_get_word_at_position_uses_utf16_columns() {
        let parser = SystemdParser::new();
        // Each emoji is 1 char but 2 UTF-16 units, so "y" is at UTF-16 column 19
        // while the line is only 18 chars long.
        let parsed = parser.parse("[Unit]\nDescription=\u{1F600}\u{1F600} x y\n");
        let word = parser.get_word_at_position(&parsed, &Position::new(1, 19));
        assert_eq!(word, Some("y".to_string()));
    }

    #[test]
    fn test_get_word_at_position_boundaries() {
        let parser = SystemdParser::new();
        let parsed = parser.parse("[Unit]\nAfter=network-online.target\n");
        let word_at =
            |character| parser.get_word_at_position(&parsed, &Position::new(1, character));

        assert_eq!(word_at(0), Some("After".to_string()));
        assert_eq!(word_at(2), Some("After".to_string()));
        // On "=", the word ending at the cursor is used.
        assert_eq!(word_at(5), Some("After".to_string()));
        assert_eq!(word_at(10), Some("network-online.target".to_string()));
        assert_eq!(word_at(26), Some("network-online.target".to_string()));
        assert_eq!(word_at(27), None);
        assert_eq!(
            parser.get_word_at_position(&parsed, &Position::new(9, 0)),
            None
        );
    }

    #[test]
    fn test_utf16_to_byte() {
        let line = "aé😀b";
        assert_eq!(utf16_to_byte(line, 0), 0);
        assert_eq!(utf16_to_byte(line, 1), 1);
        assert_eq!(utf16_to_byte(line, 2), 3);
        assert_eq!(utf16_to_byte(line, 4), 7);
        assert_eq!(utf16_to_byte(line, 99), line.len());
    }
}
