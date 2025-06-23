use crate::constants::SystemdConstants;
use crate::parser::{SystemdSection, SystemdUnit};
use dashmap::DashMap;
use log::{debug, trace};
use std::collections::HashSet;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};

#[derive(Debug)]
pub struct SystemdDiagnostics {
    diagnostics: DashMap<Url, Vec<Diagnostic>>,
    valid_sections: HashSet<&'static str>,
    section_directives: DashMap<&'static str, HashSet<&'static str>>,
}

impl SystemdDiagnostics {
    pub fn new() -> Self {
        let valid_sections: HashSet<&'static str> =
            SystemdConstants::valid_sections().iter().cloned().collect();

        let section_directives = DashMap::new();
        for (section, directives) in SystemdConstants::section_directives() {
            let directive_set: HashSet<&'static str> = directives.iter().cloned().collect();
            section_directives.insert(section, directive_set);
        }

        Self {
            diagnostics: DashMap::new(),
            valid_sections,
            section_directives,
        }
    }

    pub async fn update(&self, uri: &Url, unit: SystemdUnit) {
        trace!("Updating diagnostics for {}", uri);
        let mut diagnostics = Vec::new();

        for section in unit.sections.values() {
            self.validate_section(section, &mut diagnostics);
        }

        debug!("Generated {} diagnostics for {}", diagnostics.len(), uri);
        self.diagnostics.insert(uri.clone(), diagnostics);
    }

    pub async fn get_diagnostics(&self, uri: &Url) -> Vec<Diagnostic> {
        self.diagnostics
            .get(uri)
            .map(|entry| entry.clone())
            .unwrap_or_default()
    }

    fn validate_section(&self, section: &SystemdSection, diagnostics: &mut Vec<Diagnostic>) {
        if !self.valid_sections.contains(section.name.as_str()) {
            diagnostics.push(Diagnostic::new_simple(
                Range::new(
                    Position::new(section.line_range.0, 0),
                    Position::new(section.line_range.0, section.name.len() as u32 + 2),
                ),
                format!("Unknown section: [{}]", section.name),
            ));
            return;
        }

        if let Some(valid_directives) = self.section_directives.get(section.name.as_str()) {
            for directive in section.directives.values() {
                if !valid_directives.contains(directive.key.as_str()) {
                    diagnostics.push(Diagnostic {
                        range: Range::new(
                            Position::new(directive.line_number, directive.column_range.0),
                            Position::new(directive.line_number, directive.column_range.1),
                        ),
                        severity: Some(DiagnosticSeverity::WARNING),
                        code: None,
                        code_description: None,
                        source: Some(SystemdConstants::APP_NAME.to_string()),
                        message: format!(
                            "Unknown directive '{}' in [{}] section",
                            directive.key, section.name
                        ),
                        related_information: None,
                        tags: None,
                        data: None,
                    });
                }

                self.validate_directive_value(section, directive, diagnostics);
            }
        }
    }

    fn validate_directive_value(
        &self,
        section: &SystemdSection,
        directive: &crate::parser::SystemdDirective,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if section.name == "Service" && directive.key == "ExecStart" && directive.value.is_empty() {
            diagnostics.push(
                self.create_value_diagnostic(directive, "ExecStart cannot be empty".to_string()),
            );
            return;
        }

        let valid_values = SystemdConstants::valid_values();
        if let Some(values) = valid_values.get(directive.key.as_str()) {
            let value = directive.value.as_str();
            let is_valid = match directive.key.as_str() {
                "StandardOutput" | "StandardError" => {
                    values.iter().any(|&v| value == v || value.starts_with(v))
                }
                _ => values.contains(&value),
            };

            if !is_valid {
                diagnostics.push(self.create_value_diagnostic(
                    directive,
                    format!(
                        "Invalid {} value '{}'. Valid values: {}",
                        directive.key,
                        directive.value,
                        values.join(", ")
                    ),
                ));
            }
        }
    }

    fn create_value_diagnostic(
        &self,
        directive: &crate::parser::SystemdDirective,
        message: String,
    ) -> Diagnostic {
        let value_start = directive.column_range.1 + 1; // +1 for the '=' character
        Diagnostic {
            range: Range::new(
                Position::new(directive.line_number, value_start),
                Position::new(
                    directive.line_number,
                    value_start + directive.value.len() as u32,
                ),
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some(SystemdConstants::APP_NAME.to_string()),
            message,
            related_information: None,
            tags: None,
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{SystemdDirective, SystemdSection};
    use std::collections::HashMap;
    use tower_lsp::lsp_types::{DiagnosticSeverity, Url};

    fn create_test_unit(sections: Vec<(&str, Vec<(&str, &str)>)>) -> SystemdUnit {
        let mut unit_sections = HashMap::new();
        
        for (i, (section_name, directives)) in sections.iter().enumerate() {
            let mut section_directives = HashMap::new();
            
            for (j, (key, value)) in directives.iter().enumerate() {
                section_directives.insert(
                    key.to_string(),
                    SystemdDirective {
                        key: key.to_string(),
                        value: value.to_string(),
                        line_number: (i * 10 + j + 1) as u32,
                        column_range: (0, key.len() as u32),
                    },
                );
            }
            
            unit_sections.insert(
                section_name.to_string(),
                SystemdSection {
                    name: section_name.to_string(),
                    directives: section_directives,
                    line_range: (i as u32, (i + 1) as u32),
                },
            );
        }
        
        SystemdUnit {
            sections: unit_sections,
            raw_text: String::new(),
        }
    }

    #[tokio::test]
    async fn test_valid_unit_no_diagnostics() {
        let diagnostics = SystemdDiagnostics::new();
        let uri = Url::parse("file:///test.service").unwrap();
        
        let unit = create_test_unit(vec![
            ("Unit", vec![("Description", "Test service")]),
            ("Service", vec![("Type", "simple"), ("ExecStart", "/bin/test")]),
            ("Install", vec![("WantedBy", "multi-user.target")]),
        ]);
        
        diagnostics.update(&uri, unit).await;
        let result = diagnostics.get_diagnostics(&uri).await;
        
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_invalid_section_diagnostic() {
        let diagnostics = SystemdDiagnostics::new();
        let uri = Url::parse("file:///test.service").unwrap();
        
        let unit = create_test_unit(vec![
            ("InvalidSection", vec![("SomeKey", "SomeValue")]),
        ]);
        
        diagnostics.update(&uri, unit).await;
        let result = diagnostics.get_diagnostics(&uri).await;
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].message, "Unknown section: [InvalidSection]");
        assert_eq!(result[0].range.start.line, 0);
    }

    #[tokio::test]
    async fn test_invalid_directive_diagnostic() {
        let diagnostics = SystemdDiagnostics::new();
        let uri = Url::parse("file:///test.service").unwrap();
        
        let unit = create_test_unit(vec![
            ("Unit", vec![("InvalidDirective", "SomeValue")]),
        ]);
        
        diagnostics.update(&uri, unit).await;
        let result = diagnostics.get_diagnostics(&uri).await;
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].message, "Unknown directive 'InvalidDirective' in [Unit] section");
        assert_eq!(result[0].severity, Some(DiagnosticSeverity::WARNING));
    }

    #[tokio::test]
    async fn test_empty_execstart_diagnostic() {
        let diagnostics = SystemdDiagnostics::new();
        let uri = Url::parse("file:///test.service").unwrap();
        
        let unit = create_test_unit(vec![
            ("Service", vec![("ExecStart", "")]),
        ]);
        
        diagnostics.update(&uri, unit).await;
        let result = diagnostics.get_diagnostics(&uri).await;
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].message, "ExecStart cannot be empty");
        assert_eq!(result[0].severity, Some(DiagnosticSeverity::ERROR));
    }

    #[tokio::test]
    async fn test_invalid_type_value_diagnostic() {
        let diagnostics = SystemdDiagnostics::new();
        let uri = Url::parse("file:///test.service").unwrap();
        
        let unit = create_test_unit(vec![
            ("Service", vec![("Type", "invalid_type")]),
        ]);
        
        diagnostics.update(&uri, unit).await;
        let result = diagnostics.get_diagnostics(&uri).await;
        
        assert_eq!(result.len(), 1);
        assert!(result[0].message.starts_with("Invalid Type value 'invalid_type'"));
        assert_eq!(result[0].severity, Some(DiagnosticSeverity::ERROR));
    }

    #[tokio::test]
    async fn test_multiple_diagnostics() {
        let diagnostics = SystemdDiagnostics::new();
        let uri = Url::parse("file:///test.service").unwrap();
        
        let unit = create_test_unit(vec![
            ("InvalidSection", vec![("SomeKey", "SomeValue")]),
            ("Unit", vec![("InvalidDirective", "SomeValue")]),
            ("Service", vec![("ExecStart", ""), ("Type", "invalid_type")]),
        ]);
        
        diagnostics.update(&uri, unit).await;
        let result = diagnostics.get_diagnostics(&uri).await;
        
        assert!(result.len() >= 3);
    }

    #[tokio::test]
    async fn test_diagnostics_persistence() {
        let diagnostics = SystemdDiagnostics::new();
        let uri1 = Url::parse("file:///test1.service").unwrap();
        let uri2 = Url::parse("file:///test2.service").unwrap();
        
        let unit1 = create_test_unit(vec![
            ("InvalidSection", vec![("SomeKey", "SomeValue")]),
        ]);
        let unit2 = create_test_unit(vec![
            ("Unit", vec![("Description", "Valid service")]),
        ]);
        
        diagnostics.update(&uri1, unit1).await;
        diagnostics.update(&uri2, unit2).await;
        
        let result1 = diagnostics.get_diagnostics(&uri1).await;
        let result2 = diagnostics.get_diagnostics(&uri2).await;
        
        assert_eq!(result1.len(), 1);
        assert_eq!(result2.len(), 0);
    }
}

