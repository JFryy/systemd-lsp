use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use log::{info, debug, trace};
use std::env;

mod parser;
mod diagnostics;
mod completion;
mod constants;
mod formatting;
mod definition;

use parser::SystemdParser;
use diagnostics::SystemdDiagnostics;
use completion::SystemdCompletion;
use formatting::SystemdFormatter;
use definition::SystemdDefinitionProvider;

#[derive(Debug)]
pub struct SystemdLanguageServer {
    client: Client,
    parser: SystemdParser,
    diagnostics: SystemdDiagnostics,
    completion: SystemdCompletion,
    formatter: SystemdFormatter,
    definition_provider: SystemdDefinitionProvider,
}

#[tower_lsp::async_trait]
impl LanguageServer for SystemdLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        info!("LSP initialize request received");
        debug!("Client capabilities: {:?}", params.capabilities);
        
        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::FULL,
            )),
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
        info!("Document opened: {}", uri);
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
        debug!("Document changed: {} (version {})", uri, params.text_document.version);
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
        info!("Document saved: {}", params.text_document.uri);
        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        info!("Document closed: {}", params.text_document.uri);
        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = &params.text_document_position.position;
        
        debug!("Completion request at {}:{} in {}", position.line, position.character, uri);
        
        let result = self.completion.get_completions(uri, position).await;
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
        
        debug!("Hover request at {}:{} in {}", position.line, position.character, uri);
        
        self.client
            .log_message(MessageType::INFO, format!("Hover requested at {}:{}", position.line, position.character))
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
        debug!("Formatting request for {}", uri);
        
        if let Some(document_text) = self.parser.get_document_text(uri) {
            let edits = self.formatter.format_document(uri, &document_text);
            debug!("Generated {} formatting edits", edits.len());
            Ok(Some(edits))
        } else {
            debug!("Document not found for formatting: {}", uri);
            Ok(None)
        }
    }

    async fn range_formatting(&self, params: DocumentRangeFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;
        let range = &params.range;
        debug!("Range formatting request for {} at {:?}", uri, range);
        
        if let Some(document_text) = self.parser.get_document_text(uri) {
            let edits = self.formatter.format_range(uri, &document_text, *range);
            debug!("Generated {} range formatting edits", edits.len());
            Ok(Some(edits))
        } else {
            debug!("Document not found for range formatting: {}", uri);
            Ok(None)
        }
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = &params.text_document_position_params.position;
        
        debug!("Go to definition request at {}:{} in {}", position.line, position.character, uri);
        
        let result = self.definition_provider.get_definition(&self.parser, uri, position).await;
        
        if result.is_some() {
            debug!("Definition found and returned");
        } else {
            debug!("No definition found for this position");
        }
        
        Ok(result)
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
        }
    }

    async fn on_change(&self, params: TextDocumentItem) {
        debug!("Processing document change for {}", params.uri);
        trace!("Document text length: {} characters", params.text.len());
        
        let parsed = self.parser.parse(&params.text);
        debug!("Document parsed, found {} sections", parsed.sections.len());
        
        self.parser.update_document(&params.uri, &params.text);
        self.diagnostics.update(&params.uri, parsed).await;
        
        let diagnostics = self.diagnostics.get_diagnostics(&params.uri).await;
        debug!("Publishing {} diagnostics for {}", diagnostics.len(), params.uri);
        
        self.client
            .publish_diagnostics(params.uri.clone(), diagnostics, Some(params.version))
            .await;
    }

    async fn get_hover_info(&self, uri: &Url, position: &Position) -> Option<Hover> {
        trace!("Getting hover info for {}:{} in {}", position.line, position.character, uri);
        let parsed = self.parser.get_parsed_document(uri)?;
        
        // Check if hovering over a section header specifically
        if let Some(section_name) = self.parser.get_section_header_at_position(&parsed, position) {
            // Use the full embedded documentation for section headers
            let section_key = section_name.to_lowercase();
            if let Some(full_docs) = self.definition_provider.get_embedded_documentation(&section_key) {
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
            let directive_docs = self.get_directive_documentation(&directive_name, &current_section.name);
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
                    value: format!("**systemd {} configuration**\n\nHover over section headers `[{}]` or directive names for detailed documentation.\n\n*Press Ctrl+Space for completions*", section.name.to_lowercase(), section.name),
                }),
                range: None,
            });
        }
        
        None
    }
    
    fn get_section_documentation(&self, section_name: &str) -> Option<String> {
        self.completion.get_section_documentation(section_name)
    }
    
    fn get_directive_documentation(&self, directive_name: &str, section_name: &str) -> Option<String> {
        self.completion.get_directive_documentation(directive_name, section_name)
    }
}

fn setup_logging() {
    let is_tty = atty::is(atty::Stream::Stdin) || atty::is(atty::Stream::Stdout);
    
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
        let log_level = env::var("SYSTEMDLS_LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string());
        
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
    setup_logging();
    
    let is_tty = atty::is(atty::Stream::Stdin) || atty::is(atty::Stream::Stdout);
    
    if is_tty {
        // Terminal mode - show help and exit
        println!("systemdls - Language Server for systemd unit files");
        println!();
        println!("USAGE:");
        println!("    systemdls [OPTIONS]");
        println!();
        println!("This is a Language Server Protocol (LSP) implementation for systemd unit files.");
        println!("It should be run by your editor/IDE via LSP, not directly from the terminal.");
        println!();
        println!("ENVIRONMENT VARIABLES:");
        println!("    SYSTEMDLS_LOG_LEVEL    Set log level (error, warn, info, debug, trace)");
        println!();
        println!("For more information, see: https://github.com/jfryy/systemdls");
        return;
    }

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