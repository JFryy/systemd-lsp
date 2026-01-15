use std::process::Command;
use std::sync::Once;

static INIT: Once = Once::new();

/// Ensure the binary is built before running tests
fn ensure_binary_built() {
    INIT.call_once(|| {
        let status = Command::new("cargo")
            .args(&["build", "--release"])
            .status()
            .expect("Failed to build systemd-lsp");
        assert!(status.success(), "Failed to build binary");
    });
}

/// Helper to run systemd-lsp CLI and capture output
fn run_systemd_lsp(args: &[&str]) -> (String, String, i32) {
    ensure_binary_built();

    let output = Command::new("./target/release/systemd-lsp")
        .args(args)
        .output()
        .expect("Failed to execute systemd-lsp");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    (stdout, stderr, exit_code)
}

#[test]
fn test_cli_example_with_errors() {
    let (stdout, _stderr, exit_code) = run_systemd_lsp(&["examples/example-with-errors.service"]);

    // Should exit with code 1 (errors found)
    assert_eq!(exit_code, 1, "Expected exit code 1 for file with errors");

    // Verify all expected diagnostics are present
    assert!(
        stdout.contains("Invalid Type value 'invalid-type'"),
        "Should detect invalid Type value"
    );
    assert!(
        stdout.contains("ExecStart cannot be empty"),
        "Should detect empty ExecStart"
    );
    assert!(
        stdout.contains("Invalid Restart value 'unknown-policy'"),
        "Should detect invalid Restart value"
    );
    assert!(
        stdout.contains("Unknown directive 'InvalidDirective' in [Service] section"),
        "Should detect unknown directive in Service section"
    );
    assert!(
        stdout.contains("Invalid ProtectSystem value 'invalid'"),
        "Should detect invalid ProtectSystem value"
    );
    assert!(
        stdout.contains("Invalid StandardOutput value 'invalid-stream'"),
        "Should detect invalid StandardOutput value"
    );
    assert!(
        stdout.contains("Invalid NotifyAccess value 'invalid-access'"),
        "Should detect invalid NotifyAccess value"
    );
    assert!(
        stdout.contains("Unknown section: [UnknownSection]"),
        "Should detect unknown section"
    );
    assert!(
        stdout.contains("Unknown directive 'AnotherBadDirective' in [Unit] section"),
        "Should detect unknown directive in Unit section"
    );

    // Verify error/warning labels are present
    assert!(stdout.contains("error"), "Should label errors as 'error'");
    assert!(
        stdout.contains("warning"),
        "Should label warnings as 'warning'"
    );

    // Verify file path is shown
    assert!(
        stdout.contains("example-with-errors.service"),
        "Should show file path"
    );

    // Verify summary line shows issues
    assert!(
        stdout.contains("✗"),
        "Should show failure indicator in summary"
    );
}

#[test]
fn test_cli_valid_service_file() {
    let (stdout, _stderr, exit_code) = run_systemd_lsp(&["examples/example.service"]);

    // Should exit with code 0 (no errors)
    assert_eq!(exit_code, 0, "Expected exit code 0 for valid file");

    // Should show success message
    assert!(stdout.contains("✓"), "Should show success indicator");
    assert!(
        stdout.contains("valid"),
        "Should indicate files are valid"
    );
}

#[test]
fn test_cli_multiple_files() {
    let (stdout, _stderr, exit_code) =
        run_systemd_lsp(&["examples/example.service", "examples/example-with-errors.service"]);

    // Should exit with code 1 (at least one file has errors)
    assert_eq!(
        exit_code, 1,
        "Expected exit code 1 when any file has errors"
    );

    // Should process both files
    assert!(
        stdout.contains("example-with-errors.service"),
        "Should mention file with errors"
    );

    // Summary should show multiple files processed
    assert!(
        stdout.contains("out of 2 total"),
        "Should show total file count"
    );
}

#[test]
fn test_cli_nonexistent_file() {
    let (_stdout, stderr, exit_code) = run_systemd_lsp(&["nonexistent.service"]);

    // Should handle gracefully - exits with 1 because no systemd files found
    assert_eq!(exit_code, 1, "Should exit with error code 1");

    // Should mention the issue
    assert!(
        stderr.contains("No systemd unit files found") || stderr.contains("Error reading"),
        "Should mention no files found or read error"
    );
}

#[test]
fn test_cli_directory_non_recursive() {
    let (stdout, _stderr, exit_code) = run_systemd_lsp(&["examples"]);

    // Should process files in directory
    assert!(
        stdout.contains("example-with-errors.service") || stdout.contains("total"),
        "Should process files in examples directory"
    );

    // Verify it finds errors
    assert_eq!(
        exit_code, 1,
        "Should exit with error code 1 due to example-with-errors.service"
    );
}

#[test]
fn test_cli_directory_recursive() {
    let (stdout, _stderr, _exit_code) = run_systemd_lsp(&["examples", "--recursive"]);

    // Should process files recursively
    assert!(
        stdout.contains("example-with-errors.service") || stdout.contains("total"),
        "Should process files from examples directory recursively"
    );
}

#[test]
fn test_expected_error_count() {
    let (stdout, _stderr, exit_code) = run_systemd_lsp(&["examples/example-with-errors.service"]);

    assert_eq!(exit_code, 1, "Expected exit code 1 for file with errors");

    // Count the number of error/warning lines
    let diagnostic_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains("error") || line.contains("warning") || line.contains("unknown"))
        .collect();

    // We expect 9 diagnostics total (7 errors + 2 warnings + 1 unknown section)
    // This is a regression test - if this number changes, it might indicate
    // a change in diagnostic behavior
    assert!(
        diagnostic_lines.len() >= 9,
        "Expected at least 9 diagnostics, found {}. Diagnostics:\n{}",
        diagnostic_lines.len(),
        diagnostic_lines.join("\n")
    );
}

#[test]
fn test_line_numbers_in_output() {
    let (stdout, _stderr, exit_code) = run_systemd_lsp(&["examples/example-with-errors.service"]);

    assert_eq!(exit_code, 1, "Expected exit code 1");

    // Verify that line and column numbers are shown
    // Format is: path:line:column: severity: message
    // Diagnostic lines are indented with spaces
    let diagnostic_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| {
            // Only match indented diagnostic lines (start with spaces)
            line.starts_with("  ")
                && (line.contains("error") || line.contains("warning") || line.contains("unknown"))
        })
        .collect();

    for line in &diagnostic_lines {
        // Each diagnostic line should have line:column format
        assert!(
            line.matches(':').count() >= 3,
            "Diagnostic line should contain path:line:column:severity format. Got: {}",
            line
        );
    }
}
