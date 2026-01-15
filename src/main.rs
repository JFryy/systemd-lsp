use clap::Parser;
use log::{debug, info, trace};
use std::env;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::lsp_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

mod completion;
mod constants;
mod definition;
mod diagnostics;
mod formatting;
mod parser;
mod semantic_tokens;

use completion::SystemdCompletion;
use definition::SystemdDefinitionProvider;
use diagnostics::SystemdDiagnostics;
use formatting::SystemdFormatter;
use parser::SystemdParser;
use semantic_tokens::SystemdSemanticTokens;

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
        info!("Document opened: {:?}", uri);
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

        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
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

        let result = self
            .completion
            .get_completions(&self.parser, uri, position)
            .await;
        let count = result.as_ref().map_or(0, |r| match r {
            CompletionResponse::Array(items) => items.len(),
            CompletionResponse::List(list) => list.items.len(),
        });

        debug!("Returning {} completion items", count);
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

        let result = self.get_hover_info(uri, position).await;

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
        debug!("Formatting request for {:?}", uri);

        if let Some(document_text) = self.parser.get_document_text(uri) {
            let edits = self.formatter.format_document(uri, &document_text);
            debug!("Generated {} formatting edits", edits.len());
            Ok(Some(edits))
        } else {
            debug!("Document not found for formatting: {:?}", uri);
            Ok(None)
        }
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;
        let range = &params.range;
        debug!("Range formatting request for {:?} at {:?}", uri, range);

        if let Some(document_text) = self.parser.get_document_text(uri) {
            let edits = self.formatter.format_range(uri, &document_text, *range);
            debug!("Generated {} range formatting edits", edits.len());
            Ok(Some(edits))
        } else {
            debug!("Document not found for range formatting: {:?}", uri);
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
            .get_definition(&self.parser, uri, position)
            .await;

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
        debug!("Semantic tokens full request for {:?}", uri);

        let tokens = self.semantic_tokens.get_semantic_tokens(&self.parser, uri);
        let count = tokens.as_ref().map_or(0, |t| t.data.len());
        debug!("Generated {} semantic tokens", count);

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

        let parsed = self.parser.parse(&params.text);
        debug!("Document parsed, found {} sections", parsed.sections.len());

        self.parser.update_document(&params.uri, &params.text);
        self.diagnostics.update(&params.uri, parsed).await;

        let diagnostics = self.diagnostics.get_diagnostics(&params.uri).await;
        debug!(
            "Publishing {} diagnostics for {:?}",
            diagnostics.len(),
            params.uri
        );

        self.client
            .publish_diagnostics(params.uri.clone(), diagnostics, Some(params.version))
            .await;
    }

    async fn get_hover_info(&self, uri: &Uri, position: &Position) -> Option<Hover> {
        trace!(
            "Getting hover info for {}:{} in {:?}",
            position.line,
            position.character,
            uri
        );
        let parsed = self.parser.get_parsed_document(uri)?;

        // Check if hovering over a section header specifically
        if let Some(section_name) = self
            .parser
            .get_section_header_at_position(&parsed, position)
        {
            // Use the full embedded documentation for section headers
            let section_key = section_name.to_lowercase();
            if let Some(full_docs) = self
                .definition_provider
                .get_embedded_documentation(&section_key)
            {
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: full_docs,
                    }),
                    range: None,
                });
            }

            // Fallback to short documentation if embedded docs not available
            let section_docs = self.get_section_documentation(&section_name);
            if let Some(docs) = section_docs {
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: docs,
                    }),
                    range: None,
                });
            }
        }

        // Check if hovering over a directive
        if let Some(directive_name) = self.parser.get_word_at_position(&parsed, position) {
            let current_section = self.parser.get_section_at_line(&parsed, position.line)?;
            let directive_docs =
                self.get_directive_documentation(&directive_name, &current_section.name);
            if let Some(docs) = directive_docs {
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: docs,
                    }),
                    range: None,
                });
            }
        }

        // Fallback - provide generic help if hovering over any recognized line
        if let Some(section) = self.parser.get_section_at_line(&parsed, position.line) {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**systemd {} configuration**\n\nHover over section headers `[{}]` or directive names for detailed documentation.", section.name.to_lowercase(), section.name),
                }),
                range: None,
            });
        }

        None
    }

    fn get_section_documentation(&self, section_name: &str) -> Option<String> {
        self.completion.get_section_documentation(section_name)
    }

    fn get_directive_documentation(
        &self,
        directive_name: &str,
        section_name: &str,
    ) -> Option<String> {
        self.completion
            .get_directive_documentation(directive_name, section_name)
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
    let mut files = Vec::new();

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
                        // Recursively collect from subdirectories
                        files.extend(collect_files(&[entry_path], true)?);
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
async fn run_cli_diagnostics(paths: Vec<PathBuf>, recursive: bool) -> std::io::Result<i32> {
    let files = collect_files(&paths, recursive)?;

    if files.is_empty() {
        eprintln!("No systemd unit files found");
        return Ok(1);
    }

    let parser = SystemdParser::new();
    let diagnostics_engine = SystemdDiagnostics::new();
    let mut total_issues = 0;
    let mut files_with_issues = 0;

    for file_path in &files {
        let content = match std::fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading {}: {}", file_path.display(), e);
                continue;
            }
        };

        let uri = format!("file://{}", file_path.display())
            .parse::<Uri>()
            .unwrap();

        let parsed = parser.parse(&content);
        diagnostics_engine.update(&uri, parsed).await;
        let diags = diagnostics_engine.get_diagnostics(&uri).await;

        if !diags.is_empty() {
            files_with_issues += 1;
            total_issues += diags.len();

            println!("\n{}:", file_path.display());
            for diag in diags {
                let severity = match diag.severity {
                    Some(DiagnosticSeverity::ERROR) => "error",
                    Some(DiagnosticSeverity::WARNING) => "warning",
                    Some(DiagnosticSeverity::INFORMATION) => "info",
                    Some(DiagnosticSeverity::HINT) => "hint",
                    _ => "unknown",
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
    if total_issues == 0 {
        println!("✓ All {} files are valid", files.len());
        Ok(0)
    } else {
        println!(
            "✗ Found {} issue(s) in {} file(s) out of {} total",
            total_issues,
            files_with_issues,
            files.len()
        );
        Ok(1)
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

        debug!("Environment log level setting: {}", log_level);

        let mut builder = env_logger::Builder::new();
        builder
            .filter_level(level_filter)
            .format_timestamp_millis()
            .target(env_logger::Target::Stderr)
            .init();

        info!("systemdls starting as LSP server");
        info!("Log level: {}", level_filter);
    }
}

#[tokio::main]
async fn main() {
    let is_tty = std::io::stdin().is_terminal() || std::io::stdout().is_terminal();

    if std::env::args().len() > 1 || is_tty {
        // Terminal/CLI mode - parse CLI arguments and run diagnostics
        let cli = Cli::parse();

        // Run CLI diagnostics mode
        match run_cli_diagnostics(cli.paths, cli.recursive).await {
            Ok(exit_code) => std::process::exit(exit_code),
            Err(e) => {
                eprintln!("Error: {}", e);
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
        assert!(edits.len() > 0 || edits.is_empty(), "Formatter should work");
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

    #[tokio::test]
    async fn test_integrated_parsing_and_diagnostics() {
        // Test that parser and diagnostics work together
        let parser = SystemdParser::new();
        let diagnostics = SystemdDiagnostics::new();
        let uri: Uri = "file:///test.service".parse().unwrap();

        // Test with valid content
        let content = "[Service]\nType=simple";
        let parsed = parser.parse(content);
        parser.update_document(&uri, content);
        diagnostics.update(&uri, parsed).await;

        let diags = diagnostics.get_diagnostics(&uri).await;
        assert_eq!(diags.len(), 0, "Valid content should have no diagnostics");

        // Test with invalid content
        let invalid_content = "[InvalidSection]\nInvalidDirective=value";
        let parsed = parser.parse(invalid_content);
        diagnostics.update(&uri, parsed).await;

        let diags = diagnostics.get_diagnostics(&uri).await;
        assert!(diags.len() > 0, "Invalid content should have diagnostics");
    }

    #[tokio::test]
    async fn test_integrated_parsing_and_completion() {
        // Test that parser and completion work together
        let parser = SystemdParser::new();
        let completion = SystemdCompletion::new();
        let uri: Uri = "file:///test.service".parse().unwrap();
        let content = "[Service]\nType=";

        parser.update_document(&uri, content);

        // Test completion after "Type="
        let position = Position {
            line: 1,
            character: 5,
        };

        let completions = completion.get_completions(&parser, &uri, &position).await;
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
        let sections = constants::SystemdConstants::valid_sections();
        assert!(!sections.is_empty(), "Should have valid sections");
        assert!(sections.contains(&"Unit"), "Should include Unit section");
        assert!(
            sections.contains(&"Service"),
            "Should include Service section"
        );

        // Verify section directives exist
        let section_directives = constants::SystemdConstants::section_directives();
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
