//! LSP Navigation Features
//!
//! Advanced navigation with AI-powered go-to-definition, find references, and type navigation

use super::protocol::*;
use crate::core::{HiveError, Result};
use crate::consensus::ConsensusEngine;
use crate::analysis::{AnalysisEngine, CodeElement, SymbolKind};

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use tracing::{debug, error, info, warn};
use std::collections::HashMap;
use std::path::Path;

/// Navigation provider with AI-enhanced capabilities
pub struct NavigationProvider {
    consensus_engine: Arc<RwLock<ConsensusEngine>>,
    analysis_engine: Arc<AnalysisEngine>,
    symbol_index: Arc<RwLock<SymbolIndex>>,
    definition_cache: Arc<RwLock<HashMap<String, Vec<LocationLink>>>>,
    reference_cache: Arc<RwLock<HashMap<String, Vec<Location>>>>,
}

/// Global symbol index for workspace-wide navigation
#[derive(Debug, Default)]
pub struct SymbolIndex {
    /// Symbol definitions by URI
    pub definitions: HashMap<String, Vec<SymbolDefinition>>,
    /// Symbol references by identifier
    pub references: HashMap<String, Vec<SymbolReference>>,
    /// Cross-reference map
    pub cross_references: HashMap<String, Vec<String>>,
    /// Last update timestamp
    pub last_updated: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub struct SymbolDefinition {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
    pub container: Option<String>,
    pub detail: Option<String>,
    pub signature: Option<String>,
    pub documentation: Option<String>,
    pub modifiers: Vec<String>,
    pub scope: SymbolScope,
}

#[derive(Debug, Clone)]
pub struct SymbolReference {
    pub name: String,
    pub location: Location,
    pub context: ReferenceContext,
    pub is_declaration: bool,
    pub is_definition: bool,
    pub usage_type: UsageType,
}

#[derive(Debug, Clone)]
pub enum SymbolScope {
    Global,
    Module,
    Class,
    Function,
    Block,
}

#[derive(Debug, Clone)]
pub enum UsageType {
    Read,
    Write,
    ReadWrite,
    Call,
    Declaration,
    Definition,
    Import,
    Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceContext {
    #[serde(rename = "includeDeclaration")]
    pub include_declaration: bool,
}

impl NavigationProvider {
    /// Create new navigation provider
    pub async fn new(
        consensus_engine: Arc<RwLock<ConsensusEngine>>,
        analysis_engine: Arc<AnalysisEngine>,
    ) -> Result<Self> {
        Ok(Self {
            consensus_engine,
            analysis_engine,
            symbol_index: Arc::new(RwLock::new(SymbolIndex::default())),
            definition_cache: Arc::new(RwLock::new(HashMap::new())),
            reference_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Go to definition with AI-enhanced resolution
    pub async fn go_to_definition(
        &self,
        params: &TextDocumentPositionParams,
        document_content: &str,
        language: &str,
    ) -> Result<Option<GotoDefinitionResponse>> {
        debug!("Finding definition at {}:{}", params.position.line, params.position.character);

        // Get symbol at position
        let symbol_info = self.get_symbol_at_position(
            params,
            document_content,
            language,
        ).await?;

        if let Some(symbol) = symbol_info {
            // Check cache first
            let cache_key = format!("{}:{}", params.text_document.uri, symbol.name);
            {
                let cache = self.definition_cache.read().await;
                if let Some(cached_locations) = cache.get(&cache_key) {
                    if !cached_locations.is_empty() {
                        return Ok(Some(GotoDefinitionResponse::Link(cached_locations.clone())));
                    }
                }
            }

            // Search symbol index
            let definitions = self.find_symbol_definitions(&symbol.name, &symbol.kind).await?;
            
            if definitions.is_empty() {
                // Use AI to help find definition
                return self.ai_assisted_definition_search(&symbol, document_content, language).await;
            }

            // Convert to LocationLink
            let location_links: Vec<LocationLink> = definitions
                .into_iter()
                .map(|def| LocationLink {
                    origin_selection_range: Some(symbol.range.clone()),
                    target_uri: def.location.uri,
                    target_range: def.location.range.clone(),
                    target_selection_range: def.location.range,
                })
                .collect();

            // Cache results
            {
                let mut cache = self.definition_cache.write().await;
                cache.insert(cache_key, location_links.clone());
            }

            if !location_links.is_empty() {
                Ok(Some(GotoDefinitionResponse::Link(location_links)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Find all references with smart filtering
    pub async fn find_references(
        &self,
        params: &ReferenceParams,
        document_content: &str,
        language: &str,
    ) -> Result<Option<Vec<Location>>> {
        debug!("Finding references at {}:{}", params.position.line, params.position.character);

        let position_params = TextDocumentPositionParams {
            text_document: params.text_document.clone(),
            position: params.position.clone(),
        };

        // Get symbol at position
        let symbol_info = self.get_symbol_at_position(
            &position_params,
            document_content,
            language,
        ).await?;

        if let Some(symbol) = symbol_info {
            // Check cache
            let cache_key = format!("{}:{}", params.text_document.uri, symbol.name);
            {
                let cache = self.reference_cache.read().await;
                if let Some(cached_refs) = cache.get(&cache_key) {
                    return Ok(Some(cached_refs.clone()));
                }
            }

            // Search all references
            let mut all_references = Vec::new();
            
            // Find in symbol index
            let references = self.find_symbol_references(&symbol.name, &symbol.kind).await?;
            
            for reference in references {
                if params.context.include_declaration || !reference.is_declaration {
                    all_references.push(reference.location);
                }
            }

            // Use AI for additional context if needed
            if all_references.is_empty() {
                if let Some(ai_refs) = self.ai_assisted_reference_search(&symbol, language).await? {
                    all_references.extend(ai_refs);
                }
            }

            // Cache results
            {
                let mut cache = self.reference_cache.write().await;
                cache.insert(cache_key, all_references.clone());
            }

            Ok(Some(all_references))
        } else {
            Ok(None)
        }
    }

    /// Go to implementation
    pub async fn go_to_implementation(
        &self,
        params: &TextDocumentPositionParams,
        document_content: &str,
        language: &str,
    ) -> Result<Option<GotoImplementationResponse>> {
        debug!("Finding implementation at {}:{}", params.position.line, params.position.character);

        let symbol_info = self.get_symbol_at_position(params, document_content, language).await?;

        if let Some(symbol) = symbol_info {
            // Find implementations for interfaces/abstracts
            let implementations = self.find_implementations(&symbol.name, &symbol.kind).await?;
            
            if !implementations.is_empty() {
                let location_links: Vec<LocationLink> = implementations
                    .into_iter()
                    .map(|impl_def| LocationLink {
                        origin_selection_range: Some(symbol.range.clone()),
                        target_uri: impl_def.location.uri,
                        target_range: impl_def.location.range.clone(),
                        target_selection_range: impl_def.location.range,
                    })
                    .collect();

                Ok(Some(GotoImplementationResponse::Link(location_links)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Go to type definition
    pub async fn go_to_type_definition(
        &self,
        params: &TextDocumentPositionParams,
        document_content: &str,
        language: &str,
    ) -> Result<Option<GotoTypeDefinitionResponse>> {
        debug!("Finding type definition at {}:{}", params.position.line, params.position.character);

        let symbol_info = self.get_symbol_at_position(params, document_content, language).await?;

        if let Some(symbol) = symbol_info {
            // Find type definition
            if let Some(type_name) = self.extract_type_name(&symbol, document_content).await? {
                let type_definitions = self.find_symbol_definitions(&type_name, &SymbolKind::Type).await?;
                
                if !type_definitions.is_empty() {
                    let location_links: Vec<LocationLink> = type_definitions
                        .into_iter()
                        .map(|type_def| LocationLink {
                            origin_selection_range: Some(symbol.range.clone()),
                            target_uri: type_def.location.uri,
                            target_range: type_def.location.range.clone(),
                            target_selection_range: type_def.location.range,
                        })
                        .collect();

                    return Ok(Some(GotoTypeDefinitionResponse::Link(location_links)));
                }
            }
        }

        Ok(None)
    }

    /// Go to declaration
    pub async fn go_to_declaration(
        &self,
        params: &TextDocumentPositionParams,
        document_content: &str,
        language: &str,
    ) -> Result<Option<GotoDeclarationResponse>> {
        debug!("Finding declaration at {}:{}", params.position.line, params.position.character);

        let symbol_info = self.get_symbol_at_position(params, document_content, language).await?;

        if let Some(symbol) = symbol_info {
            // Find declarations (different from definitions)
            let declarations = self.find_symbol_declarations(&symbol.name, &symbol.kind).await?;
            
            if !declarations.is_empty() {
                let location_links: Vec<LocationLink> = declarations
                    .into_iter()
                    .map(|decl| LocationLink {
                        origin_selection_range: Some(symbol.range.clone()),
                        target_uri: decl.location.uri,
                        target_range: decl.location.range.clone(),
                        target_selection_range: decl.location.range,
                    })
                    .collect();

                Ok(Some(GotoDeclarationResponse::Link(location_links)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Update symbol index for a document
    pub async fn update_symbol_index(&self, uri: &str, content: &str, language: &str) -> Result<()> {
        debug!("Updating symbol index for {}", uri);

        // Parse document for symbols
        let parse_result = self.analysis_engine.parse_code(content, Some(language)).await?;
        
        let mut definitions = Vec::new();
        let mut references = Vec::new();

        // Extract symbols from AST
        for element in parse_result.elements {
            match element {
                CodeElement::Function { name, range, signature, .. } => {
                    definitions.push(SymbolDefinition {
                        name: name.clone(),
                        kind: SymbolKind::Function,
                        location: Location {
                            uri: uri.to_string(),
                            range: range.clone(),
                        },
                        container: None,
                        detail: None,
                        signature: Some(signature),
                        documentation: None,
                        modifiers: Vec::new(),
                        scope: SymbolScope::Global,
                    });
                }
                CodeElement::Class { name, range, .. } => {
                    definitions.push(SymbolDefinition {
                        name: name.clone(),
                        kind: SymbolKind::Class,
                        location: Location {
                            uri: uri.to_string(),
                            range: range.clone(),
                        },
                        container: None,
                        detail: None,
                        signature: None,
                        documentation: None,
                        modifiers: Vec::new(),
                        scope: SymbolScope::Global,
                    });
                }
                CodeElement::Variable { name, range, var_type, .. } => {
                    definitions.push(SymbolDefinition {
                        name: name.clone(),
                        kind: SymbolKind::Variable,
                        location: Location {
                            uri: uri.to_string(),
                            range: range.clone(),
                        },
                        container: None,
                        detail: var_type,
                        signature: None,
                        documentation: None,
                        modifiers: Vec::new(),
                        scope: SymbolScope::Function,
                    });
                }
                _ => {} // Handle other element types
            }
        }

        // Update index
        {
            let mut index = self.symbol_index.write().await;
            index.definitions.insert(uri.to_string(), definitions);
            index.last_updated = std::time::SystemTime::now();
        }

        Ok(())
    }

    /// Get symbol information at position
    async fn get_symbol_at_position(
        &self,
        params: &TextDocumentPositionParams,
        content: &str,
        language: &str,
    ) -> Result<Option<SymbolInfo>> {
        // Parse content and find symbol at position
        let lines: Vec<&str> = content.lines().collect();
        
        if params.position.line as usize >= lines.len() {
            return Ok(None);
        }

        let line = lines[params.position.line as usize];
        let char_pos = params.position.character as usize;

        if char_pos >= line.len() {
            return Ok(None);
        }

        // Extract word at position
        let word = self.extract_word_at_position(line, char_pos);
        if word.is_empty() {
            return Ok(None);
        }

        // Use analysis engine to get symbol info
        let symbol_info = self.analysis_engine.get_symbol_info(
            content,
            params.position.line as usize,
            params.position.character as usize,
            Some(language),
        ).await?;

        if let Some(info) = symbol_info {
            Ok(Some(SymbolInfo {
                name: info.name,
                kind: info.kind,
                range: Range {
                    start: Position {
                        line: info.range.start.line as u32,
                        character: info.range.start.character as u32,
                    },
                    end: Position {
                        line: info.range.end.line as u32,
                        character: info.range.end.character as u32,
                    },
                },
                detail: info.detail,
            }))
        } else {
            Ok(None)
        }
    }

    /// Extract word at character position
    fn extract_word_at_position(&self, line: &str, char_pos: usize) -> String {
        let chars: Vec<char> = line.chars().collect();
        
        if char_pos >= chars.len() {
            return String::new();
        }

        // Find word boundaries
        let mut start = char_pos;
        let mut end = char_pos;

        // Extend backward
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        // Extend forward
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        chars[start..end].iter().collect()
    }

    /// Find symbol definitions
    async fn find_symbol_definitions(&self, name: &str, kind: &SymbolKind) -> Result<Vec<SymbolDefinition>> {
        let index = self.symbol_index.read().await;
        let mut definitions = Vec::new();

        for (_uri, doc_definitions) in &index.definitions {
            for def in doc_definitions {
                if def.name == name && def.kind == *kind {
                    definitions.push(def.clone());
                }
            }
        }

        Ok(definitions)
    }

    /// Find symbol references
    async fn find_symbol_references(&self, name: &str, kind: &SymbolKind) -> Result<Vec<SymbolReference>> {
        let index = self.symbol_index.read().await;
        let mut references = Vec::new();

        if let Some(refs) = index.references.get(name) {
            references.extend(refs.clone());
        }

        Ok(references)
    }

    /// Find implementations
    async fn find_implementations(&self, name: &str, kind: &SymbolKind) -> Result<Vec<SymbolDefinition>> {
        // Find implementations of interfaces/abstract classes
        let index = self.symbol_index.read().await;
        let mut implementations = Vec::new();

        // This would require more sophisticated analysis
        // For now, return empty
        Ok(implementations)
    }

    /// Find declarations
    async fn find_symbol_declarations(&self, name: &str, kind: &SymbolKind) -> Result<Vec<SymbolDefinition>> {
        // Find declarations (vs definitions)
        let definitions = self.find_symbol_definitions(name, kind).await?;
        
        // Filter for declarations only
        let declarations: Vec<SymbolDefinition> = definitions
            .into_iter()
            .filter(|def| {
                // Simple heuristic: check if it's in a header file or has specific modifiers
                def.location.uri.ends_with(".h") || 
                def.location.uri.ends_with(".hpp") ||
                def.modifiers.contains(&"extern".to_string())
            })
            .collect();

        Ok(declarations)
    }

    /// Extract type name from symbol
    async fn extract_type_name(&self, symbol: &SymbolInfo, content: &str) -> Result<Option<String>> {
        // Use AI to determine the type of the symbol
        let consensus = self.consensus_engine.read().await;
        let prompt = format!(
            "What is the type of the symbol '{}' in this context?\n\n```\n{}\n```\n\nReturn only the type name.",
            symbol.name,
            content
        );

        match consensus.ask(&prompt).await {
            Ok(response) => {
                let type_name = response.summary.trim().to_string();
                if !type_name.is_empty() && type_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    Ok(Some(type_name))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }

    /// AI-assisted definition search
    async fn ai_assisted_definition_search(
        &self,
        symbol: &SymbolInfo,
        content: &str,
        language: &str,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let consensus = self.consensus_engine.read().await;
        let prompt = format!(
            "Find the definition of symbol '{}' in this {} code. Provide the file path and line number if you can determine it.\n\n```\n{}\n```",
            symbol.name,
            language,
            content
        );

        match consensus.ask(&prompt).await {
            Ok(response) => {
                // Parse AI response for location hints
                // This is a simplified implementation
                if response.summary.contains("line") {
                    warn!("AI found potential definition but need better parsing: {}", response.summary);
                }
                Ok(None)
            }
            Err(e) => {
                warn!("AI-assisted definition search failed: {}", e);
                Ok(None)
            }
        }
    }

    /// AI-assisted reference search
    async fn ai_assisted_reference_search(
        &self,
        symbol: &SymbolInfo,
        language: &str,
    ) -> Result<Option<Vec<Location>>> {
        let consensus = self.consensus_engine.read().await;
        let prompt = format!(
            "Find all references to symbol '{}' in {} code. List the locations where this symbol is used.",
            symbol.name,
            language
        );

        match consensus.ask(&prompt).await {
            Ok(response) => {
                // Parse AI response for reference locations
                debug!("AI reference search result: {}", response.summary);
                Ok(None) // Simplified for now
            }
            Err(e) => {
                warn!("AI-assisted reference search failed: {}", e);
                Ok(None)
            }
        }
    }

    /// Clear caches
    pub async fn clear_caches(&self) {
        let mut def_cache = self.definition_cache.write().await;
        let mut ref_cache = self.reference_cache.write().await;
        def_cache.clear();
        ref_cache.clear();
    }

    /// Get navigation statistics
    pub async fn get_statistics(&self) -> NavigationStatistics {
        let index = self.symbol_index.read().await;
        let def_cache = self.definition_cache.read().await;
        let ref_cache = self.reference_cache.read().await;

        NavigationStatistics {
            total_definitions: index.definitions.values().map(|v| v.len()).sum(),
            total_references: index.references.values().map(|v| v.len()).sum(),
            cached_definitions: def_cache.len(),
            cached_references: ref_cache.len(),
            last_updated: index.last_updated,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: SymbolKind,
    pub range: Range,
    pub detail: Option<String>,
}

#[derive(Debug)]
pub struct NavigationStatistics {
    pub total_definitions: usize,
    pub total_references: usize,
    pub cached_definitions: usize,
    pub cached_references: usize,
    pub last_updated: std::time::SystemTime,
}

// LSP 3.17 Navigation Types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceParams {
    #[serde(flatten)]
    pub text_document_position: TextDocumentPositionParams,
    pub context: ReferenceContext,
    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,
    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkDoneProgressParams {
    #[serde(rename = "workDoneToken", skip_serializing_if = "Option::is_none")]
    pub work_done_token: Option<ProgressToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialResultParams {
    #[serde(rename = "partialResultToken", skip_serializing_if = "Option::is_none")]
    pub partial_result_token: Option<ProgressToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProgressToken {
    String(String),
    Number(i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GotoDefinitionResponse {
    Scalar(Location),
    Array(Vec<Location>),
    Link(Vec<LocationLink>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GotoImplementationResponse {
    Scalar(Location),
    Array(Vec<Location>),
    Link(Vec<LocationLink>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GotoTypeDefinitionResponse {
    Scalar(Location),
    Array(Vec<Location>),
    Link(Vec<LocationLink>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GotoDeclarationResponse {
    Scalar(Location),
    Array(Vec<Location>),
    Link(Vec<LocationLink>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationLink {
    #[serde(rename = "originSelectionRange", skip_serializing_if = "Option::is_none")]
    pub origin_selection_range: Option<Range>,
    #[serde(rename = "targetUri")]
    pub target_uri: String,
    #[serde(rename = "targetRange")]
    pub target_range: Range,
    #[serde(rename = "targetSelectionRange")]
    pub target_selection_range: Range,
}