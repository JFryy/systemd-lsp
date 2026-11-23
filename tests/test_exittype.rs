use systemd_lsp::{SystemdConstants, SystemdDiagnostics, SystemdParser};
use tower_lsp_server::ls_types::Uri;

fn parse_document(content: &str) -> (SystemdParser, Uri) {
    let parser = SystemdParser::new();
    let uri = "file:///test.service".parse::<Uri>().unwrap();
    parser.update_document(&uri, content);
    (parser, uri)
}

#[test]
fn exittype_allowed_values_are_registered() {
    assert_eq!(
        SystemdConstants::valid_values_for_section("Service", "ExitType"),
        Some(&["main", "cgroup"][..])
    );
}

#[test]
fn exittype_invalid_value_produces_diagnostic() {
    let (parser, uri) = parse_document("[Service]\nExitType=bogus\n");
    let unit = parser.get_parsed_document(&uri).unwrap();

    let diagnostics = SystemdDiagnostics::new().diagnose(&unit);

    assert!(
        diagnostics
            .iter()
            .any(|d| d.message == "Invalid ExitType value 'bogus'. Valid values: main, cgroup"),
        "expected an invalid ExitType diagnostic, got {diagnostics:?}"
    );
}

#[test]
fn exittype_valid_values_produce_no_diagnostics() {
    for value in ["main", "cgroup"] {
        let (parser, uri) = parse_document(&format!("[Service]\nExitType={value}\n"));
        let unit = parser.get_parsed_document(&uri).unwrap();

        let diagnostics = SystemdDiagnostics::new().diagnose(&unit);

        assert!(
            diagnostics.is_empty(),
            "ExitType={value} should be valid, got {diagnostics:?}"
        );
    }
}
