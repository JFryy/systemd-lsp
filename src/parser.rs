use dashmap::DashMap;
use log::{debug, trace};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_lsp::lsp_types::{Position, Url};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdUnit {
    pub sections: HashMap<String, SystemdSection>,
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdSection {
    pub name: String,
    pub directives: HashMap<String, SystemdDirective>,
    pub line_range: (u32, u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdDirective {
    pub key: String,
    pub value: String,
    pub line_number: u32,
    pub column_range: (u32, u32),
}

#[derive(Debug)]
pub struct SystemdParser {
    documents: DashMap<Url, SystemdUnit>,
    section_regex: Regex,
    directive_regex: Regex,
}

impl SystemdParser {
    pub fn new() -> Self {
        Self {
            documents: DashMap::new(),
            section_regex: Regex::new(r"^\[([^\]]+)\]$").unwrap(),
            directive_regex: Regex::new(r"^([^=]+)=(.*)$").unwrap(),
        }
    }

    pub fn parse(&self, text: &str) -> SystemdUnit {
        trace!("Parsing systemd unit file ({} characters)", text.len());
        let mut unit = SystemdUnit {
            sections: HashMap::new(),
            raw_text: text.to_string(),
        };

        let mut current_section: Option<String> = None;
        
        for (line_num, line) in text.lines().enumerate() {
            let line_num = line_num as u32;
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(captures) = self.section_regex.captures(trimmed) {
                if let Some(section_name) = current_section.take() {
                    if let Some(section) = unit.sections.get_mut(&section_name) {
                        section.line_range.1 = line_num - 1;
                    }
                }
                
                let section_name = captures[1].to_string();
                current_section = Some(section_name.clone());
                
                unit.sections.insert(
                    section_name.clone(),
                    SystemdSection {
                        name: section_name,
                        directives: HashMap::new(),
                        line_range: (line_num, line_num),
                    },
                );
            } else if let Some(captures) = self.directive_regex.captures(trimmed) {
                if let Some(section_name) = &current_section {
                    let key = captures[1].trim().to_string();
                    let value = captures[2].trim().to_string();
                    
                    let key_start = line.find(&key).unwrap_or(0) as u32;
                    let key_end = key_start + key.len() as u32;
                    
                    let directive = SystemdDirective {
                        key: key.clone(),
                        value,
                        line_number: line_num,
                        column_range: (key_start, key_end),
                    };

                    if let Some(section) = unit.sections.get_mut(section_name) {
                        section.directives.insert(key, directive);
                    }
                }
            }
        }

        if let Some(section_name) = current_section {
            if let Some(section) = unit.sections.get_mut(&section_name) {
                section.line_range.1 = text.lines().count() as u32;
            }
        }

        debug!("Parsed {} sections with {} total directives", 
               unit.sections.len(), 
               unit.sections.values().map(|s| s.directives.len()).sum::<usize>());
        unit
    }

    pub fn update_document(&self, uri: &Url, text: &str) {
        let parsed = self.parse(text);
        self.documents.insert(uri.clone(), parsed);
    }

    pub fn get_parsed_document(&self, uri: &Url) -> Option<SystemdUnit> {
        self.documents.get(uri).map(|entry| entry.clone())
    }

    pub fn get_word_at_position(&self, unit: &SystemdUnit, position: &Position) -> Option<String> {
        for section in unit.sections.values() {
            for directive in section.directives.values() {
                if directive.line_number == position.line
                    && position.character >= directive.column_range.0
                    && position.character <= directive.column_range.1
                {
                    return Some(directive.key.clone());
                }
            }
        }
        None
    }
    
    pub fn get_section_header_at_position(&self, unit: &SystemdUnit, position: &Position) -> Option<String> {
        for section in unit.sections.values() {
            if position.line == section.line_range.0 {
                return Some(section.name.clone());
            }
        }
        None
    }

    pub fn get_section_at_line<'a>(&self, unit: &'a SystemdUnit, line: u32) -> Option<&'a SystemdSection> {
        unit.sections
            .values()
            .find(|section| line >= section.line_range.0 && line <= section.line_range.1)
    }
}