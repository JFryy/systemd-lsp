use crate::parser::{SystemdUnit, SystemdSection};
use crate::constants::SystemdConstants;
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
        let valid_sections: HashSet<&'static str> = SystemdConstants::valid_sections().iter().cloned().collect();
        
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
            diagnostics.push(self.create_value_diagnostic(
                directive,
                "ExecStart cannot be empty".to_string(),
            ));
            return;
        }

        let valid_values = SystemdConstants::valid_values();
        if let Some(values) = valid_values.get(directive.key.as_str()) {
            let value = directive.value.as_str();
            let is_valid = match directive.key.as_str() {
                "StandardOutput" | "StandardError" => {
                    values.iter().any(|&v| value == v || value.starts_with(v))
                }
                _ => values.contains(&value)
            };

            if !is_valid {
                diagnostics.push(self.create_value_diagnostic(
                    directive,
                    format!("Invalid {} value '{}'. Valid values: {}", 
                           directive.key, directive.value, values.join(", ")),
                ));
            }
        }
    }

    fn create_value_diagnostic(&self, directive: &crate::parser::SystemdDirective, message: String) -> Diagnostic {
        let value_start = directive.column_range.1 + 1; // +1 for the '=' character
        Diagnostic {
            range: Range::new(
                Position::new(directive.line_number, value_start),
                Position::new(directive.line_number, value_start + directive.value.len() as u32),
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