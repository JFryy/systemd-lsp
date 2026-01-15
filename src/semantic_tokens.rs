use crate::parser::{SystemdParser, SystemdUnit};
use tower_lsp_server::lsp_types::{SemanticToken, SemanticTokens, SemanticTokensLegend, Uri};

const TOKEN_TYPES: &[&str] = &["keyword", "string"];
pub(crate) const TOKEN_TYPE_KEYWORD: u32 = 0;
pub(crate) const TOKEN_TYPE_STRING: u32 = 1;

#[derive(Debug)]
pub struct SystemdSemanticTokens;

#[derive(Debug)]
struct TokenData {
    line: u32,
    start: u32,
    length: u32,
    token_type: u32,
    modifiers: u32,
}

impl SystemdSemanticTokens {
    pub fn new() -> Self {
        Self
    }

    pub fn legend() -> SemanticTokensLegend {
        SemanticTokensLegend {
            token_types: TOKEN_TYPES.iter().map(|t| (*t).into()).collect(),
            token_modifiers: Vec::new(),
        }
    }

    pub fn get_semantic_tokens(&self, parser: &SystemdParser, uri: &Uri) -> Option<SemanticTokens> {
        let unit = parser.get_parsed_document(uri)?;
        let mut tokens = Self::collect_tokens(&unit);

        if tokens.is_empty() {
            return Some(SemanticTokens {
                result_id: None,
                data: Vec::new(),
            });
        }

        tokens.sort_by(|a, b| match a.line.cmp(&b.line) {
            std::cmp::Ordering::Equal => a.start.cmp(&b.start),
            other => other,
        });

        let data = Self::encode_tokens(tokens);
        Some(SemanticTokens {
            result_id: None,
            data,
        })
    }

    fn collect_tokens(unit: &SystemdUnit) -> Vec<TokenData> {
        let mut tokens = Vec::new();

        for section in unit.sections.values() {
            for directive in &section.directives {
                // Highlight directive keys
                if directive.column_range.1 > directive.column_range.0 {
                    tokens.push(TokenData {
                        line: directive.line_number,
                        start: directive.column_range.0,
                        length: directive.column_range.1 - directive.column_range.0,
                        token_type: TOKEN_TYPE_KEYWORD,
                        modifiers: 0,
                    });
                }

                // Highlight directive values across all spans (including multi-line)
                for span in &directive.value_spans {
                    if span.end > span.start {
                        tokens.push(TokenData {
                            line: span.line,
                            start: span.start,
                            length: span.end - span.start,
                            token_type: TOKEN_TYPE_STRING,
                            modifiers: 0,
                        });
                    }
                }
            }
        }

        tokens
    }

    fn encode_tokens(tokens: Vec<TokenData>) -> Vec<SemanticToken> {
        let mut data = Vec::with_capacity(tokens.len());
        let mut previous_line = 0u32;
        let mut previous_start = 0u32;
        let mut first_token = true;

        for token in tokens {
            let line_delta = if first_token {
                token.line
            } else {
                token.line - previous_line
            };

            let start_delta = if first_token || line_delta != 0 {
                token.start
            } else {
                token.start - previous_start
            };

            data.push(SemanticToken {
                delta_line: line_delta,
                delta_start: start_delta,
                length: token.length,
                token_type: token.token_type,
                token_modifiers_bitset: token.modifiers,
            });

            previous_line = token.line;
            previous_start = token.start;
            first_token = false;
        }

        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::SystemdParser;
    use tower_lsp_server::lsp_types::Uri;

    #[test]
    fn test_multiline_execstart_tokens_cover_all_lines() {
        let parser = SystemdParser::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let content =
            "[Service]\nExecStart=/usr/bin/test \\\n    --flag value \\\n    --another-flag\n";

        parser.update_document(&uri, content);
        let semantic = SystemdSemanticTokens::new();
        let tokens = semantic
            .get_semantic_tokens(&parser, &uri)
            .expect("semantic tokens");

        let decoded = decode_tokens(&tokens.data);
        let string_tokens: Vec<_> = decoded
            .iter()
            .filter(|token| token.token_type == TOKEN_TYPE_STRING)
            .collect();

        assert_eq!(string_tokens.len(), 3);
        assert_eq!(string_tokens[0].line, 1);
        assert_eq!(string_tokens[0].start, 10);
        assert_eq!(string_tokens[0].length, 13);

        assert_eq!(string_tokens[1].line, 2);
        assert_eq!(string_tokens[1].start, 4);
        assert_eq!(string_tokens[1].length, 12);

        assert_eq!(string_tokens[2].line, 3);
        assert_eq!(string_tokens[2].start, 4);
        assert_eq!(string_tokens[2].length, 14);
    }

    struct DecodedToken {
        line: u32,
        start: u32,
        length: u32,
        token_type: u32,
        _modifiers: u32,
    }

    fn decode_tokens(data: &[SemanticToken]) -> Vec<DecodedToken> {
        let mut result = Vec::new();
        let mut current_line = 0u32;
        let mut current_start = 0u32;

        for token in data {
            let line_delta = token.delta_line;
            let start_delta = token.delta_start;

            current_line += line_delta;
            if line_delta == 0 {
                current_start += start_delta;
            } else {
                current_start = start_delta;
            }

            result.push(DecodedToken {
                line: current_line,
                start: current_start,
                length: token.length,
                token_type: token.token_type,
                _modifiers: token.token_modifiers_bitset,
            });
        }

        result
    }

    #[test]
    fn test_empty_document() {
        let parser = SystemdParser::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let content = "";

        parser.update_document(&uri, content);
        let semantic = SystemdSemanticTokens::new();
        let tokens = semantic
            .get_semantic_tokens(&parser, &uri)
            .expect("semantic tokens");

        assert_eq!(tokens.data.len(), 0);
    }

    #[test]
    fn test_document_with_only_sections() {
        let parser = SystemdParser::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let content = "[Unit]\n[Service]\n[Install]";

        parser.update_document(&uri, content);
        let semantic = SystemdSemanticTokens::new();
        let tokens = semantic
            .get_semantic_tokens(&parser, &uri)
            .expect("semantic tokens");

        // No directives, so no tokens
        assert_eq!(tokens.data.len(), 0);
    }

    #[test]
    fn test_document_with_only_comments() {
        let parser = SystemdParser::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let content = "# This is a comment\n# Another comment\n";

        parser.update_document(&uri, content);
        let semantic = SystemdSemanticTokens::new();
        let tokens = semantic
            .get_semantic_tokens(&parser, &uri)
            .expect("semantic tokens");

        // Comments are not tokenized
        assert_eq!(tokens.data.len(), 0);
    }

    #[test]
    fn test_single_directive() {
        let parser = SystemdParser::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let content = "[Service]\nDescription=Test Service";

        parser.update_document(&uri, content);
        let semantic = SystemdSemanticTokens::new();
        let tokens = semantic
            .get_semantic_tokens(&parser, &uri)
            .expect("semantic tokens");

        let decoded = decode_tokens(&tokens.data);
        assert_eq!(decoded.len(), 2); // One for key, one for value

        // Check directive key token
        assert_eq!(decoded[0].line, 1);
        assert_eq!(decoded[0].token_type, TOKEN_TYPE_KEYWORD);
        assert_eq!(decoded[0].length, 11); // "Description"

        // Check directive value token
        assert_eq!(decoded[1].line, 1);
        assert_eq!(decoded[1].token_type, TOKEN_TYPE_STRING);
        assert_eq!(decoded[1].length, 12); // "Test Service"
    }

    #[test]
    fn test_multiple_directives_same_section() {
        let parser = SystemdParser::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let content = "[Service]\nType=simple\nExecStart=/bin/test\nRestart=always";

        parser.update_document(&uri, content);
        let semantic = SystemdSemanticTokens::new();
        let tokens = semantic
            .get_semantic_tokens(&parser, &uri)
            .expect("semantic tokens");

        let decoded = decode_tokens(&tokens.data);
        assert_eq!(decoded.len(), 6); // 3 directives × 2 tokens each

        // Verify token types alternate between keyword and string
        assert_eq!(decoded[0].token_type, TOKEN_TYPE_KEYWORD); // Type
        assert_eq!(decoded[1].token_type, TOKEN_TYPE_STRING); // simple
        assert_eq!(decoded[2].token_type, TOKEN_TYPE_KEYWORD); // ExecStart
        assert_eq!(decoded[3].token_type, TOKEN_TYPE_STRING); // /bin/test
        assert_eq!(decoded[4].token_type, TOKEN_TYPE_KEYWORD); // Restart
        assert_eq!(decoded[5].token_type, TOKEN_TYPE_STRING); // always
    }

    #[test]
    fn test_tokens_are_sorted_by_position() {
        let parser = SystemdParser::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let content = "[Unit]\nDescription=Test\n\n[Service]\nType=simple";

        parser.update_document(&uri, content);
        let semantic = SystemdSemanticTokens::new();
        let tokens = semantic
            .get_semantic_tokens(&parser, &uri)
            .expect("semantic tokens");

        let decoded = decode_tokens(&tokens.data);

        // Verify tokens are in ascending order by line
        for i in 1..decoded.len() {
            assert!(
                decoded[i].line >= decoded[i - 1].line,
                "Tokens should be sorted by line number"
            );
            if decoded[i].line == decoded[i - 1].line {
                assert!(
                    decoded[i].start >= decoded[i - 1].start,
                    "Tokens on same line should be sorted by start position"
                );
            }
        }
    }

    #[test]
    fn test_directive_with_empty_value() {
        let parser = SystemdParser::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let content = "[Service]\nExecStart=";

        parser.update_document(&uri, content);
        let semantic = SystemdSemanticTokens::new();
        let tokens = semantic
            .get_semantic_tokens(&parser, &uri)
            .expect("semantic tokens");

        let decoded = decode_tokens(&tokens.data);
        // Should have token for key, but no token for empty value
        let keyword_tokens: Vec<_> = decoded
            .iter()
            .filter(|t| t.token_type == TOKEN_TYPE_KEYWORD)
            .collect();

        assert_eq!(keyword_tokens.len(), 1);
        assert_eq!(keyword_tokens[0].length, 9); // "ExecStart"
    }

    #[test]
    fn test_document_not_found() {
        let parser = SystemdParser::new();
        let uri = "file:///nonexistent.service".parse::<Uri>().unwrap();
        let semantic = SystemdSemanticTokens::new();
        let tokens = semantic.get_semantic_tokens(&parser, &uri);

        assert!(
            tokens.is_none(),
            "Should return None for non-existent document"
        );
    }

    #[test]
    fn test_multiple_sections_with_directives() {
        let parser = SystemdParser::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let content = "[Unit]\nDescription=Test\n\n[Service]\nType=simple\n\n[Install]\nWantedBy=multi-user.target";

        parser.update_document(&uri, content);
        let semantic = SystemdSemanticTokens::new();
        let tokens = semantic
            .get_semantic_tokens(&parser, &uri)
            .expect("semantic tokens");

        let decoded = decode_tokens(&tokens.data);
        assert_eq!(decoded.len(), 6); // 3 directives × 2 tokens each

        // Verify tokens span across multiple sections
        let lines: Vec<_> = decoded.iter().map(|t| t.line).collect();
        assert!(lines.contains(&1)); // Unit section (Description on line 1)
        assert!(lines.contains(&4)); // Service section (Type on line 4)
        assert!(lines.contains(&7)); // Install section (WantedBy on line 7)
    }

    #[test]
    fn test_token_delta_encoding() {
        let parser = SystemdParser::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let content = "[Service]\nType=simple\nExecStart=/bin/test";

        parser.update_document(&uri, content);
        let semantic = SystemdSemanticTokens::new();
        let tokens = semantic
            .get_semantic_tokens(&parser, &uri)
            .expect("semantic tokens");

        // First token should have absolute position
        assert_eq!(tokens.data[0].delta_line, 1); // Line 1

        // Second token on same line should have delta_line = 0
        assert_eq!(tokens.data[1].delta_line, 0);

        // Third token on next line should have delta_line = 1
        assert_eq!(tokens.data[2].delta_line, 1);
    }
}
