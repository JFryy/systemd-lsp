use log::debug;
use std::collections::HashMap;
use std::path::PathBuf;
use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};

use crate::parser::SystemdParser;

#[derive(Debug)]
pub struct SystemdDefinitionProvider {
    documentation: HashMap<String, String>,
    shared_temp_file: Option<PathBuf>,
}

impl SystemdDefinitionProvider {
    pub fn new() -> Self {
        let mut documentation = HashMap::new();

        // Embed all documentation directly in the binary for distribution reliability
        documentation.insert(
            "unit".to_string(),
            include_str!("../docs/unit.md").to_string(),
        );
        documentation.insert(
            "service".to_string(),
            include_str!("../docs/service.md").to_string(),
        );
        documentation.insert(
            "install".to_string(),
            include_str!("../docs/install.md").to_string(),
        );
        documentation.insert(
            "socket".to_string(),
            include_str!("../docs/socket.md").to_string(),
        );
        documentation.insert(
            "timer".to_string(),
            include_str!("../docs/timer.md").to_string(),
        );
        documentation.insert(
            "mount".to_string(),
            include_str!("../docs/mount.md").to_string(),
        );
        documentation.insert(
            "path".to_string(),
            include_str!("../docs/path.md").to_string(),
        );
        documentation.insert(
            "swap".to_string(),
            include_str!("../docs/swap.md").to_string(),
        );

        // Create a single shared temp file for all documentation
        let shared_temp_file = if let Ok(temp_dir) = std::env::temp_dir().canonicalize() {
            let temp_file = temp_dir.join("systemdls-documentation.md");

            // Create the file with initial content if it doesn't exist
            if !temp_file.exists() {
                let initial_content = "# systemd Documentation\n\nSelect a section header and use goto definition to view documentation.\n";
                if std::fs::write(&temp_file, initial_content).is_ok() {
                    debug!("Created shared temp file for documentation");
                    Some(temp_file)
                } else {
                    debug!("Failed to create shared temp file");
                    None
                }
            } else {
                debug!("Reusing existing shared temp file");
                Some(temp_file)
            }
        } else {
            debug!("Failed to get temp directory");
            None
        };

        Self {
            documentation,
            shared_temp_file,
        }
    }

    pub async fn get_definition(
        &self,
        parser: &SystemdParser,
        uri: &Url,
        position: &Position,
    ) -> Option<GotoDefinitionResponse> {
        debug!(
            "Definition request at {}:{} in {}",
            position.line, position.character, uri
        );

        let parsed = parser.get_parsed_document(uri)?;

        debug!(
            "Found parsed document with {} sections",
            parsed.sections.len()
        );
        for (name, section) in &parsed.sections {
            debug!(
                "Section '{}' at lines {}-{}",
                name, section.line_range.0, section.line_range.1
            );
        }

        // Only handle section headers for go-to definition
        if let Some(section_name) = parser.get_section_header_at_position(&parsed, position) {
            debug!("Found section header '{}' at position", section_name);
            return self.get_section_man_page_definition(&section_name).await;
        } else {
            debug!(
                "No section header found at position {}:{}",
                position.line, position.character
            );
        }

        None
    }

    async fn get_section_man_page_definition(
        &self,
        section_name: &str,
    ) -> Option<GotoDefinitionResponse> {
        let section_key = section_name.to_lowercase();

        // Update the shared temp file with the requested section's documentation
        if let Some(temp_file) = &self.shared_temp_file {
            if let Some(content) = self.documentation.get(&section_key) {
                if std::fs::write(temp_file, content).is_ok() {
                    debug!(
                        "Updated shared temp file with {} documentation",
                        section_name
                    );
                    if let Ok(uri) = Url::from_file_path(temp_file) {
                        let location = Location {
                            uri,
                            range: Range {
                                start: Position {
                                    line: 0,
                                    character: 0,
                                },
                                end: Position {
                                    line: 0,
                                    character: 0,
                                },
                            },
                        };
                        return Some(GotoDefinitionResponse::Scalar(location));
                    }
                }
            }
        }

        debug!("No documentation available for section: {}", section_name);
        None
    }

    /// Get embedded documentation for a section
    pub fn get_embedded_documentation(&self, section_key: &str) -> Option<String> {
        self.documentation.get(section_key).cloned()
    }

    /// Clean up temporary documentation files
    pub fn cleanup_temp_files(&self) {
        if let Some(temp_file) = &self.shared_temp_file {
            if temp_file.exists() {
                if let Err(e) = std::fs::remove_file(temp_file) {
                    debug!("Failed to remove shared temp file: {}", e);
                } else {
                    debug!("Cleaned up shared temp file");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::SystemdParser;
    use tower_lsp::lsp_types::{Position, Url};

    #[test]
    fn test_embedded_documentation_exists() {
        let provider = SystemdDefinitionProvider::new();

        // Test that all expected documentation is embedded
        assert!(provider.documentation.contains_key("unit"));
        assert!(provider.documentation.contains_key("service"));
        assert!(provider.documentation.contains_key("install"));
        assert!(provider.documentation.contains_key("socket"));
        assert!(provider.documentation.contains_key("timer"));
        assert!(provider.documentation.contains_key("mount"));
        assert!(provider.documentation.contains_key("path"));
        assert!(provider.documentation.contains_key("swap"));

        // Test content is not empty
        assert!(!provider.documentation["unit"].is_empty());
        assert!(!provider.documentation["service"].is_empty());
        assert!(!provider.documentation["install"].is_empty());

        // Test content contains expected headers
        assert!(provider.documentation["unit"].contains("# [Unit] Section"));
        assert!(provider.documentation["service"].contains("# [Service] Section"));
        assert!(provider.documentation["install"].contains("# [Install] Section"));
    }

    #[tokio::test]
    async fn test_get_definition_for_valid_section() {
        let provider = SystemdDefinitionProvider::new();
        let parser = SystemdParser::new();

        // Create a test systemd file
        let content = "[Unit]\nDescription=Test service\n\n[Service]\nType=simple\n";
        let _parsed = parser.parse(content);
        let uri = Url::parse("file:///test.service").unwrap();

        // Store the parsed document
        parser.update_document(&uri, content);

        // Test definition for Unit section (line 0)
        let position = Position {
            line: 0,
            character: 0,
        };
        let result = provider.get_definition(&parser, &uri, &position).await;
        assert!(result.is_some());

        if let Some(GotoDefinitionResponse::Scalar(location)) = result {
            assert!(location
                .uri
                .to_string()
                .contains("systemdls-documentation.md"));
        }
    }

    #[tokio::test]
    async fn test_get_definition_for_invalid_position() {
        let provider = SystemdDefinitionProvider::new();
        let parser = SystemdParser::new();

        let content = "[Unit]\nDescription=Test service\n";
        let uri = Url::parse("file:///test.service").unwrap();
        parser.update_document(&uri, content);

        // Test position not on a section header (line 1)
        let position = Position {
            line: 1,
            character: 0,
        };
        let result = provider.get_definition(&parser, &uri, &position).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_definition_for_unknown_section() {
        let provider = SystemdDefinitionProvider::new();
        let parser = SystemdParser::new();

        // Create a file with an unknown section type
        let content = "[Unknown]\nSomeDirective=value\n";
        let uri = Url::parse("file:///test.service").unwrap();
        parser.update_document(&uri, content);

        let position = Position {
            line: 0,
            character: 0,
        };
        let result = provider.get_definition(&parser, &uri, &position).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_definition_case_insensitive() {
        let provider = SystemdDefinitionProvider::new();
        let parser = SystemdParser::new();

        // Test with different case variations
        let test_cases = vec!["[UNIT]", "[Unit]", "[unit]"];

        for section_header in test_cases {
            let content = format!("{}\nDescription=Test\n", section_header);
            let uri = Url::parse(&format!("file:///test{}.service", section_header)).unwrap();
            parser.update_document(&uri, &content);

            let position = Position {
                line: 0,
                character: 0,
            };
            let result = provider.get_definition(&parser, &uri, &position).await;
            assert!(
                result.is_some(),
                "Failed for section header: {}",
                section_header
            );
        }
    }

    #[test]
    fn test_documentation_content_quality() {
        let provider = SystemdDefinitionProvider::new();

        // Test that documentation contains useful content
        let unit_docs = &provider.documentation["unit"];
        assert!(unit_docs.contains("Description="));
        assert!(unit_docs.contains("Requires="));
        assert!(unit_docs.contains("After="));

        let service_docs = &provider.documentation["service"];
        assert!(service_docs.contains("Type="));
        assert!(service_docs.contains("ExecStart="));
        assert!(service_docs.contains("Restart="));

        let install_docs = &provider.documentation["install"];
        assert!(install_docs.contains("WantedBy="));
        assert!(install_docs.contains("multi-user.target"));
    }
}
