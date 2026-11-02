use crate::constants::SystemdConstants;
use crate::parser::{utf16_len, SystemdDirective, SystemdSection, SystemdUnit};
use dashmap::DashMap;
use log::{debug, trace};
use std::collections::{HashMap, HashSet};
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, Position, Range, Uri};

#[derive(Debug)]
pub struct SystemdDiagnostics {
    diagnostics: DashMap<Uri, Vec<Diagnostic>>,
    valid_sections: HashSet<&'static str>,
    section_directives: HashMap<&'static str, HashSet<&'static str>>,
}

impl Default for SystemdDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemdDiagnostics {
    pub fn new() -> Self {
        let section_directives = SystemdConstants::section_directives()
            .iter()
            .map(|(&section, directives)| (section, directives.iter().copied().collect()))
            .collect();

        Self {
            diagnostics: DashMap::new(),
            valid_sections: SystemdConstants::valid_sections().iter().copied().collect(),
            section_directives,
        }
    }

    /// Validates `unit` and returns its diagnostics without storing them.
    pub fn diagnose(&self, unit: &SystemdUnit) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for section in unit.sections.values() {
            self.validate_section(section, &mut diagnostics);
        }
        diagnostics
    }

    pub fn update(&self, uri: &Uri, unit: &SystemdUnit) {
        trace!("Updating diagnostics for {uri:?}");
        let diagnostics = self.diagnose(unit);
        debug!("Generated {} diagnostics for {:?}", diagnostics.len(), uri);
        self.diagnostics.insert(uri.clone(), diagnostics);
    }

    pub fn get_diagnostics(&self, uri: &Uri) -> Vec<Diagnostic> {
        self.diagnostics
            .get(uri)
            .map(|entry| entry.clone())
            .unwrap_or_default()
    }

    fn validate_section(&self, section: &SystemdSection, diagnostics: &mut Vec<Diagnostic>) {
        if !self.valid_sections.contains(section.name.as_str()) {
            let header_width = utf16_len(&section.name) + 2;
            for &(header_line, _) in &section.line_ranges {
                diagnostics.push(Diagnostic {
                    range: Range::new(
                        Position::new(header_line, 0),
                        Position::new(header_line, header_width),
                    ),
                    message: format!("Unknown section: [{}]", section.name),
                    severity: Some(DiagnosticSeverity::WARNING),
                    ..Diagnostic::default()
                });
            }
            return;
        }

        if let Some(valid_directives) = self.section_directives.get(section.name.as_str()) {
            for directive in &section.directives {
                if !valid_directives.contains(directive.key.as_str()) {
                    diagnostics.push(Diagnostic {
                        range: Range::new(
                            Position::new(directive.line_number, directive.column_range.0),
                            Position::new(directive.line_number, directive.column_range.1),
                        ),
                        severity: Some(DiagnosticSeverity::WARNING),
                        source: Some(SystemdConstants::APP_NAME.to_string()),
                        message: format!(
                            "Unknown directive '{}' in [{}] section",
                            directive.key, section.name
                        ),
                        ..Diagnostic::default()
                    });
                }

                self.validate_directive_value(section, directive, diagnostics);
            }
        }
    }

    fn validate_directive_value(
        &self,
        section: &SystemdSection,
        directive: &SystemdDirective,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if section.name == "Service" && directive.key == "ExecStart" && directive.value.is_empty() {
            diagnostics.push(Self::create_value_diagnostic(
                directive,
                "ExecStart cannot be empty".to_string(),
            ));
            return;
        }

        if let Some(values) =
            SystemdConstants::valid_values_for_section(&section.name, &directive.key)
        {
            let value = directive.value.as_str();
            let is_valid = match directive.key.as_str() {
                "StandardOutput" | "StandardError" => {
                    values.iter().any(|&v| value == v || value.starts_with(v))
                }
                _ => values.contains(&value),
            };

            if !is_valid {
                diagnostics.push(Self::create_value_diagnostic(
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

    /// Error spanning the directive's value; an empty value is reported just after the `=`.
    fn create_value_diagnostic(directive: &SystemdDirective, message: String) -> Diagnostic {
        let range = match (directive.value_spans.first(), directive.value_spans.last()) {
            (Some(first), Some(last)) => Range::new(
                Position::new(first.line, first.start),
                Position::new(last.line, last.end),
            ),
            _ => {
                let after_equals =
                    Position::new(directive.line_number, directive.column_range.1 + 1);
                Range::new(after_equals, after_equals)
            }
        };

        Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some(SystemdConstants::APP_NAME.to_string()),
            message,
            ..Diagnostic::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{DirectiveValueSpan, SystemdDirective, SystemdSection};
    use std::collections::HashMap;
    use tower_lsp_server::ls_types::{DiagnosticSeverity, Uri};

    fn create_test_unit(sections: Vec<(&str, Vec<(&str, &str)>)>) -> SystemdUnit {
        let mut unit_sections = HashMap::new();

        for (i, (section_name, directives)) in sections.iter().enumerate() {
            let mut section_directives = Vec::new();

            for (j, (key, value)) in directives.iter().enumerate() {
                let line_number = (i * 10 + j + 1) as u32;
                let value_start = key.len() as u32 + 1;
                let end_column = value_start + value.len() as u32;
                let spans = vec![DirectiveValueSpan {
                    line: line_number,
                    start: value_start,
                    end: end_column,
                }];

                section_directives.push(SystemdDirective {
                    key: key.to_string(),
                    value: value.to_string(),
                    line_number,
                    column_range: (0, key.len() as u32),
                    end_line_number: line_number,
                    value_spans: spans,
                });
            }

            unit_sections.insert(
                section_name.to_string(),
                SystemdSection {
                    name: section_name.to_string(),
                    directives: section_directives,
                    line_ranges: vec![(i as u32, (i + 1) as u32)],
                },
            );
        }

        SystemdUnit {
            sections: unit_sections,
            raw_text: String::new(),
        }
    }

    #[test]
    fn test_valid_unit_no_diagnostics() {
        let unit = create_test_unit(vec![
            ("Unit", vec![("Description", "Test service")]),
            (
                "Service",
                vec![("Type", "simple"), ("ExecStart", "/bin/test")],
            ),
            ("Install", vec![("WantedBy", "multi-user.target")]),
        ]);

        let result = SystemdDiagnostics::new().diagnose(&unit);

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_invalid_section_diagnostic() {
        let unit = create_test_unit(vec![("InvalidSection", vec![("SomeKey", "SomeValue")])]);

        let result = SystemdDiagnostics::new().diagnose(&unit);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].message, "Unknown section: [InvalidSection]");
        assert_eq!(result[0].range.start.line, 0);
    }

    #[test]
    fn test_invalid_directive_diagnostic() {
        let unit = create_test_unit(vec![("Unit", vec![("InvalidDirective", "SomeValue")])]);

        let result = SystemdDiagnostics::new().diagnose(&unit);

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].message,
            "Unknown directive 'InvalidDirective' in [Unit] section"
        );
        assert_eq!(result[0].severity, Some(DiagnosticSeverity::WARNING));
    }

    #[test]
    fn test_empty_execstart_diagnostic() {
        let unit = create_test_unit(vec![("Service", vec![("ExecStart", "")])]);

        let result = SystemdDiagnostics::new().diagnose(&unit);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].message, "ExecStart cannot be empty");
        assert_eq!(result[0].severity, Some(DiagnosticSeverity::ERROR));
    }

    #[test]
    fn test_invalid_type_value_diagnostic() {
        let unit = create_test_unit(vec![("Service", vec![("Type", "invalid_type")])]);

        let result = SystemdDiagnostics::new().diagnose(&unit);

        assert_eq!(result.len(), 1);
        assert!(result[0]
            .message
            .starts_with("Invalid Type value 'invalid_type'"));
        assert_eq!(result[0].severity, Some(DiagnosticSeverity::ERROR));
    }

    #[test]
    fn test_multiple_diagnostics() {
        let unit = create_test_unit(vec![
            ("InvalidSection", vec![("SomeKey", "SomeValue")]),
            ("Unit", vec![("InvalidDirective", "SomeValue")]),
            ("Service", vec![("ExecStart", ""), ("Type", "invalid_type")]),
        ]);

        let result = SystemdDiagnostics::new().diagnose(&unit);

        assert!(result.len() >= 3);
    }

    #[test]
    fn test_diagnostics_persistence() {
        let diagnostics = SystemdDiagnostics::new();
        let uri1 = "file:///test1.service".parse::<Uri>().unwrap();
        let uri2 = "file:///test2.service".parse::<Uri>().unwrap();

        let unit1 = create_test_unit(vec![("InvalidSection", vec![("SomeKey", "SomeValue")])]);
        let unit2 = create_test_unit(vec![("Unit", vec![("Description", "Valid service")])]);

        diagnostics.update(&uri1, &unit1);
        diagnostics.update(&uri2, &unit2);

        let result1 = diagnostics.get_diagnostics(&uri1);
        let result2 = diagnostics.get_diagnostics(&uri2);

        assert_eq!(result1.len(), 1);
        assert_eq!(result2.len(), 0);
    }
}
