use crate::constants::SystemdConstants;
use log::{debug, trace};
use std::collections::HashMap;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, Documentation, MarkupContent,
    MarkupKind, Position, Url,
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
        uri: &Url,
        position: &Position,
    ) -> Option<CompletionResponse> {
        trace!(
            "Generating completions for {}:{} in {}",
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

