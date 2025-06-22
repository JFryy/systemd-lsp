use crate::constants::SystemdConstants;
use log::{debug, trace};
use std::collections::HashMap;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, Documentation, MarkupContent, MarkupKind, Position, Url,
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
                &format!("systemd.{}.5", name.to_lowercase())
            );
            
            section_completions.push(Self::create_completion_item(
                format!("[{}]", name),
                CompletionItemKind::MODULE,
                format!("systemd {} section", name.to_lowercase()),
                documentation,
                Some(format!("[{}]", name))
            ));
        }

        let mut directive_completions = HashMap::new();
        let directive_descriptions = SystemdConstants::directive_descriptions();
        
        for (section, directives) in SystemdConstants::section_directives() {
            let mut completion_items = Vec::new();
            for &directive in directives {
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

    pub async fn get_completions(&self, uri: &Url, position: &Position) -> Option<CompletionResponse> {
        trace!("Generating completions for {}:{} in {}", position.line, position.character, uri);
        
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
        let documentation = Self::create_documentation(
            key,
            description,
            "systemd.service(5), systemd.unit(5)"
        );
        
        Self::create_completion_item(
            key.to_string(),
            CompletionItemKind::PROPERTY,
            "systemd directive".to_string(),
            documentation,
            Some(format!("{}=", key))
        )
    }


    pub fn get_section_documentation(&self, section_name: &str) -> Option<String> {
        SystemdConstants::section_documentation()
            .get(section_name)
            .map(|description| {
                format!(
                    "**[{}] Section**\n\n{}\n\n**Reference:** systemd.{}.5",
                    section_name, description, section_name.to_lowercase()
                )
            })
    }

    pub fn get_directive_documentation(&self, directive_name: &str, section_name: &str) -> Option<String> {
        match (section_name, directive_name) {
            // Unit section directives
            ("Unit", "Description") => Some("**Description**\n\nA human readable name for the unit. This is used by systemd and other UIs as a user-visible label for the unit, so you should make this string as informative and descriptive as possible.\n\n**Reference:** systemd.unit(5)".to_string()),
            ("Unit", "Documentation") => Some("**Documentation**\n\nA space-separated list of URIs referencing documentation for this unit or its configuration. Accepted are only URIs of the types \"http://\", \"https://\", \"file:\", \"info:\", \"man:\".\n\n**Reference:** systemd.unit(5)".to_string()),
            ("Unit", "Wants") => Some("**Wants**\n\nConfigures requirement dependencies on other units. When this unit is activated, the units listed here will be activated as well. If one of the other units gets deactivated or fails, this unit will continue to function.\n\n**Reference:** systemd.unit(5)".to_string()),
            ("Unit", "Requires") => Some("**Requires**\n\nConfigures requirement dependencies on other units. When this unit is activated, the units listed here will be activated as well. If one of the other units fails to activate, this unit will also fail.\n\n**Reference:** systemd.unit(5)".to_string()),
            ("Unit", "After") => Some("**After**\n\nConfigures ordering dependencies between units. If a unit foo.service contains a setting After=bar.service and both units are being started, bar.service's start-up is ordered before foo.service's start-up.\n\n**Reference:** systemd.unit(5)".to_string()),
            ("Unit", "Before") => Some("**Before**\n\nConfigures ordering dependencies between units. If a unit foo.service contains a setting Before=bar.service and both units are being started, foo.service's start-up is ordered before bar.service's start-up.\n\n**Reference:** systemd.unit(5)".to_string()),
            
            // Service section directives
            ("Service", "Type") => Some("**Type**\n\nConfigures the process start-up type for this service unit. One of:\n- **simple** (default): systemd considers the service to be started up immediately\n- **exec**: like simple, but systemd will wait for exec() in the main service process\n- **forking**: the service calls fork() and the parent exits\n- **oneshot**: similar to simple, but the process must exit before systemd starts follow-up units\n- **dbus**: the service is considered ready when the specified BusName appears on DBus\n- **notify**: the service will issue a notification when it has finished starting up\n- **idle**: similar to simple, but actual execution is delayed until all active jobs are dispatched\n\n**Reference:** systemd.service(5)".to_string()),
            ("Service", "ExecStart") => Some("**ExecStart**\n\nCommands with their arguments that are executed when this service is started. The value is split into zero or more command lines according to the rules described below.\n\nUnless Type= is oneshot, exactly one command must be given. When Type=oneshot is used, zero or more commands may be specified.\n\n**Reference:** systemd.service(5)".to_string()),
            ("Service", "User") => Some("**User**\n\nSet the UNIX user or UID that the processes are executed as. Takes a single user name or UID as argument.\n\nFor system services (services run by the system service manager, i.e. managed by PID 1) and for user services of the root user (services managed by root's instance of systemd --user), the default is \"root\".\n\n**Reference:** systemd.exec(5)".to_string()),
            ("Service", "Group") => Some("**Group**\n\nSet the UNIX user group or GID that the processes are executed as. Takes a single group name or GID as argument. For system services the default is the group of the user specified with User=.\n\n**Reference:** systemd.exec(5)".to_string()),
            ("Service", "WorkingDirectory") => Some("**WorkingDirectory**\n\nTakes a directory path relative to the service's root directory specified by RootDirectory=, or the special value \"~\". Sets the working directory for executed processes.\n\n**Reference:** systemd.exec(5)".to_string()),
            ("Service", "Environment") => Some("**Environment**\n\nSets environment variables for executed processes. Takes a space-separated list of variable assignments. This option may be specified more than once, in which case all specified variables will be set.\n\n**Example:** Environment=\"VAR1=word1 word2\" VAR2=word3 \"VAR3=$word 5 6\"\n\n**Reference:** systemd.exec(5)".to_string()),
            ("Service", "NoNewPrivileges") => Some("**NoNewPrivileges**\n\nTakes a boolean argument. If true, ensures that the service process and all its children can never gain new privileges through execve() (e.g., via setuid or setgid bits, or filesystem capabilities).\n\nThis is the simplest and most effective way to ensure that a process and its children can never elevate privileges again.\n\n**Default:** false\n**Recommended:** true for most services\n\n**Reference:** systemd.exec(5)".to_string()),
            ("Service", "ProtectSystem") => Some("**ProtectSystem**\n\nTakes a boolean argument or the special values \"strict\" or \"full\". If true, mounts the /usr and /boot directories read-only for processes invoked by this unit. If set to \"full\", the /etc directory is mounted read-only, too. If set to \"strict\" the entire file system hierarchy is mounted read-only.\n\n**Values:**\n- **false**: No protection\n- **true**: /usr and /boot read-only\n- **full**: /usr, /boot, and /etc read-only  \n- **strict**: Entire filesystem read-only\n\n**Reference:** systemd.exec(5)".to_string()),
            ("Service", "ProtectHome") => Some("**ProtectHome**\n\nTakes a boolean argument or the special values \"read-only\" or \"tmpfs\". If true, the directories /home, /root, and /run/user are made inaccessible and empty for processes invoked by this unit.\n\n**Values:**\n- **false**: No protection\n- **true**: Home directories inaccessible\n- **read-only**: Home directories read-only\n- **tmpfs**: Mount tmpfs over home directories\n\n**Reference:** systemd.exec(5)".to_string()),
            ("Service", "PrivateTmp") => Some("**PrivateTmp**\n\nTakes a boolean argument. If true, sets up a new file system namespace for the executed processes and mounts private /tmp and /var/tmp directories inside it that is not shared by processes outside of the namespace.\n\nThis is useful to secure access to temporary files of the process, but makes sharing between processes via /tmp or /var/tmp impossible.\n\n**Default:** false\n**Recommended:** true for most services\n\n**Reference:** systemd.exec(5)".to_string()),
            
            // Install section directives  
            ("Install", "WantedBy") => Some("**WantedBy**\n\nThis option may be used more than once, or a space-separated list of unit names may be given. A symbolic link is created in the .wants/ directory of each of the listed units when this unit is enabled.\n\nThis has the effect that when the listed unit is started, this unit will be started too.\n\n**Example:** WantedBy=multi-user.target\n\n**Reference:** systemd.unit(5)".to_string()),
            ("Install", "RequiredBy") => Some("**RequiredBy**\n\nThis option may be used more than once, or a space-separated list of unit names may be given. A symbolic link is created in the .requires/ directory of each of the listed units when this unit is enabled.\n\nThis has the effect that when the listed unit is started, this unit will be started too. Additionally, if this unit fails or is stopped, the listed unit will also be stopped.\n\n**Reference:** systemd.unit(5)".to_string()),
            
            _ => None,
        }
    }
}