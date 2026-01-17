use log::debug;
use std::path::PathBuf;
use tower_lsp_server::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Uri};
use tower_lsp_server::UriExt;

use crate::constants::SystemdConstants;
use crate::parser::SystemdParser;

#[derive(Debug)]
// this is the shared file for loading the embedded documentation
pub struct SystemdDefinitionProvider {
    shared_temp_file: Option<PathBuf>,
}

impl SystemdDefinitionProvider {
    pub fn new() -> Self {
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

        Self { shared_temp_file }
    }

    pub async fn get_definition(
        &self,
        parser: &SystemdParser,
        uri: &Uri,
        position: &Position,
    ) -> Option<GotoDefinitionResponse> {
        debug!(
            "Definition request at {}:{} in {:?}",
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
        let docs = SystemdConstants::section_documentation();

        // Try to find documentation (case-insensitive)
        let content = docs.iter().find_map(|(key, value)| {
            if key.eq_ignore_ascii_case(section_name) {
                Some(*value)
            } else {
                None
            }
        });

        // Update the shared temp file with the requested section's documentation
        if let Some(temp_file) = &self.shared_temp_file {
            if let Some(content) = content {
                if std::fs::write(temp_file, content).is_ok() {
                    debug!(
                        "Updated shared temp file with {} documentation",
                        section_name
                    );
                    if let Some(uri) = Uri::from_file_path(temp_file) {
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
        let docs = SystemdConstants::section_documentation();
        docs.iter().find_map(|(key, value)| {
            if key.eq_ignore_ascii_case(section_key) {
                Some(value.to_string())
            } else {
                None
            }
        })
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
    use tower_lsp_server::lsp_types::{Position, Uri};

    #[test]
    fn test_embedded_documentation_exists() {
        let docs = SystemdConstants::section_documentation();

        // Test that all expected documentation is present and not empty
        let expected_sections = vec![
            "Unit", "Service", "Install", "Socket", "Timer",
            "Mount", "Path", "Swap", "Automount",
            "Slice", "Scope"
        ];

        for section in expected_sections {
            assert!(docs.contains_key(section), "{} should exist", section);
            assert!(!docs[section].is_empty(), "{} docs should not be empty", section);
        }
    }

    #[tokio::test]
    async fn test_get_definition_for_valid_section() {
        let provider = SystemdDefinitionProvider::new();
        let parser = SystemdParser::new();

        // Create a test systemd file
        let content = "[Unit]\nDescription=Test service\n\n[Service]\nType=simple\n";
        let _parsed = parser.parse(content);
        let uri = "file:///test.service".parse::<Uri>().unwrap();

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
        let uri = "file:///test.service".parse::<Uri>().unwrap();
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
        let uri = "file:///test.service".parse::<Uri>().unwrap();
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
        let test_cases = ["[UNIT]", "[Unit]", "[unit]"];

        for (i, section_header) in test_cases.iter().enumerate() {
            let content = format!("{}\nDescription=Test\n", section_header);
            let uri = format!("file:///test_{}.service", i)
                .parse::<Uri>()
                .unwrap();
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
        let docs = SystemdConstants::section_documentation();

        // Test that documentation contains useful content
        let unit_docs = docs["Unit"];
        assert!(unit_docs.len() > 100, "Unit docs should be substantial");
        assert!(unit_docs.contains("[Unit]"));

        let service_docs = docs["Service"];
        assert!(service_docs.len() > 100, "Service docs should be substantial");
        assert!(service_docs.contains("[Service]"));

        let install_docs = docs["Install"];
        assert!(install_docs.len() > 100, "Install docs should be substantial");
        assert!(install_docs.contains("[Install]"));
    }
}
