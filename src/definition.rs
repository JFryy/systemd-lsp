use log::debug;
use std::path::PathBuf;
use tower_lsp_server::ls_types::{GotoDefinitionResponse, Location, Position, Range, Uri};

use crate::constants::SystemdConstants;
use crate::parser::SystemdParser;

#[derive(Debug)]
// Directory holding one documentation file per section, private to this process
pub struct SystemdDefinitionProvider {
    docs_dir: Option<PathBuf>,
}

impl Default for SystemdDefinitionProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemdDefinitionProvider {
    pub fn new() -> Self {
        // Include the process ID so that several running servers don't share files
        let docs_dir = match std::env::temp_dir().canonicalize() {
            Ok(temp_dir) => Some(temp_dir.join(format!("systemd-lsp-{}", std::process::id()))),
            Err(e) => {
                debug!("Failed to get temp directory: {e}");
                None
            }
        };

        Self { docs_dir }
    }

    #[cfg(test)]
    fn with_docs_dir(docs_dir: PathBuf) -> Self {
        Self {
            docs_dir: Some(docs_dir),
        }
    }

    pub fn get_definition(
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
            debug!("Section '{}' at lines {:?}", name, section.line_ranges);
        }

        // Only handle section headers for go-to definition
        if let Some(section_name) = parser.get_section_header_at_position(&parsed, position) {
            debug!("Found section header '{section_name}' at position");
            return self.get_section_man_page_definition(&section_name);
        } else {
            debug!(
                "No section header found at position {}:{}",
                position.line, position.character
            );
        }

        None
    }

    fn get_section_man_page_definition(
        &self,
        section_name: &str,
    ) -> Option<GotoDefinitionResponse> {
        let Some(content) = SystemdConstants::section_documentation_for(section_name) else {
            debug!("No documentation available for section: {section_name}");
            return None;
        };
        let docs_dir = self.docs_dir.as_ref()?;

        // Section names only reach this point if they match a known section,
        // so the lowercased name is safe to use as a file name.
        let doc_file = docs_dir.join(format!("{}.md", section_name.to_ascii_lowercase()));

        if !doc_file.exists() {
            if let Err(e) =
                std::fs::create_dir_all(docs_dir).and_then(|()| std::fs::write(&doc_file, content))
            {
                debug!("Failed to write {section_name} documentation: {e}");
                return None;
            }
            debug!("Wrote {section_name} documentation to {doc_file:?}");
        }

        let uri = Uri::from_file_path(&doc_file)?;
        Some(GotoDefinitionResponse::Scalar(Location::new(
            uri,
            Range::default(),
        )))
    }

    /// Get embedded documentation for a section
    pub fn get_embedded_documentation(&self, section_key: &str) -> Option<&'static str> {
        SystemdConstants::section_documentation_for(section_key)
    }

    /// Clean up temporary documentation files
    pub fn cleanup_temp_files(&self) {
        if let Some(docs_dir) = &self.docs_dir {
            if docs_dir.exists() {
                if let Err(e) = std::fs::remove_dir_all(docs_dir) {
                    debug!("Failed to remove documentation directory: {e}");
                } else {
                    debug!("Cleaned up documentation directory");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::SystemdParser;
    use tower_lsp_server::ls_types::{Position, Uri};

    #[test]
    fn test_embedded_documentation_exists() {
        let docs = SystemdConstants::section_documentation();

        // Test that all expected documentation is present and not empty
        let expected_sections = vec![
            "Unit",
            "Service",
            "Install",
            "Socket",
            "Timer",
            "Mount",
            "Path",
            "Swap",
            "Automount",
            "Slice",
            "Scope",
        ];

        for section in expected_sections {
            assert!(docs.contains_key(section), "{section} should exist");
            assert!(
                !docs[section].is_empty(),
                "{section} docs should not be empty"
            );
        }
    }

    #[test]
    fn test_get_definition_for_valid_section() {
        let provider = SystemdDefinitionProvider::new();
        let parser = SystemdParser::new();
        let content = "[Unit]\nDescription=Test service\n\n[Service]\nType=simple\n";
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        parser.update_document(&uri, content);

        let result = provider.get_definition(&parser, &uri, &Position::new(0, 0));

        let Some(GotoDefinitionResponse::Scalar(location)) = result else {
            panic!("expected a scalar location, got {result:?}");
        };
        let expected_dir = format!("systemd-lsp-{}", std::process::id());
        assert!(location.uri.to_string().contains(&expected_dir));
        assert!(location.uri.to_string().ends_with("unit.md"));

        provider.cleanup_temp_files();
    }

    fn test_docs_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("systemd-lsp-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn definition_uri(
        provider: &SystemdDefinitionProvider,
        parser: &SystemdParser,
        uri: &Uri,
        line: u32,
    ) -> Uri {
        match provider.get_definition(parser, uri, &Position::new(line, 0)) {
            Some(GotoDefinitionResponse::Scalar(location)) => location.uri,
            other => panic!("expected a scalar location, got {other:?}"),
        }
    }

    fn read_uri(uri: &Uri) -> String {
        std::fs::read_to_string(uri.to_file_path().unwrap()).unwrap()
    }

    #[test]
    fn test_definition_uses_stable_file_per_section() {
        let provider = SystemdDefinitionProvider::with_docs_dir(test_docs_dir("per-section"));
        let parser = SystemdParser::new();
        let content = "[Unit]\nDescription=Test service\n\n[Service]\nType=simple\n";
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        parser.update_document(&uri, content);

        let unit_uri = definition_uri(&provider, &parser, &uri, 0);
        let service_uri = definition_uri(&provider, &parser, &uri, 3);
        assert_ne!(unit_uri, service_uri);

        // Opening another section must not overwrite the first one
        assert!(read_uri(&unit_uri).contains("[Unit]"));
        assert!(read_uri(&service_uri).contains("[Service]"));

        // Later requests return the same URI
        assert_eq!(definition_uri(&provider, &parser, &uri, 0), unit_uri);
        assert_eq!(definition_uri(&provider, &parser, &uri, 3), service_uri);

        provider.cleanup_temp_files();
    }

    #[test]
    fn test_definition_instances_do_not_share_files() {
        let first = SystemdDefinitionProvider::with_docs_dir(test_docs_dir("instance-a"));
        let second = SystemdDefinitionProvider::with_docs_dir(test_docs_dir("instance-b"));
        let parser = SystemdParser::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        parser.update_document(&uri, "[Unit]\nDescription=Test\n");

        let first_uri = definition_uri(&first, &parser, &uri, 0);
        let second_uri = definition_uri(&second, &parser, &uri, 0);
        assert_ne!(first_uri, second_uri);

        // Shutting down one instance leaves the other's files in place
        second.cleanup_temp_files();
        assert!(!second_uri.to_file_path().unwrap().exists());
        assert!(read_uri(&first_uri).contains("[Unit]"));

        first.cleanup_temp_files();
        assert!(!first_uri.to_file_path().unwrap().exists());
    }

    #[test]
    fn test_default_docs_dir_is_per_process() {
        let provider = SystemdDefinitionProvider::new();
        let docs_dir = provider.docs_dir.expect("temp dir should be available");
        assert_eq!(
            docs_dir.file_name().unwrap().to_string_lossy(),
            format!("systemd-lsp-{}", std::process::id())
        );
    }

    #[test]
    fn test_get_definition_for_invalid_position() {
        let provider = SystemdDefinitionProvider::new();
        let parser = SystemdParser::new();

        let content = "[Unit]\nDescription=Test service\n";
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        parser.update_document(&uri, content);

        // Line 1 is not a section header
        let result = provider.get_definition(&parser, &uri, &Position::new(1, 0));
        assert!(result.is_none());
    }

    #[test]
    fn test_get_definition_for_unknown_section() {
        let provider = SystemdDefinitionProvider::new();
        let parser = SystemdParser::new();

        // Create a file with an unknown section type
        let content = "[Unknown]\nSomeDirective=value\n";
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        parser.update_document(&uri, content);

        let result = provider.get_definition(&parser, &uri, &Position::new(0, 0));
        assert!(result.is_none());
    }

    #[test]
    fn test_get_definition_case_insensitive() {
        let provider = SystemdDefinitionProvider::with_docs_dir(test_docs_dir("case-insensitive"));
        let parser = SystemdParser::new();

        // Test with different case variations
        let test_cases = ["[UNIT]", "[Unit]", "[unit]"];

        for (i, section_header) in test_cases.iter().enumerate() {
            let content = format!("{section_header}\nDescription=Test\n");
            let uri = format!("file:///test_{i}.service").parse::<Uri>().unwrap();
            parser.update_document(&uri, &content);

            let result = provider.get_definition(&parser, &uri, &Position::new(0, 0));
            assert!(
                result.is_some(),
                "Failed for section header: {section_header}"
            );
        }

        provider.cleanup_temp_files();
    }

    #[test]
    fn test_documentation_content_quality() {
        let docs = SystemdConstants::section_documentation();

        // Test that documentation contains useful content
        let unit_docs = docs["Unit"];
        assert!(unit_docs.len() > 100, "Unit docs should be substantial");
        assert!(unit_docs.contains("[Unit]"));

        let service_docs = docs["Service"];
        assert!(
            service_docs.len() > 100,
            "Service docs should be substantial"
        );
        assert!(service_docs.contains("[Service]"));

        let install_docs = docs["Install"];
        assert!(
            install_docs.len() > 100,
            "Install docs should be substantial"
        );
        assert!(install_docs.contains("[Install]"));
    }
}
