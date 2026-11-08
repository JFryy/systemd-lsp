use crate::constants::SystemdConstants;
use crate::parser::{utf16_len, utf16_to_byte, SystemdParser, SystemdUnit};
use log::{debug, trace};
use std::collections::HashMap;
use tower_lsp_server::ls_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, CompletionTextEdit, Documentation,
    MarkupContent, MarkupKind, Position, Range, TextEdit, Uri,
};

#[derive(Debug)]
pub struct SystemdCompletion {
    section_completions: Vec<CompletionItem>,
    directive_completions: HashMap<String, Vec<CompletionItem>>,
}

#[derive(Debug)]
enum CompletionContext<'a> {
    SectionHeader,
    Directive(&'a str),
    Value {
        section: &'a str,
        directive: &'a str,
    },
    Global,
}

impl Default for SystemdCompletion {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemdCompletion {
    pub fn new() -> Self {
        let mut section_completions = Vec::new();
        for (&name, &description) in SystemdConstants::section_documentation() {
            let documentation = Self::create_documentation(
                &format!("[{name}] Section"),
                description,
                &format!("systemd.{}.5", name.to_lowercase()),
            );

            section_completions.push(Self::create_completion_item(
                format!("[{name}]"),
                CompletionItemKind::MODULE,
                format!("systemd {} section", name.to_lowercase()),
                documentation,
                Some(format!("[{name}]")),
            ));
        }

        let mut directive_completions = HashMap::new();
        let directive_descriptions = SystemdConstants::directive_descriptions();

        for (&section, directives) in SystemdConstants::section_directives() {
            let completion_items = directives
                .iter()
                .map(|&directive| {
                    let description = directive_descriptions
                        .get(&(section, directive))
                        .copied()
                        .unwrap_or("systemd directive");
                    Self::create_directive_completion(section, directive, description)
                })
                .collect();
            directive_completions.insert(section.to_string(), completion_items);
        }

        Self {
            section_completions,
            directive_completions,
        }
    }

    pub fn get_completions(
        &self,
        parser: &SystemdParser,
        uri: &Uri,
        position: &Position,
    ) -> Option<CompletionResponse> {
        trace!(
            "Generating completions for {}:{} in {:?}",
            position.line,
            position.character,
            uri
        );

        let unit = parser.get_parsed_document(uri)?;
        let document_text = unit.raw_text.as_str();
        let completion_context = Self::determine_context(parser, &unit, position);

        debug!("Completion context: {completion_context:?}");

        match completion_context {
            CompletionContext::SectionHeader => {
                // Only show section completions
                debug!("Providing section completions");
                let Some(range) = Self::section_header_range(document_text, position) else {
                    return Some(CompletionResponse::Array(self.section_completions.clone()));
                };
                let items = self
                    .section_completions
                    .iter()
                    .cloned()
                    .map(|mut item| {
                        let new_text = item.insert_text.clone().unwrap_or(item.label.clone());
                        item.text_edit =
                            Some(CompletionTextEdit::Edit(TextEdit::new(range, new_text)));
                        item
                    })
                    .collect();
                Some(CompletionResponse::Array(items))
            }
            CompletionContext::Directive(section_name) => {
                // Only show directives for the current section
                debug!("Providing directive completions for section: {section_name}");
                if let Some(directives) = self.directive_completions.get(section_name) {
                    Some(CompletionResponse::Array(directives.clone()))
                } else {
                    debug!("No directives found for section: {section_name}");
                    Some(CompletionResponse::Array(Vec::new()))
                }
            }
            CompletionContext::Value {
                section: section_name,
                directive,
            } => {
                debug!("Providing value completions for {section_name}.{directive}");
                match Self::get_value_completions(section_name, directive) {
                    Some(items) if !items.is_empty() => Some(CompletionResponse::Array(items)),
                    _ => {
                        debug!("No value completions available for {section_name}.{directive}");
                        None
                    }
                }
            }
            CompletionContext::Global => {
                // Show section completions if we're not inside any section
                debug!("Providing global completions (sections)");
                Some(CompletionResponse::Array(self.section_completions.clone()))
            }
        }
    }

    /// Range of the partially typed header ("[Se" or "[Se]") that a section completion replaces,
    /// so clients don't insert a second "[". Columns are UTF-16 code units.
    fn section_header_range(document_text: &str, position: &Position) -> Option<Range> {
        let line = document_text.lines().nth(position.line as usize)?;
        let cursor_byte = utf16_to_byte(line, position.character);
        let start_byte = line[..cursor_byte].find('[')?;
        let end_byte = if line[cursor_byte..].starts_with(']') {
            cursor_byte + 1
        } else {
            cursor_byte
        };
        Some(Range::new(
            Position::new(position.line, utf16_len(&line[..start_byte])),
            Position::new(position.line, utf16_len(&line[..end_byte])),
        ))
    }

    fn determine_context<'a>(
        parser: &SystemdParser,
        unit: &'a SystemdUnit,
        position: &Position,
    ) -> CompletionContext<'a> {
        let Some(current_line) = unit.raw_text.lines().nth(position.line as usize) else {
            return CompletionContext::Global;
        };

        if current_line.trim_start().starts_with('[') {
            return CompletionContext::SectionHeader;
        }

        // Check if we're currently on a section header line
        if let Some(_section_name) = parser.get_section_header_at_position(unit, position) {
            return CompletionContext::SectionHeader;
        }

        // Check if we're inside a section (for directive completions)
        if let Some(section) = parser.get_section_at_line(unit, position.line) {
            // Check if cursor is positioned within the value part of a directive on the same line
            if let Some((raw_key, _)) = current_line.split_once('=') {
                let key = raw_key.trim();
                if position.character > utf16_len(raw_key) && !key.is_empty() {
                    return CompletionContext::Value {
                        section: &section.name,
                        directive: key,
                    };
                }
            }

            // Detect multi-line or continuation value contexts using recorded spans
            if let Some(directive) = section.directives.iter().find(|directive| {
                directive.value_spans.iter().any(|span| {
                    span.line == position.line
                        && (span.line != directive.line_number || position.character >= span.start)
                })
            }) {
                return CompletionContext::Value {
                    section: &section.name,
                    directive: &directive.key,
                };
            }

            return CompletionContext::Directive(&section.name);
        }

        // Default to global context (show sections)
        CompletionContext::Global
    }

    fn get_value_completions(
        section_name: &str,
        directive_name: &str,
    ) -> Option<Vec<CompletionItem>> {
        let canonical_section =
            SystemdConstants::canonical_section(section_name).unwrap_or(section_name);
        let canonical_directive =
            SystemdConstants::canonical_directive(section_name, directive_name)
                .unwrap_or(directive_name);

        let values =
            SystemdConstants::valid_values_for_section(canonical_section, canonical_directive)?;

        if values.is_empty() {
            return None;
        }

        let items = values
            .iter()
            .map(|value| {
                Self::create_value_completion(canonical_section, canonical_directive, value)
            })
            .collect::<Vec<_>>();

        Some(items)
    }

    fn create_documentation(title: &str, description: &str, reference: &str) -> Documentation {
        Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("**{title}**\n\n{description}\n\n---\n*Reference: {reference}*"),
        })
    }

    fn create_completion_item(
        label: String,
        kind: CompletionItemKind,
        detail: String,
        documentation: Documentation,
        insert_text: Option<String>,
    ) -> CompletionItem {
        CompletionItem {
            label,
            label_details: None,
            kind: Some(kind),
            detail: Some(detail),
            documentation: Some(documentation),
            deprecated: None,
            preselect: None,
            sort_text: None,
            filter_text: None,
            insert_text,
            insert_text_format: None,
            insert_text_mode: None,
            text_edit: None,
            additional_text_edits: None,
            command: None,
            commit_characters: None,
            data: None,
            tags: None,
        }
    }

    /*
     * We Want to extract directive documentation from markdown files. To allow for
     * deduplication and maintainability, each section has a comprehensive markdown file
     * we require a strict format to adhere to discoverability of the documenation.
     * Most notably, directive headers must be prefixed with "### " and suffixed with "=".
     */
    fn extract_directive_from_markdown(section_name: &str, directive_name: &str) -> Option<String> {
        // Helper function to search for a directive in a markdown string
        fn search_in_markdown(markdown_content: &str, directive_name: &str) -> Option<String> {
            let directive_header = format!("### {directive_name}=");
            let directive_header_lower = directive_header.to_lowercase();

            let lines = markdown_content.lines();
            let mut found_header = false;
            let mut doc_lines = Vec::new();

            for line in lines {
                if line.to_lowercase() == directive_header_lower {
                    found_header = true;
                    continue;
                }

                if found_header {
                    // Stop at the next directive header (###) or section header (##)
                    if line.starts_with("### ") || line.starts_with("## ") {
                        break;
                    }
                    doc_lines.push(line);
                }
            }

            if doc_lines.is_empty() {
                return None;
            }

            // Join the lines and trim
            let documentation = doc_lines.join("\n").trim().to_string();

            // Remove the trailing "**Reference:**" line if present
            let documentation = if let Some(last_ref_pos) = documentation.rfind("**Reference:**") {
                documentation[..last_ref_pos].trim().to_string()
            } else {
                documentation
            };

            Some(documentation)
        }

        // First, try to find the directive in the section's own markdown file
        if let Some(result) = SystemdConstants::section_documentation_for(section_name)
            .and_then(|content| search_in_markdown(content, directive_name))
        {
            return Some(result);
        }

        // If not found in the section's own docs, search in shared documentation
        // (exec, kill, resource-control) based on which section we're in
        let shared_docs = SystemdConstants::shared_documentation();
        SystemdConstants::section_shared_docs(section_name)
            .iter()
            .filter_map(|key| shared_docs.get(key))
            .find_map(|content| search_in_markdown(content, directive_name))
    }

    // We also leverage the directive completion for auto complete and hover.
    fn create_directive_completion(
        section: &str,
        key: &str,
        short_description: &str,
    ) -> CompletionItem {
        // Try to get comprehensive markdown documentation
        let documentation =
            if let Some(markdown_doc) = Self::extract_directive_from_markdown(section, key) {
                Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**{key}**\n\n{markdown_doc}"),
                })
            } else {
                // Fall back to short description
                Self::create_documentation(key, short_description, "systemd documentation")
            };

        Self::create_completion_item(
            key.to_string(),
            CompletionItemKind::PROPERTY,
            "systemd directive".to_string(),
            documentation,
            Some(format!("{key}=")),
        )
    }

    /* Extract value-specific documentation from the directive markdown. Similarly to directive
     * being parsed from sections, we also use a format to find matches for value documentation.
     * The prefix of "- **" is required, followed by the value name, and then either ":
     * description".
     */
    fn extract_value_documentation(
        section_name: &str,
        directive_name: &str,
        value: &str,
    ) -> Option<String> {
        // Get the full directive documentation
        let directive_doc = Self::extract_directive_from_markdown(section_name, directive_name)?;

        // Look for the value in bullet list format: - **value**: description
        // or: - **value** (default): description
        let value_lower = value.to_lowercase();

        for line in directive_doc.lines() {
            let line_trimmed = line.trim();
            if !line_trimmed.starts_with("- **") {
                continue;
            }

            // Extract the value name from the bullet: - **value**: or - **value** (default):
            let after_bullets = line_trimmed.trim_start_matches("- **");

            // Find where the value name ends (could be **: or ** or **:)
            let value_end = after_bullets.find("**").unwrap_or(0);
            if value_end == 0 {
                continue;
            }

            let documented_value = &after_bullets[..value_end];

            // Check if this matches our value (case-insensitive, ignoring (default) etc)
            if documented_value.to_lowercase().starts_with(&value_lower)
                || value_lower.starts_with(&documented_value.to_lowercase())
            {
                // Extract the description after the **: or ):
                let rest = &after_bullets[value_end..];
                if let Some(desc_start) = rest.find(':') {
                    let description = rest[desc_start + 1..].trim();
                    if !description.is_empty() {
                        return Some(description.to_string());
                    }
                }
            }
        }

        None
    }

    // we want this for hover and autocomplete of values
    fn create_value_completion(section: &str, directive: &str, value: &str) -> CompletionItem {
        // Try to get value-specific documentation from markdown
        let documentation =
            if let Some(value_doc) = Self::extract_value_documentation(section, directive, value) {
                Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**{value}**\n\n{value_doc}"),
                })
            } else {
                Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("Valid `{value}` option for `{directive}`"),
                })
            };

        Self::create_completion_item(
            value.to_string(),
            CompletionItemKind::VALUE,
            format!("{directive} value"),
            documentation,
            Some(value.to_string()),
        )
    }

    pub fn get_section_documentation(&self, section_name: &str) -> Option<String> {
        SystemdConstants::section_documentation_for(section_name).map(|description| {
            format!(
                "**[{}] Section**\n\n{}\n\n**Reference:** systemd.{}.5",
                section_name,
                description,
                section_name.to_lowercase()
            )
        })
    }

    pub fn get_directive_documentation(
        &self,
        directive_name: &str,
        section_name: &str,
    ) -> Option<String> {
        Self::extract_directive_from_markdown(section_name, directive_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp_server::ls_types::{Position, Uri};

    #[test]
    fn test_completion_creation() {
        let completion = SystemdCompletion::new();

        // Test that sections are populated
        assert!(!completion.section_completions.is_empty());

        // Test that directive completions exist for main sections
        assert!(completion.directive_completions.contains_key("Unit"));
        assert!(completion.directive_completions.contains_key("Service"));
        assert!(completion.directive_completions.contains_key("Install"));
    }

    #[test]
    fn test_get_completions_returns_results() {
        let completion = SystemdCompletion::new();
        let parser = SystemdParser::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let position = Position::new(0, 0);

        // Add a basic document for testing
        let document_text = "[Unit]\nDescription=Test\n\n[Service]\nType=simple\n";
        parser.update_document(&uri, document_text);

        let result = completion.get_completions(&parser, &uri, &position);

        assert!(result.is_some());
        if let Some(CompletionResponse::Array(items)) = result {
            assert!(!items.is_empty());
            // At global level, should show section completions
            assert!(items.iter().any(|item| item.label == "[Unit]"));
            assert!(items.iter().any(|item| item.label == "[Service]"));
            assert!(items.iter().any(|item| item.label == "[Install]"));
        }
    }

    #[test]
    fn test_completion_item_properties() {
        let completion = SystemdCompletion::new();
        let parser = SystemdParser::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let position = Position::new(0, 0);

        // Add a basic document for testing
        let document_text = "[Unit]\nDescription=Test\n\n[Service]\nType=simple\n";
        parser.update_document(&uri, document_text);

        let result = completion.get_completions(&parser, &uri, &position);

        if let Some(CompletionResponse::Array(items)) = result {
            // Find a section completion
            let section_item = items.iter().find(|item| item.label == "[Unit]").unwrap();
            assert_eq!(section_item.kind, Some(CompletionItemKind::MODULE));
            assert!(section_item.detail.is_some());
            assert!(section_item.documentation.is_some());
        }
    }

    #[test]
    fn test_create_documentation() {
        let doc = SystemdCompletion::create_documentation(
            "Test Title",
            "Test description",
            "test.reference",
        );

        if let Documentation::MarkupContent(content) = doc {
            assert_eq!(content.kind, MarkupKind::Markdown);
            assert!(content.value.contains("**Test Title**"));
            assert!(content.value.contains("Test description"));
            assert!(content.value.contains("test.reference"));
        } else {
            panic!("Expected MarkupContent documentation");
        }
    }

    #[test]
    fn test_create_directive_completion() {
        let completion =
            SystemdCompletion::create_directive_completion("Service", "Type", "Test description");

        assert_eq!(completion.label, "Type");
        assert_eq!(completion.kind, Some(CompletionItemKind::PROPERTY));
        assert_eq!(completion.detail, Some("systemd directive".to_string()));
        assert_eq!(completion.insert_text, Some("Type=".to_string()));
        assert!(completion.documentation.is_some());
    }

    #[test]
    fn test_get_section_documentation() {
        let completion = SystemdCompletion::new();

        let doc = completion.get_section_documentation("Unit");
        assert!(doc.is_some());

        let doc_content = doc.unwrap();
        assert!(doc_content.contains("**[Unit] Section**"));
        assert!(doc_content.contains("**Reference:** systemd.unit.5"));

        // Test non-existent section
        let no_doc = completion.get_section_documentation("NonExistentSection");
        assert!(no_doc.is_none());
    }

    #[test]
    fn test_get_directive_documentation() {
        let completion = SystemdCompletion::new();

        // Test existing detailed documentation
        let desc_doc = completion.get_directive_documentation("description", "Unit");
        assert!(desc_doc.is_some());

        let type_doc = completion.get_directive_documentation("type", "Service");
        assert!(type_doc.is_some());

        // Test case insensitivity
        let desc_doc_upper = completion.get_directive_documentation("DESCRIPTION", "Unit");
        assert!(desc_doc_upper.is_some());
        assert_eq!(desc_doc, desc_doc_upper);

        // Test non-existent documentation
        let no_doc = completion.get_directive_documentation("NonExistent", "Unit");
        assert!(no_doc.is_none());
    }

    #[test]
    fn test_no_duplicate_completions() {
        let completion = SystemdCompletion::new();
        let parser = SystemdParser::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let position = Position::new(0, 0);

        // Add a basic document for testing
        let document_text = "[Unit]\nDescription=Test\n\n[Service]\nType=simple\n";
        parser.update_document(&uri, document_text);

        let result = completion.get_completions(&parser, &uri, &position);

        if let Some(CompletionResponse::Array(items)) = result {
            let mut labels = std::collections::HashSet::new();
            let mut duplicates = Vec::new();

            for item in &items {
                if !labels.insert(&item.label) {
                    duplicates.push(&item.label);
                }
            }

            assert!(
                duplicates.is_empty(),
                "Found duplicate completion labels: {duplicates:?}"
            );
        }
    }

    #[test]
    fn test_context_aware_completions() {
        let completion = SystemdCompletion::new();
        let parser = SystemdParser::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();

        // Test document with sections
        let document_text = "[Unit]\nDescription=Test\n\n[Service]\nType=simple\n";
        parser.update_document(&uri, document_text);

        // Test completion at global level (line 0, before any sections)
        let global_result = completion.get_completions(&parser, &uri, &Position::new(0, 0));
        if let Some(CompletionResponse::Array(items)) = global_result {
            // Should only show section completions at global level
            assert!(items.iter().any(|item| item.label == "[Unit]"));
            assert!(items.iter().any(|item| item.label == "[Service]"));
            // Should not show directives at global level
            assert!(!items.iter().any(|item| item.label == "Description"));
            assert!(!items.iter().any(|item| item.label == "Type"));
        }

        // Test completion inside Unit section (line 1)
        let unit_result = completion.get_completions(&parser, &uri, &Position::new(1, 0));
        if let Some(CompletionResponse::Array(items)) = unit_result {
            // Should only show Unit section directives
            assert!(items.iter().any(|item| item.label == "Description"));
            assert!(items.iter().any(|item| item.label == "Documentation"));
            // Should not show Service-specific directives
            assert!(!items.iter().any(|item| item.label == "Type"));
            assert!(!items.iter().any(|item| item.label == "ExecStart"));
        }

        // Test completion inside Service section (line 4)
        let service_result = completion.get_completions(&parser, &uri, &Position::new(4, 0));
        if let Some(CompletionResponse::Array(items)) = service_result {
            // Should only show Service section directives
            assert!(items.iter().any(|item| item.label == "Type"));
            assert!(items.iter().any(|item| item.label == "ExecStart"));
            // Should not show Unit-specific directives
            assert!(!items.iter().any(|item| item.label == "Documentation"));
        }
    }

    #[test]
    fn test_section_header_completion() {
        let completion = SystemdCompletion::new();
        let parser = SystemdParser::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();

        // Test document with partial section header
        let document_text = "[Un";
        parser.update_document(&uri, document_text);

        // Test completion in the middle of a section header
        let result = completion.get_completions(&parser, &uri, &Position::new(0, 3));
        if let Some(CompletionResponse::Array(items)) = result {
            // Should show section completions when in a section header
            assert!(items.iter().any(|item| item.label == "[Unit]"));
            assert!(items.iter().any(|item| item.label == "[Service]"));
            // Should not show directives in section header context
            assert!(!items.iter().any(|item| item.label == "Description"));
            assert!(!items.iter().any(|item| item.label == "Type"));
        }
    }

    #[test]
    fn test_section_header_completion_replaces_typed_bracket() {
        let completion = SystemdCompletion::new();
        let parser = SystemdParser::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();

        for (document_text, cursor, end) in [("[Se", 3, 3), ("[Se]", 3, 4), ("  [", 3, 3)] {
            parser.update_document(&uri, document_text);
            let Some(CompletionResponse::Array(items)) =
                completion.get_completions(&parser, &uri, &Position::new(0, cursor))
            else {
                panic!("expected section completions for {document_text:?}");
            };
            let service = items
                .iter()
                .find(|item| item.label == "[Service]")
                .expect("[Service] completion");
            let start = document_text.find('[').unwrap() as u32;
            let expected = CompletionTextEdit::Edit(TextEdit::new(
                Range::new(Position::new(0, start), Position::new(0, end)),
                "[Service]".to_string(),
            ));
            assert_eq!(service.text_edit, Some(expected), "for {document_text:?}");
        }
    }

    #[test]
    fn test_value_completions_for_restart_directive() {
        let completion = SystemdCompletion::new();
        let parser = SystemdParser::new();
        let uri = "file:///value-test.service".parse::<Uri>().unwrap();

        let document_text = "[Service]\nRestart=\n";
        parser.update_document(&uri, document_text);

        let cursor = "Restart=".chars().count() as u32;
        let result = completion.get_completions(&parser, &uri, &Position::new(1, cursor));

        if let Some(CompletionResponse::Array(items)) = result {
            assert!(items.iter().any(|item| item.label == "no"));
            assert!(items.iter().any(|item| item.label == "always"));
            assert!(items
                .iter()
                .all(|item| item.kind == Some(CompletionItemKind::VALUE)));
        } else {
            panic!("Expected value completions for Restart directive");
        }
    }

    #[test]
    fn test_value_completions_for_killmode_directive() {
        let completion = SystemdCompletion::new();
        let parser = SystemdParser::new();
        let uri = "file:///killmode-test.service".parse::<Uri>().unwrap();

        parser.update_document(&uri, "[Service]\nKillMode=\n");

        let cursor = "KillMode=".chars().count() as u32;
        let result = completion.get_completions(&parser, &uri, &Position::new(1, cursor));

        let Some(CompletionResponse::Array(items)) = result else {
            panic!("Expected value completions for KillMode directive");
        };
        let mut labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();
        labels.sort_unstable();
        assert_eq!(labels, ["control-group", "mixed", "none", "process"]);
    }

    #[test]
    fn test_value_completions_for_boolean_accept_and_persistent() {
        for (file, content, key) in [
            ("accept-test.socket", "[Socket]\nAccept=\n", "Accept"),
            (
                "persistent-test.timer",
                "[Timer]\nPersistent=\n",
                "Persistent",
            ),
        ] {
            let completion = SystemdCompletion::new();
            let parser = SystemdParser::new();
            let uri = format!("file:///{file}").parse::<Uri>().unwrap();

            parser.update_document(&uri, content);

            let cursor = format!("{key}=").chars().count() as u32;
            let result = completion.get_completions(&parser, &uri, &Position::new(1, cursor));

            let Some(CompletionResponse::Array(items)) = result else {
                panic!("Expected value completions for {key} directive");
            };
            for value in ["yes", "no", "true", "false"] {
                assert!(
                    items.iter().any(|item| item.label == value),
                    "{key}= should offer '{value}'"
                );
            }
        }
    }

    #[test]
    fn test_no_value_completions_for_freeform_directive() {
        let completion = SystemdCompletion::new();
        let parser = SystemdParser::new();
        let uri = "file:///value-none.service".parse::<Uri>().unwrap();

        let document_text = "[Unit]\nDescription=\n";
        parser.update_document(&uri, document_text);

        let cursor = "Description=".chars().count() as u32;
        let result = completion.get_completions(&parser, &uri, &Position::new(1, cursor));

        assert!(
            result.is_none(),
            "Expected no completions for freeform directive value"
        );
    }
}
