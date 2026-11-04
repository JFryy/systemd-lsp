use crate::parser::{lsp_u32, utf16_len};
use log::{debug, trace};
use tower_lsp_server::ls_types::{Position, Range, TextEdit, Uri};

#[derive(Debug, Default)]
pub struct SystemdFormatter;

impl SystemdFormatter {
    pub fn new() -> Self {
        Self
    }

    pub fn format_document(&self, uri: &Uri, text: &str) -> Vec<TextEdit> {
        debug!("Formatting document: {uri:?}");
        trace!("Document text length: {}", text.len());

        let formatted_content = self.apply_opinionated_formatting(text);

        // If content changed, replace the entire document
        if formatted_content != text {
            vec![TextEdit {
                range: Range::new(
                    Position::new(0, 0),
                    Position::new(lsp_u32(text.lines().count()), 0),
                ),
                new_text: formatted_content,
            }]
        } else {
            vec![]
        }
    }

    /// Formats the lines touched by `range`, leaving the rest of the document untouched.
    ///
    /// Only the per-line rules apply (trimming, `key=value` spacing, comment cleanup).
    /// Blank lines are never added or removed, because section spacing depends on lines
    /// outside the selection; use document formatting for that.
    pub fn format_range(&self, uri: &Uri, text: &str, range: Range) -> Vec<TextEdit> {
        debug!("Formatting range in document: {uri:?}");

        let start = range.start.line as usize;
        let mut end = range.end.line as usize;
        // A selection that ends at the start of a line doesn't include that line.
        if range.end.character == 0 && end > start {
            end -= 1;
        }

        text.lines()
            .enumerate()
            .skip(start)
            .take((end + 1).saturating_sub(start))
            .filter_map(|(line_number, line)| {
                let formatted = Self::format_line(line);
                (formatted != line).then(|| {
                    let line_number = lsp_u32(line_number);
                    TextEdit {
                        range: Range::new(
                            Position::new(line_number, 0),
                            Position::new(line_number, utf16_len(line)),
                        ),
                        new_text: formatted,
                    }
                })
            })
            .collect()
    }

    /// Applies the rules that only depend on the line itself.
    fn format_line(line: &str) -> String {
        let trimmed = line.trim();
        if trimmed.starts_with(['#', ';']) || (trimmed.starts_with('[') && trimmed.ends_with(']')) {
            return trimmed.to_string();
        }
        match trimmed.split_once('=') {
            // Opinionated formatting: no spaces around equals
            Some((key, value)) => format!("{}={}", key.trim(), value.trim()),
            None => trimmed.to_string(),
        }
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

            // Comments keep the blank-line state unchanged
            if trimmed.starts_with(['#', ';']) {
                result.push(Self::format_line(trimmed));
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                // Add single blank line before section (except for first section)
                if in_section || previous_was_section {
                    result.push(String::new());
                }
                in_section = true;
                previous_was_section = true;
            } else {
                previous_was_section = false;
            }
            result.push(Self::format_line(trimmed));
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
    fn test_opinionated_formatting_preserves_semicolon_comments() {
        let formatter = SystemdFormatter::new();
        let input = "[Service]\n  ; Restart = always\nType=simple\n";
        let expected = "[Service]\n; Restart = always\nType=simple\n";

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
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let input = "[Unit]\nDescription = Test\n[Service]\nType = simple\n";

        let edits = formatter.format_document(&uri, input);
        assert_eq!(edits.len(), 1);

        let expected = "[Unit]\nDescription=Test\n\n[Service]\nType=simple\n";
        assert_eq!(edits[0].new_text, expected);
    }

    #[test]
    fn test_format_document_no_changes_needed() {
        let formatter = SystemdFormatter::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
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

    fn range(start_line: u32, start_char: u32, end_line: u32, end_char: u32) -> Range {
        Range::new(
            Position::new(start_line, start_char),
            Position::new(end_line, end_char),
        )
    }

    fn line_edit(line: u32, end_char: u32, new_text: &str) -> TextEdit {
        TextEdit {
            range: range(line, 0, line, end_char),
            new_text: new_text.to_string(),
        }
    }

    #[test]
    fn test_format_range_only_edits_selected_lines() {
        let formatter = SystemdFormatter::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let input =
            "[Unit]\nDescription = Test\n  After = network.target\n[Service]\nType = simple\n";

        // The selection starts and ends inside lines 1 and 2; both lines are formatted.
        let edits = formatter.format_range(&uri, input, range(1, 4, 2, 3));
        assert_eq!(
            edits,
            vec![
                line_edit(1, 18, "Description=Test"),
                line_edit(2, 24, "After=network.target"),
            ]
        );
    }

    #[test]
    fn test_format_range_excludes_line_where_selection_ends_at_column_zero() {
        let formatter = SystemdFormatter::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let input = "[Unit]\nDescription = Test\nAfter = network.target\n";

        let edits = formatter.format_range(&uri, input, range(1, 0, 2, 0));
        assert_eq!(edits, vec![line_edit(1, 18, "Description=Test")]);
    }

    #[test]
    fn test_format_range_does_not_add_or_remove_lines() {
        let formatter = SystemdFormatter::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        // Document formatting would add a blank line before [Service] and drop the
        // extra blank lines; range formatting leaves the line structure alone.
        let input = "[Unit]\nDescription=Test\n\n\n  \n[Service]\nType=simple\n";

        let edits = formatter.format_range(&uri, input, range(0, 0, 7, 0));
        assert_eq!(edits, vec![line_edit(4, 2, "")]);
    }

    #[test]
    fn test_format_range_without_changes_returns_no_edits() {
        let formatter = SystemdFormatter::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let input = "[Unit]\nDescription=Test\n";

        assert!(formatter
            .format_range(&uri, input, range(0, 0, 1, 16))
            .is_empty());
    }

    #[test]
    fn test_format_range_uses_utf16_columns_and_ignores_crlf() {
        let formatter = SystemdFormatter::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let input = "[Unit]\r\nDescription = Café 🚀\r\n";

        // "Description = Café 🚀" is 21 UTF-16 code units; the `\r` is not part of the edit.
        let edits = formatter.format_range(&uri, input, range(1, 0, 1, 0));
        assert_eq!(edits, vec![line_edit(1, 21, "Description=Café 🚀")]);
    }
}
