use log::{debug, trace};
use tower_lsp::lsp_types::{Position, Range, TextEdit, Url};

#[derive(Debug)]
pub struct SystemdFormatter;

impl SystemdFormatter {
    pub fn new() -> Self {
        Self
    }

    pub fn format_document(&self, uri: &Url, text: &str) -> Vec<TextEdit> {
        debug!("Formatting document: {}", uri);
        trace!("Document text length: {}", text.len());

        let formatted_content = self.apply_opinionated_formatting(text);

        // If content changed, replace the entire document
        if formatted_content != text {
            vec![TextEdit {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: text.lines().count() as u32,
                        character: 0,
                    },
                },
                new_text: formatted_content,
            }]
        } else {
            vec![]
        }
    }

    pub fn format_range(&self, uri: &Url, text: &str, _range: Range) -> Vec<TextEdit> {
        debug!("Formatting range in document: {}", uri);

        // For range formatting, we'll format the entire document to maintain consistency
        // since our opinionated formatting affects spacing between sections
        self.format_document(uri, text)
    }

    fn apply_opinionated_formatting(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut in_section = false;
        let mut previous_was_section = false;

        for line in lines.iter() {
            let trimmed = line.trim();

            // Skip completely empty lines - we'll add them back strategically
            if trimmed.is_empty() {
                continue;
            }

            // Handle comments - preserve but clean whitespace
            if trimmed.starts_with('#') {
                result.push(trimmed.to_string());
                continue;
            }

            // Handle section headers
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                // Add single blank line before section (except for first section)
                if in_section || previous_was_section {
                    result.push(String::new());
                }

                result.push(trimmed.to_string());
                in_section = true;
                previous_was_section = true;
                continue;
            }

            // Handle directives (key=value pairs)
            if let Some(equals_pos) = trimmed.find('=') {
                let key = trimmed[..equals_pos].trim();
                let value = trimmed[equals_pos + 1..].trim();

                // Opinionated formatting: no spaces around equals
                let formatted = format!("{}={}", key, value);
                result.push(formatted);
                previous_was_section = false;
                continue;
            }

            // Handle any other lines (preserve them but trim whitespace)
            result.push(trimmed.to_string());
            previous_was_section = false;
        }

        // Join with newlines and ensure file ends with single newline
        let mut formatted = result.join("\n");
        if !formatted.is_empty() && !formatted.ends_with('\n') {
            formatted.push('\n');
        }

        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opinionated_formatting_section_spacing() {
        let formatter = SystemdFormatter::new();
        let input = "[Unit]\nDescription=Test\n[Service]\nType=simple\n[Install]\nWantedBy=multi-user.target\n";
        let expected = "[Unit]\nDescription=Test\n\n[Service]\nType=simple\n\n[Install]\nWantedBy=multi-user.target\n";

        let formatted = formatter.apply_opinionated_formatting(input);
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_opinionated_formatting_no_spaces_around_equals() {
        let formatter = SystemdFormatter::new();
        let input = "[Unit]\nDescription = Test Service\nAfter =  network.target\nWants=   network-online.target\n";
        let expected =
            "[Unit]\nDescription=Test Service\nAfter=network.target\nWants=network-online.target\n";

        let formatted = formatter.apply_opinionated_formatting(input);
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_opinionated_formatting_preserves_comments() {
        let formatter = SystemdFormatter::new();
        let input = "# This is a comment\n[Unit]\n# Another comment\nDescription=Test\n";
        let expected = "# This is a comment\n[Unit]\n# Another comment\nDescription=Test\n";

        let formatted = formatter.apply_opinionated_formatting(input);
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_opinionated_formatting_removes_extra_blank_lines() {
        let formatter = SystemdFormatter::new();
        let input = "\n\n[Unit]\n\n\nDescription=Test\n\n\n\n[Service]\n\n\nType=simple\n\n\n";
        let expected = "[Unit]\nDescription=Test\n\n[Service]\nType=simple\n";

        let formatted = formatter.apply_opinionated_formatting(input);
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_opinionated_formatting_single_section() {
        let formatter = SystemdFormatter::new();
        let input = "[Unit]\nDescription=Test\nAfter=network.target\n";
        let expected = "[Unit]\nDescription=Test\nAfter=network.target\n";

        let formatted = formatter.apply_opinionated_formatting(input);
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_format_document_integration() {
        let formatter = SystemdFormatter::new();
        let uri = Url::parse("file:///test.service").unwrap();
        let input = "[Unit]\nDescription = Test\n[Service]\nType = simple\n";

        let edits = formatter.format_document(&uri, input);
        assert_eq!(edits.len(), 1);

        let expected = "[Unit]\nDescription=Test\n\n[Service]\nType=simple\n";
        assert_eq!(edits[0].new_text, expected);
    }

    #[test]
    fn test_format_document_no_changes_needed() {
        let formatter = SystemdFormatter::new();
        let uri = Url::parse("file:///test.service").unwrap();
        let input = "[Unit]\nDescription=Test\n\n[Service]\nType=simple\n";

        let edits = formatter.format_document(&uri, input);
        assert_eq!(edits.len(), 0);
    }

    #[test]
    fn test_opinionated_formatting_no_blank_lines_within_sections() {
        let formatter = SystemdFormatter::new();
        let input = "[Unit]\nDescription=Test\n\nAfter=network.target\n\n\n[Service]\nType=simple\n\nExecStart=/bin/test\n\nRestart=always\n";
        let expected = "[Unit]\nDescription=Test\nAfter=network.target\n\n[Service]\nType=simple\nExecStart=/bin/test\nRestart=always\n";

        let formatted = formatter.apply_opinionated_formatting(input);
        assert_eq!(formatted, expected);
    }
}
