use crate::constants::SystemdConstants;
use log::{debug, trace};
use std::collections::HashMap;
use tower_lsp_server::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, Documentation, MarkupContent,
    MarkupKind, Position, Uri,
};

#[derive(Debug)]
pub struct SystemdCompletion {
    section_completions: Vec<CompletionItem>,
    directive_completions: HashMap<String, Vec<CompletionItem>>,
}

impl SystemdCompletion {
    pub fn new() -> Self {
        let mut section_completions = Vec::new();
        for (name, description) in SystemdConstants::section_documentation() {
            let documentation = Self::create_documentation(
                &format!("[{}] Section", name),
                description,
                &format!("systemd.{}.5", name.to_lowercase()),
            );

            section_completions.push(Self::create_completion_item(
                format!("[{}]", name),
                CompletionItemKind::MODULE,
                format!("systemd {} section", name.to_lowercase()),
                documentation,
                Some(format!("[{}]", name)),
            ));
        }

        let mut directive_completions = HashMap::new();
        let directive_descriptions = SystemdConstants::directive_descriptions();

        for (section, directives) in SystemdConstants::section_directives() {
            let mut completion_items = Vec::new();
            for directive in directives {
                let description = directive_descriptions
                    .get(&(section, directive))
                    .unwrap_or(&"systemd directive")
                    .to_string();
                completion_items.push(Self::create_directive_completion(directive, &description));
            }
            directive_completions.insert(section.to_string(), completion_items);
        }

        Self {
            section_completions,
            directive_completions,
        }
    }

    pub async fn get_completions(
        &self,
        uri: &Uri,
        position: &Position,
    ) -> Option<CompletionResponse> {
        trace!(
            "Generating completions for {}:{} in {:?}",
            position.line,
            position.character,
            uri
        );

        // Return both sections and commonly used directives
        let mut all_completions = Vec::new();
        let mut seen_labels = std::collections::HashSet::new();

        // Add section completions first
        for completion in &self.section_completions {
            if seen_labels.insert(completion.label.clone()) {
                all_completions.push(completion.clone());
            }
        }

        // Add Service section directives (most commonly used)
        if let Some(service_directives) = self.directive_completions.get("Service") {
            for completion in service_directives {
                if seen_labels.insert(completion.label.clone()) {
                    all_completions.push(completion.clone());
                }
            }
        }

        // Add Unit section directives
        if let Some(unit_directives) = self.directive_completions.get("Unit") {
            for completion in unit_directives {
                if seen_labels.insert(completion.label.clone()) {
                    all_completions.push(completion.clone());
                }
            }
        }

        // Add Install section directives
        if let Some(install_directives) = self.directive_completions.get("Install") {
            for completion in install_directives {
                if seen_labels.insert(completion.label.clone()) {
                    all_completions.push(completion.clone());
                }
            }
        }

        // Add other section directives
        for section in &["Timer", "Socket", "Mount", "Path", "Swap"] {
            if let Some(directives) = self.directive_completions.get(*section) {
                for completion in directives {
                    if seen_labels.insert(completion.label.clone()) {
                        all_completions.push(completion.clone());
                    }
                }
            }
        }

        debug!("Generated {} total completion items", all_completions.len());
        Some(CompletionResponse::Array(all_completions))
    }

    fn create_documentation(title: &str, description: &str, reference: &str) -> Documentation {
        Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!(
                "**{}**\n\n{}\n\n---\n*Reference: {}*",
                title, description, reference
            ),
        })
    }

    fn create_completion_item(
        label: String,
        kind: CompletionItemKind,
        detail: String,
        documentation: Documentation,
        insert_text: Option<String>,
    ) -> CompletionItem {
        CompletionItem {
            label,
            label_details: None,
            kind: Some(kind),
            detail: Some(detail),
            documentation: Some(documentation),
            deprecated: None,
            preselect: None,
            sort_text: None,
            filter_text: None,
            insert_text,
            insert_text_format: None,
            insert_text_mode: None,
            text_edit: None,
            additional_text_edits: None,
            command: None,
            commit_characters: None,
            data: None,
            tags: None,
        }
    }

    fn create_directive_completion(key: &str, description: &str) -> CompletionItem {
        let documentation =
            Self::create_documentation(key, description, "systemd.service(5), systemd.unit(5)");

        Self::create_completion_item(
            key.to_string(),
            CompletionItemKind::PROPERTY,
            "systemd directive".to_string(),
            documentation,
            Some(format!("{}=", key)),
        )
    }

    pub fn get_section_documentation(&self, section_name: &str) -> Option<String> {
        SystemdConstants::section_documentation()
            .get(section_name)
            .map(|description| {
                format!(
                    "**[{}] Section**\n\n{}\n\n**Reference:** systemd.{}.5",
                    section_name,
                    description,
                    section_name.to_lowercase()
                )
            })
    }

    pub fn get_directive_documentation(
        &self,
        directive_name: &str,
        section_name: &str,
    ) -> Option<String> {
        match (section_name, directive_name.to_lowercase().as_str()) {
            ("Unit", "description") => {
                Some(include_str!("../docs/directives/unit/description-detailed.txt").to_string())
            }
            ("Service", "type") => {
                Some(include_str!("../docs/directives/service/type-detailed.txt").to_string())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp_server::lsp_types::{Position, Uri};

    #[tokio::test]
    async fn test_completion_creation() {
        let completion = SystemdCompletion::new();

        // Test that sections are populated
        assert!(!completion.section_completions.is_empty());

        // Test that directive completions exist for main sections
        assert!(completion.directive_completions.contains_key("Unit"));
        assert!(completion.directive_completions.contains_key("Service"));
        assert!(completion.directive_completions.contains_key("Install"));
    }

    #[tokio::test]
    async fn test_get_completions_returns_results() {
        let completion = SystemdCompletion::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let position = Position::new(0, 0);

        let result = completion.get_completions(&uri, &position).await;

        assert!(result.is_some());
        if let Some(CompletionResponse::Array(items)) = result {
            assert!(!items.is_empty());

            // Should contain section completions
            assert!(items.iter().any(|item| item.label == "[Unit]"));
            assert!(items.iter().any(|item| item.label == "[Service]"));
            assert!(items.iter().any(|item| item.label == "[Install]"));

            // Should contain common directives
            assert!(items.iter().any(|item| item.label == "Description"));
            assert!(items.iter().any(|item| item.label == "Type"));
            assert!(items.iter().any(|item| item.label == "ExecStart"));
        }
    }

    #[tokio::test]
    async fn test_completion_item_properties() {
        let completion = SystemdCompletion::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let position = Position::new(0, 0);

        let result = completion.get_completions(&uri, &position).await;

        if let Some(CompletionResponse::Array(items)) = result {
            // Find a section completion
            let section_item = items.iter().find(|item| item.label == "[Unit]").unwrap();
            assert_eq!(section_item.kind, Some(CompletionItemKind::MODULE));
            assert!(section_item.detail.is_some());
            assert!(section_item.documentation.is_some());

            // Find a directive completion
            let directive_item = items
                .iter()
                .find(|item| item.label == "Description")
                .unwrap();
            assert_eq!(directive_item.kind, Some(CompletionItemKind::PROPERTY));
            assert!(directive_item.insert_text.is_some());
            assert_eq!(directive_item.insert_text.as_ref().unwrap(), "Description=");
        }
    }

    #[test]
    fn test_create_documentation() {
        let doc = SystemdCompletion::create_documentation(
            "Test Title",
            "Test description",
            "test.reference",
        );

        if let Documentation::MarkupContent(content) = doc {
            assert_eq!(content.kind, MarkupKind::Markdown);
            assert!(content.value.contains("**Test Title**"));
            assert!(content.value.contains("Test description"));
            assert!(content.value.contains("test.reference"));
        } else {
            panic!("Expected MarkupContent documentation");
        }
    }

    #[test]
    fn test_create_directive_completion() {
        let completion =
            SystemdCompletion::create_directive_completion("TestKey", "Test description");

        assert_eq!(completion.label, "TestKey");
        assert_eq!(completion.kind, Some(CompletionItemKind::PROPERTY));
        assert_eq!(completion.detail, Some("systemd directive".to_string()));
        assert_eq!(completion.insert_text, Some("TestKey=".to_string()));
        assert!(completion.documentation.is_some());
    }

    #[test]
    fn test_get_section_documentation() {
        let completion = SystemdCompletion::new();

        let doc = completion.get_section_documentation("Unit");
        assert!(doc.is_some());

        let doc_content = doc.unwrap();
        assert!(doc_content.contains("**[Unit] Section**"));
        assert!(doc_content.contains("**Reference:** systemd.unit.5"));

        // Test non-existent section
        let no_doc = completion.get_section_documentation("NonExistentSection");
        assert!(no_doc.is_none());
    }

    #[test]
    fn test_get_directive_documentation() {
        let completion = SystemdCompletion::new();

        // Test existing detailed documentation
        let desc_doc = completion.get_directive_documentation("description", "Unit");
        assert!(desc_doc.is_some());

        let type_doc = completion.get_directive_documentation("type", "Service");
        assert!(type_doc.is_some());

        // Test case insensitivity
        let desc_doc_upper = completion.get_directive_documentation("DESCRIPTION", "Unit");
        assert!(desc_doc_upper.is_some());
        assert_eq!(desc_doc, desc_doc_upper);

        // Test non-existent documentation
        let no_doc = completion.get_directive_documentation("NonExistent", "Unit");
        assert!(no_doc.is_none());
    }

    #[tokio::test]
    async fn test_no_duplicate_completions() {
        let completion = SystemdCompletion::new();
        let uri = "file:///test.service".parse::<Uri>().unwrap();
        let position = Position::new(0, 0);

        let result = completion.get_completions(&uri, &position).await;

        if let Some(CompletionResponse::Array(items)) = result {
            let mut labels = std::collections::HashSet::new();
            let mut duplicates = Vec::new();

            for item in &items {
                if !labels.insert(&item.label) {
                    duplicates.push(&item.label);
                }
            }

            assert!(
                duplicates.is_empty(),
                "Found duplicate completion labels: {:?}",
                duplicates
            );
        }
    }
}
