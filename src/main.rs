use clap::Parser;
use log::{debug, info, trace, warn};
use std::env;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use systemd_lsp::{
    SystemdCompletion, SystemdDefinitionProvider, SystemdDiagnostics, SystemdFormatter,
    SystemdParser, SystemdSemanticTokens,
};
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
pub struct SystemdLanguageServer {
    client: Client,
    parser: SystemdParser,
    diagnostics: SystemdDiagnostics,
    completion: SystemdCompletion,
    formatter: SystemdFormatter,
    definition_provider: SystemdDefinitionProvider,
    semantic_tokens: SystemdSemanticTokens,
}

impl LanguageServer for SystemdLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        info!("LSP initialize request received");
        debug!("Client capabilities: {:?}", params.capabilities);

        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(false),
                trigger_characters: Some(vec!["=".to_string(), "[".to_string()]),
                work_done_progress_options: Default::default(),
                all_commit_characters: None,
                completion_item: None,
            }),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            document_formatting_provider: Some(OneOf::Left(true)),
            document_range_formatting_provider: Some(OneOf::Left(true)),
            definition_provider: Some(OneOf::Left(true)),
            semantic_tokens_provider: Some(
                SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                    work_done_progress_options: Default::default(),
                    legend: SystemdSemanticTokens::legend(),
                    range: Some(false),
                    full: Some(SemanticTokensFullOptions::Bool(true)),
                }),
            ),
            ..ServerCapabilities::default()
        };

        info!("Server capabilities configured");
        debug!("Completion trigger characters: [=, []");
        debug!("Text document sync: FULL");
        debug!("Hover provider: enabled");

        Ok(InitializeResult {
            capabilities,
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("LSP server initialized successfully");
        self.client
            .log_message(MessageType::INFO, "systemdls initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        info!("LSP server shutdown requested");

        // Clean up temporary documentation files
        self.definition_provider.cleanup_temp_files();

        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = &params.text_document.uri;
        info!("Document opened: {uri:?}");
        debug!("Document language: {}", params.text_document.language_id);
        debug!("Document version: {}", params.text_document.version);

        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: params.text_document.text,
            version: params.text_document.version,
            language_id: params.text_document.language_id,
        })
        .await
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        let uri = &params.text_document.uri;
        debug!(
            "Document changed: {:?} (version {})",
            uri, params.text_document.version
        );
        trace!("Content changes: {} items", params.content_changes.len());

        // With full sync the last change carries the whole document.
        let Some(change) = params.content_changes.pop() else {
            warn!(
                "Ignoring change notification with no content for {:?}",
                params.text_document.uri
            );
            return;
        };

        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: change.text,
            version: params.text_document.version,
            language_id: "systemd".to_string(),
        })
        .await
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        info!("Document saved: {:?}", params.text_document.uri);
        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        info!("Document closed: {:?}", params.text_document.uri);
        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = &params.text_document_position.position;

        debug!(
            "Completion request at {}:{} in {:?}",
            position.line, position.character, uri
        );

        let result = self.completion.get_completions(&self.parser, uri, position);
        let count = result.as_ref().map_or(0, |r| match r {
            CompletionResponse::Array(items) => items.len(),
            CompletionResponse::List(list) => list.items.len(),
        });

        debug!("Returning {count} completion items");
        Ok(result)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = &params.text_document_position_params.position;

        debug!(
            "Hover request at {}:{} in {:?}",
            position.line, position.character, uri
        );

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Hover requested at {}:{}",
                    position.line, position.character
                ),
            )
            .await;

        let result = self.get_hover_info(uri, position);

        if result.is_some() {
            debug!("Hover info found and returned");
            self.client
                .log_message(MessageType::INFO, "Hover info found")
                .await;
        } else {
            debug!("No hover info available for this position");
            self.client
                .log_message(MessageType::INFO, "No hover info found")
                .await;
        }

        Ok(result)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;
        debug!("Formatting request for {uri:?}");

        if let Some(unit) = self.parser.get_parsed_document(uri) {
            let edits = self.formatter.format_document(uri, &unit.raw_text);
            debug!("Generated {} formatting edits", edits.len());
            Ok(Some(edits))
        } else {
            debug!("Document not found for formatting: {uri:?}");
            Ok(None)
        }
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;
        let range = &params.range;
        debug!("Range formatting request for {uri:?} at {range:?}");

        if let Some(unit) = self.parser.get_parsed_document(uri) {
            let edits = self.formatter.format_range(uri, &unit.raw_text, *range);
            debug!("Generated {} range formatting edits", edits.len());
            Ok(Some(edits))
        } else {
            debug!("Document not found for range formatting: {uri:?}");
            Ok(None)
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = &params.text_document_position_params.position;

        debug!(
            "Go to definition request at {}:{} in {:?}",
            position.line, position.character, uri
        );

        let result = self
            .definition_provider
            .get_definition(&self.parser, uri, position);

        if result.is_some() {
            debug!("Definition found and returned");
        } else {
            debug!("No definition found for this position");
        }

        Ok(result)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = &params.text_document.uri;
        debug!("Semantic tokens full request for {uri:?}");

        let tokens = self.semantic_tokens.get_semantic_tokens(&self.parser, uri);
        let count = tokens.as_ref().map_or(0, |t| t.data.len());
        debug!("Generated {count} semantic tokens");

        Ok(tokens.map(SemanticTokensResult::Tokens))
    }
}

impl SystemdLanguageServer {
    pub fn new(client: Client) -> Self {
        debug!("Initializing parser, diagnostics, and completion modules");
        Self {
            client,
            parser: SystemdParser::new(),
            diagnostics: SystemdDiagnostics::new(),
            completion: SystemdCompletion::new(),
            formatter: SystemdFormatter::new(),
            definition_provider: SystemdDefinitionProvider::new(),
            semantic_tokens: SystemdSemanticTokens::new(),
        }
    }

    async fn on_change(&self, params: TextDocumentItem) {
        debug!("Processing document change for {:?}", params.uri);
        trace!("Document text length: {} characters", params.text.len());

        let parsed = self.parser.update_document(&params.uri, &params.text);
        debug!("Document parsed, found {} sections", parsed.sections.len());

        self.diagnostics.update(&params.uri, &parsed);
        let diagnostics = self.diagnostics.get_diagnostics(&params.uri);
        debug!(
            "Publishing {} diagnostics for {:?}",
            diagnostics.len(),
            params.uri
        );

        self.client
            .publish_diagnostics(params.uri.clone(), diagnostics, Some(params.version))
            .await;
    }

    fn get_hover_info(&self, uri: &Uri, position: &Position) -> Option<Hover> {
        trace!(
            "Getting hover info for {}:{} in {:?}",
            position.line,
            position.character,
            uri
        );
        let parsed = self.parser.get_parsed_document(uri)?;

        if let Some(section_name) = self
            .parser
            .get_section_header_at_position(&parsed, position)
        {
            if let Some(docs) = self
                .definition_provider
                .get_embedded_documentation(&section_name)
            {
                return Some(Self::markdown_hover(Self::truncate_documentation(docs)));
            }
        }

        if let Some(directive_name) = self.parser.get_word_at_position(&parsed, position) {
            let current_section = self.parser.get_section_at_line(&parsed, position.line)?;
            if let Some(docs) = self
                .completion
                .get_directive_documentation(&directive_name, &current_section.name)
            {
                return Some(Self::markdown_hover(Self::truncate_documentation(&docs)));
            }
        }

        // Generic help for any other line inside a section
        let section = self.parser.get_section_at_line(&parsed, position.line)?;
        Some(Self::markdown_hover(format!(
            "**systemd {} configuration**\n\nHover over section headers `[{}]` or directive names for detailed documentation.",
            section.name.to_lowercase(),
            section.name
        )))
    }

    fn markdown_hover(value: String) -> Hover {
        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value,
            }),
            range: None,
        }
    }

    /// Truncate documentation to a reasonable size for hover display
    fn truncate_documentation(docs: &str) -> String {
        const MAX_LINES: usize = 50;
        const MAX_CHARS: usize = 4000;

        let mut lines = docs.lines();
        let head: Vec<&str> = lines.by_ref().take(MAX_LINES).collect();
        if lines.next().is_none() && docs.len() <= MAX_CHARS {
            return docs.to_string();
        }

        let mut result = head.join("\n");
        if result.len() > MAX_CHARS {
            let mut end = MAX_CHARS;
            while !result.is_char_boundary(end) {
                end -= 1;
            }
            result.truncate(end);
            // Try to truncate at a word boundary
            if let Some(last_space) = result.rfind(' ') {
                result.truncate(last_space);
            }
        }

        // Add ellipsis to indicate truncation
        result.push_str(
            "\n\n---\n*Documentation truncated. Use 'Go to Definition' for full details.*",
        );
        result
    }
}

/// CLI arguments for systemd-lsp
#[derive(Parser, Debug)]
#[command(name = "systemd-lsp")]
#[command(
    author,
    version,
    about = "Language Server Protocol implementation for systemd unit files"
)]
#[command(
    long_about = "Language server for systemd unit files covering diagnostic, formatting, and autocomplete functionality w/ documentation.\n\n\
When run in a terminal with file paths, it validates systemd unit files and reports diagnostics.\n\
When run without a TTY (from an editor), it operates as an LSP server."
)]
struct Cli {
    /// Files or directories to validate (supports .service, .socket, .timer, .target, .mount, .automount, .swap, .path, .slice, .scope)
    #[arg(value_name = "PATH", required = true)]
    paths: Vec<PathBuf>,

    /// Recursively search directories for systemd unit files
    #[arg(
        short,
        long,
        help = "Recursively validate all systemd unit files in directories"
    )]
    recursive: bool,
}

/// Collect systemd unit files from the given paths
fn collect_files(paths: &[PathBuf], recursive: bool) -> std::io::Result<Vec<PathBuf>> {
    // Read max depth once from environment variable
    let max_depth: u32 = match std::env::var("SYSTEMD_LSP_MAX_DEPTH") {
        Ok(val) => val.parse().unwrap_or(20),
        Err(_) => 20,
    };
    collect_files_recursive(paths, recursive, 0, max_depth)
}

/// Collect systemd unit files with depth tracking
fn collect_files_recursive(
    paths: &[PathBuf],
    recursive: bool,
    depth: u32,
    max_depth: u32,
) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // Stop if we've recursed too deep
    if depth > max_depth {
        return Ok(files);
    }

    for path in paths {
        if path.is_file() {
            // Check if it looks like a systemd unit file
            if is_systemd_file(path) {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            if recursive {
                // Walk directory recursively
                for entry in std::fs::read_dir(path)? {
                    let entry = entry?;
                    let entry_path = entry.path();
                    if entry_path.is_file() && is_systemd_file(&entry_path) {
                        files.push(entry_path);
                    } else if entry_path.is_dir() {
                        // Recursively collect from subdirectories with incremented depth
                        files.extend(collect_files_recursive(
                            &[entry_path],
                            true,
                            depth + 1,
                            max_depth,
                        )?);
                    }
                }
            } else {
                // Only check files in this directory
                for entry in std::fs::read_dir(path)? {
                    let entry = entry?;
                    let entry_path = entry.path();
                    if entry_path.is_file() && is_systemd_file(&entry_path) {
                        files.push(entry_path);
                    }
                }
            }
        }
    }

    Ok(files)
}

/// Check if a file is a systemd unit file based on extension
fn is_systemd_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(
            ext.to_str().unwrap_or(""),
            "service"
                | "socket"
                | "timer"
                | "target"
                | "mount"
                | "automount"
                | "swap"
                | "path"
                | "slice"
                | "scope"
        )
    } else {
        false
    }
}

/// Run diagnostics on files in CLI mode
fn run_cli_diagnostics(paths: Vec<PathBuf>, recursive: bool) -> std::io::Result<i32> {
    let files = collect_files(&paths, recursive)?;

    if files.is_empty() {
        eprintln!("No systemd unit files found");
        return Ok(1);
    }

    let parser = SystemdParser::new();
    let diagnostics_engine = SystemdDiagnostics::new();
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut files_with_issues = 0;

    for file_path in &files {
        let content = match std::fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading {}: {}", file_path.display(), e);
                continue;
            }
        };

        let diags = diagnostics_engine.diagnose(&parser.parse(&content));

        if !diags.is_empty() {
            files_with_issues += 1;

            println!("\n{}:", file_path.display());
            for diag in &diags {
                let severity = match diag.severity {
                    Some(DiagnosticSeverity::ERROR) => {
                        total_errors += 1;
                        "error"
                    }
                    Some(DiagnosticSeverity::WARNING) => {
                        total_warnings += 1;
                        "warning"
                    }
                    Some(DiagnosticSeverity::INFORMATION) => "info",
                    Some(DiagnosticSeverity::HINT) => "hint",
                    _ => {
                        total_errors += 1; // Treat unknown severity as error
                        "unknown"
                    }
                };

                println!(
                    "  {}:{}:{}: {}: {}",
                    file_path.display(),
                    diag.range.start.line + 1,
                    diag.range.start.character + 1,
                    severity,
                    diag.message
                );
            }
        }
    }

    println!();
    if total_errors == 0 && total_warnings == 0 {
        println!("✓ All {} files are valid", files.len());
        Ok(0)
    } else if total_errors > 0 {
        println!(
            "✗ Found {} error(s) and {} warning(s) in {} file(s) out of {} total",
            total_errors,
            total_warnings,
            files_with_issues,
            files.len()
        );
        Ok(1)
    } else {
        // Only warnings, no errors
        println!(
            "⚠ Found {} warning(s) in {} file(s) out of {} total",
            total_warnings,
            files_with_issues,
            files.len()
        );
        Ok(0)
    }
}

fn setup_logging() {
    let is_tty = std::io::stdin().is_terminal() || std::io::stdout().is_terminal();
    if is_tty {
        // Running in terminal mode - set up console logging
        let mut builder = env_logger::Builder::from_default_env();
        builder
            .filter_level(log::LevelFilter::Info)
            .format_timestamp_secs()
            .init();

        info!("systemdls running in terminal mode");
        info!("Use --help for usage information");
    } else {
        // Running as LSP server - log to file or stderr
        let log_level = env::var("SYSTEMDLS_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        let level_filter = match log_level.to_lowercase().as_str() {
            "error" => log::LevelFilter::Error,
            "warn" => log::LevelFilter::Warn,
            "info" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            "trace" => log::LevelFilter::Trace,
            _ => log::LevelFilter::Info,
        };

        debug!("Environment log level setting: {log_level}");

        let mut builder = env_logger::Builder::new();
        builder
            .filter_level(level_filter)
            .format_timestamp_millis()
            .target(env_logger::Target::Stderr)
            .init();

        info!("systemdls starting as LSP server");
        info!("Log level: {level_filter}");
    }
}

#[tokio::main]
async fn main() {
    let is_tty = std::io::stdin().is_terminal() || std::io::stdout().is_terminal();

    if std::env::args().len() > 1 || is_tty {
        // Terminal/CLI mode - parse CLI arguments and run diagnostics
        let cli = Cli::parse();

        // Run CLI diagnostics mode
        match run_cli_diagnostics(cli.paths, cli.recursive) {
            Ok(exit_code) => std::process::exit(exit_code),
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    } else {
        // LSP server mode (no args and not a TTY)
        setup_logging();
        info!("Initializing systemd language server components");

        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        debug!("Creating LSP service");
        let (service, socket) = LspService::new(|client| {
            info!("Creating new SystemdLanguageServer instance");
            SystemdLanguageServer::new(client)
        });

        info!("Starting LSP server");
        Server::new(stdin, stdout, socket).serve(service).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Most LSP handler tests require a full integration test setup with a real LSP client.
    // The Client type cannot be easily instantiated in unit tests as it's created internally
    // by tower_lsp_server::LspService. The tests below focus on testing individual components
    // and helper functions that don't require the full LSP infrastructure.

    #[test]
    fn test_semantic_tokens_legend() {
        // Test that semantic tokens legend is properly configured
        let legend = SystemdSemanticTokens::legend();
        assert!(!legend.token_types.is_empty(), "Should have token types");
        assert_eq!(legend.token_types.len(), 2, "Should have 2 token types");
    }

    #[test]
    fn test_parser_initialization() {
        // Test that parser can be created and used independently
        let parser = SystemdParser::new();
        let uri: Uri = "file:///test.service".parse().unwrap();
        let content = "[Unit]\nDescription=Test";

        parser.update_document(&uri, content);
        let parsed = parser.get_parsed_document(&uri);

        assert!(parsed.is_some(), "Should parse and store document");
    }

    #[test]
    fn test_completion_module_initialization() {
        // Test that completion module initializes properly
        let completion = SystemdCompletion::new();

        // Test that it can provide section documentation
        let docs = completion.get_section_documentation("Unit");
        assert!(docs.is_some(), "Should have Unit section documentation");

        let docs = completion.get_section_documentation("Service");
        assert!(docs.is_some(), "Should have Service section documentation");
    }

    #[test]
    fn test_diagnostics_module_initialization() {
        // Test that diagnostics module initializes properly
        let _diagnostics = SystemdDiagnostics::new();
        // If this doesn't panic, the module initialized correctly
    }

    #[test]
    fn test_formatter_module_initialization() {
        // Test that formatter module initializes properly
        let formatter = SystemdFormatter::new();
        let uri: Uri = "file:///test.service".parse().unwrap();
        let content = "[Unit]\nDescription=Test\n\n\n[Service]\nType=simple";

        let edits = formatter.format_document(&uri, content);
        // Formatter should produce edits for the extra blank lines
        assert!(
            !edits.is_empty(),
            "Formatter should produce edits for the extra blank lines"
        );
    }

    #[test]
    fn test_definition_provider_initialization() {
        // Test that definition provider initializes properly
        let provider = SystemdDefinitionProvider::new();

        // Test that it has embedded documentation
        let docs = provider.get_embedded_documentation("unit");
        assert!(
            docs.is_some(),
            "Should have embedded documentation for unit section"
        );
    }

    #[test]
    fn test_truncate_documentation_respects_char_boundaries() {
        // 3-byte characters put byte 4000 in the middle of a character.
        let docs = "\u{2192}".repeat(2000);
        let truncated = SystemdLanguageServer::truncate_documentation(&docs);
        assert!(truncated.starts_with('\u{2192}'));
        assert!(truncated.ends_with("for full details.*"));

        let short = "line\n".repeat(10);
        assert_eq!(SystemdLanguageServer::truncate_documentation(&short), short);

        let long = "line\n".repeat(60);
        let truncated = SystemdLanguageServer::truncate_documentation(&long);
        assert_eq!(truncated.matches("line").count(), 50);
    }

    #[test]
    fn test_integrated_parsing_and_semantics() {
        // Test that parser and semantic tokens work together
        let parser = SystemdParser::new();
        let semantic = SystemdSemanticTokens::new();
        let uri: Uri = "file:///test.service".parse().unwrap();
        let content = "[Service]\nType=simple\nExecStart=/usr/bin/test";

        parser.update_document(&uri, content);
        let tokens = semantic.get_semantic_tokens(&parser, &uri);

        assert!(tokens.is_some(), "Should generate semantic tokens");
        if let Some(tokens) = tokens {
            assert!(!tokens.data.is_empty(), "Should have token data");
        }
    }

    #[test]
    fn test_integrated_parsing_and_diagnostics() {
        // Test that parser and diagnostics work together
        let parser = SystemdParser::new();
        let diagnostics = SystemdDiagnostics::new();
        let uri: Uri = "file:///test.service".parse().unwrap();

        // Test with valid content
        let parsed = parser.update_document(&uri, "[Service]\nType=simple");
        diagnostics.update(&uri, &parsed);

        let diags = diagnostics.get_diagnostics(&uri);
        assert_eq!(diags.len(), 0, "Valid content should have no diagnostics");

        // Test with invalid content
        let parsed = parser.update_document(&uri, "[InvalidSection]\nInvalidDirective=value");
        diagnostics.update(&uri, &parsed);

        let diags = diagnostics.get_diagnostics(&uri);
        assert!(!diags.is_empty(), "Invalid content should have diagnostics");
    }

    #[tokio::test]
    async fn test_did_change_without_content_changes_is_ignored() {
        let (service, _socket) = LspService::new(SystemdLanguageServer::new);
        let uri: Uri = "file:///test.service".parse().unwrap();

        service
            .inner()
            .did_change(DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier::new(uri.clone(), 2),
                content_changes: Vec::new(),
            })
            .await;

        assert!(service.inner().parser.get_parsed_document(&uri).is_none());
    }

    #[test]
    fn test_integrated_parsing_and_completion() {
        // Test that parser and completion work together
        let parser = SystemdParser::new();
        let completion = SystemdCompletion::new();
        let uri: Uri = "file:///test.service".parse().unwrap();
        let content = "[Service]\nType=";

        parser.update_document(&uri, content);

        // Test completion after "Type="
        let completions = completion.get_completions(&parser, &uri, &Position::new(1, 5));
        assert!(completions.is_some(), "Should provide completions");
    }

    #[test]
    fn test_parser_with_multiline_directives() {
        // Test parsing of multi-line directives (important for semantic tokens)
        let parser = SystemdParser::new();
        let uri: Uri = "file:///test.service".parse().unwrap();
        let content = "[Service]\nExecStart=/usr/bin/test \\\n    --flag value \\\n    --another";

        parser.update_document(&uri, content);
        let parsed = parser.get_parsed_document(&uri);

        assert!(parsed.is_some(), "Should parse multi-line directives");
        if let Some(parsed) = parsed {
            let service_section = parsed.sections.get("Service");
            assert!(service_section.is_some(), "Should have Service section");

            if let Some(section) = service_section {
                assert_eq!(
                    section.directives.len(),
                    1,
                    "Multi-line directive should be one directive"
                );

                // The directive should have multiple value spans
                assert!(
                    section.directives[0].value_spans.len() >= 3,
                    "Should have value spans for each line"
                );
            }
        }
    }

    #[test]
    fn test_formatter_preserves_section_structure() {
        // Test that formatter maintains section structure
        let formatter = SystemdFormatter::new();
        let uri: Uri = "file:///test.service".parse().unwrap();
        let content = "[Unit]\nDescription=Test\n[Service]\nType=simple\n[Install]\nWantedBy=multi-user.target";

        let _edits = formatter.format_document(&uri, content);

        // Formatter should add blank lines between sections
        // If we got here without panic, the formatter works
    }

    #[test]
    fn test_constants_validation() {
        // Test that constants module has valid data
        let sections = systemd_lsp::SystemdConstants::valid_sections();
        assert!(!sections.is_empty(), "Should have valid sections");
        assert!(sections.contains(&"Unit"), "Should include Unit section");
        assert!(
            sections.contains(&"Service"),
            "Should include Service section"
        );

        // Verify section directives exist
        let section_directives = systemd_lsp::SystemdConstants::section_directives();
        assert!(
            !section_directives.is_empty(),
            "Should have section directives"
        );
        assert!(
            section_directives.contains_key("Unit"),
            "Should have Unit directives"
        );
        assert!(
            section_directives.contains_key("Service"),
            "Should have Service directives"
        );
    }

    // Additional integration-style tests that verify components work together

    #[test]
    fn test_end_to_end_document_processing() {
        // Simulate processing a document through multiple components
        let parser = SystemdParser::new();
        let formatter = SystemdFormatter::new();
        let semantic = SystemdSemanticTokens::new();
        let uri: Uri = "file:///test.service".parse().unwrap();

        // Original content with formatting issues
        let content = "[Unit]\nDescription=Test\n\n\n[Service]\nType=simple";

        // Parse it
        parser.update_document(&uri, content);
        let parsed = parser.get_parsed_document(&uri);
        assert!(parsed.is_some(), "Document should be parsed");

        // Format it
        let _edits = formatter.format_document(&uri, content);

        // Generate semantic tokens
        let tokens = semantic.get_semantic_tokens(&parser, &uri);
        assert!(tokens.is_some(), "Should generate semantic tokens");
    }
}
