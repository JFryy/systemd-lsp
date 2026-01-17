use systemd_lsp::{SystemdCompletion, SystemdConstants, SystemdDefinitionProvider, SystemdParser};
use tower_lsp_server::lsp_types::{Position, Uri};

fn parse_document(content: &str) -> (SystemdParser, Uri) {
    let parser = SystemdParser::new();
    let uri = "file:///test.service".parse::<Uri>().unwrap();
    parser.update_document(&uri, content);
    (parser, uri)
}

#[cfg(test)]
mod hover_tests {
    use super::*;

    #[test]
    fn section_hover_returns_documentation() {
        let provider = SystemdDefinitionProvider::new();
        let sections = ["unit", "service", "install", "socket", "timer"];

        for section in sections {
            let docs = provider.get_embedded_documentation(section);
            assert!(docs.is_some(), "{} should have docs", section);
            assert!(docs.unwrap().len() > 100, "{} docs should be substantial", section);
        }
    }

    #[test]
    fn directive_hover_returns_documentation() {
        let completion = SystemdCompletion::new();
        let directives = [
            ("Unit", "Description"),
            ("Unit", "Wants"),
            ("Service", "Type"),
            ("Service", "ExecStart"),
            ("Install", "WantedBy"),
        ];

        for (section, directive) in directives {
            let docs = completion.get_directive_documentation(directive, section);
            assert!(docs.is_some(), "{}/{} should have docs", section, directive);
            assert!(!docs.unwrap().is_empty());
        }
    }

    #[test]
    fn hover_is_case_insensitive() {
        let completion = SystemdCompletion::new();
        let variants = ["Description", "description", "DESCRIPTION"];

        for variant in variants {
            let docs = completion.get_directive_documentation(variant, "Unit");
            assert!(docs.is_some(), "Case variant '{}' should work", variant);
        }
    }

    #[test]
    fn unknown_directive_returns_none() {
        let completion = SystemdCompletion::new();
        assert!(completion.get_directive_documentation("FakeDirective", "Unit").is_none());
    }

    #[test]
    fn directive_in_wrong_section_returns_none() {
        let completion = SystemdCompletion::new();
        assert!(completion.get_directive_documentation("Type", "Unit").is_none());
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn parser_identifies_section_headers() {
        let (parser, uri) = parse_document("[Unit]\nDescription=Test\n");
        let parsed = parser.get_parsed_document(&uri).unwrap();
        let position = Position { line: 0, character: 2 };

        let section = parser.get_section_header_at_position(&parsed, &position);
        assert_eq!(section.as_deref(), Some("Unit"));
    }

    #[test]
    fn parser_extracts_directive_names() {
        let (parser, uri) = parse_document("[Unit]\nDescription=Test\n");
        let parsed = parser.get_parsed_document(&uri).unwrap();
        let position = Position { line: 1, character: 5 };

        let word = parser.get_word_at_position(&parsed, &position);
        assert_eq!(word.as_deref(), Some("Description"));
    }

    #[test]
    fn parser_tracks_section_context() {
        let (parser, _) = parse_document("[Unit]\nDescription=Test\n\n[Service]\nType=simple\n");
        let parsed = parser.parse("[Unit]\nDescription=Test\n\n[Service]\nType=simple\n");

        let unit_section = parser.get_section_at_line(&parsed, 1).unwrap();
        assert_eq!(unit_section.name, "Unit");

        let service_section = parser.get_section_at_line(&parsed, 4).unwrap();
        assert_eq!(service_section.name, "Service");
    }

    #[test]
    fn parser_stores_directive_positions() {
        let parser = SystemdParser::new();
        let parsed = parser.parse("[Service]\nType=simple\nExecStart=/bin/test\n");

        let section = parsed.sections.get("Service").unwrap();
        assert_eq!(section.directives.len(), 2);
        assert_eq!(section.directives[0].key, "Type");
        assert_eq!(section.directives[0].line_number, 1);
        assert_eq!(section.directives[1].key, "ExecStart");
        assert_eq!(section.directives[1].line_number, 2);
    }
}

#[cfg(test)]
mod documentation_tests {
    use super::*;

    #[test]
    fn all_documented_sections_accessible() {
        let provider = SystemdDefinitionProvider::new();
        let sections = [
            "unit", "service", "install", "socket", "timer", "mount", "path", "swap",
            "container", "pod", "volume", "network", "kube", "build", "image",
        ];

        for section in sections {
            assert!(
                provider.get_embedded_documentation(section).is_some(),
                "Section '{}' should have docs",
                section
            );
        }
    }

    #[test]
    fn directive_descriptions_exist() {
        let descriptions = SystemdConstants::directive_descriptions();
        assert!(descriptions.contains_key(&("Unit", "Description")));
        assert!(descriptions.contains_key(&("Service", "Type")));
        assert!(descriptions.contains_key(&("Install", "WantedBy")));

        for content in descriptions.values() {
            assert!(!content.is_empty(), "Descriptions should not be empty");
        }
    }

    #[test]
    fn section_directives_defined() {
        let directives = SystemdConstants::section_directives();
        assert!(directives.contains_key("Unit"));
        assert!(directives.contains_key("Service"));
        assert!(directives.contains_key("Install"));
        assert!(!directives["Unit"].is_empty());
        assert!(!directives["Service"].is_empty());
    }

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn all_sections_have_documentation() {
        let provider = SystemdDefinitionProvider::new();
        let sections = SystemdConstants::valid_sections();
        let mut missing = Vec::new();

        for section in sections {
            if provider.get_embedded_documentation(&section.to_lowercase()).is_none() {
                missing.push(section);
            }
        }

        let total = SystemdConstants::valid_sections().len();
        let coverage = ((total - missing.len()) as f64 / total as f64) * 100.0;
        println!("\nSection Coverage: {}/{} ({:.1}%)", total - missing.len(), total, coverage);

        if !missing.is_empty() {
            println!("Missing sections:");
            for s in &missing {
                println!("  - {}", s);
            }
            panic!("{} sections missing documentation", missing.len());
        } else {
            println!("✓ All sections have documentation!");
        }
    }

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn all_directives_have_documentation() {
        let completion = SystemdCompletion::new();
        let directives = SystemdConstants::section_directives();
        let mut missing = Vec::new();
        let mut total = 0;

        for (section, dir_list) in directives.iter() {
            for directive in dir_list.iter() {
                total += 1;
                if completion.get_directive_documentation(directive, section).is_none() {
                    missing.push(format!("{}/{}", section, directive));
                }
            }
        }

        let coverage = ((total - missing.len()) as f64 / total as f64) * 100.0;
        println!("\nDirective Coverage: {}/{} ({:.1}%)", total - missing.len(), total, coverage);

        if !missing.is_empty() {
            println!("Missing ({}):", missing.len());
            for d in missing.iter().take(10) {
                println!("  - {}", d);
            }
            if missing.len() > 10 {
                println!("  ... and {} more", missing.len() - 10);
            }
            panic!("{} directives missing documentation", missing.len());
        } else {
            println!("✓ All directives have documentation!");
        }
    }
}
