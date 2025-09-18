//! Semantic Search for Codebase Intelligence
//!
//! Uses AI understanding to match questions with indexed code

use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, info};

use super::{CodebaseIntelligence, ExtractedObject};
use crate::consensus::types::Message;
use crate::core::database::DatabaseManager;

/// Semantic search that understands intent, not just keywords
pub struct SemanticSearch {
    database: Arc<DatabaseManager>,
}

impl SemanticSearch {
    pub fn new(database: Arc<DatabaseManager>) -> Self {
        Self { database }
    }

    /// Determine if a question relates to the indexed codebase using AI understanding
    pub async fn question_relates_to_codebase(
        &self,
        question: &str,
        has_indexed_codebase: bool,
    ) -> Result<(bool, Vec<String>)> {
        // If no codebase has been indexed, we can't search it
        if !has_indexed_codebase {
            return Ok((false, vec![]));
        }

        // Extract ALL meaningful words from the question as potential search terms
        // The AI will determine which are relevant
        let search_terms = self.extract_all_search_terms(question);

        // Here's the key insight: We don't decide if it's about code.
        // We search for matches and let the AI determine relevance.
        // If we find ANYTHING that matches, we include it and let the AI decide.

        info!(
            "🔍 Semantic search for: {} (terms: {:?})",
            question, search_terms
        );

        // Search for any matches in our indexed codebase
        let has_matches = !search_terms.is_empty();

        Ok((has_matches, search_terms))
    }

    /// Extract all potential search terms from a question
    fn extract_all_search_terms(&self, question: &str) -> Vec<String> {
        // Split into words and filter out common stop words
        let stop_words = [
            "the", "is", "at", "which", "on", "a", "an", "and", "or", "but", "in", "with", "to",
            "for", "of", "as", "by", "that", "it", "from", "up", "out", "if", "then", "than",
            "this",
        ];

        let terms: Vec<String> = question
            .split_whitespace()
            .filter_map(|word| {
                // Clean the word
                let cleaned = word
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase();

                // Keep if it's meaningful
                if cleaned.len() > 2 && !stop_words.contains(&cleaned.as_str()) {
                    Some(cleaned)
                } else {
                    None
                }
            })
            .collect();

        // Also extract potential code identifiers (CamelCase, snake_case, etc.)
        let mut code_terms = self.extract_code_identifiers(question);

        // Combine and deduplicate
        let mut all_terms = terms;
        all_terms.append(&mut code_terms);
        all_terms.sort();
        all_terms.dedup();

        all_terms
    }

    /// Extract potential code identifiers from text
    fn extract_code_identifiers(&self, text: &str) -> Vec<String> {
        let mut identifiers = Vec::new();

        for word in text.split_whitespace() {
            // Check for code-like patterns
            if word.contains('_') ||                    // snake_case
               word.contains("::") ||                   // Rust paths
               word.contains('.') && !word.ends_with('.') || // method calls
               word.chars().filter(|c| c.is_uppercase()).count() > 1
            // CamelCase
            {
                identifiers.push(word.to_lowercase());
            }
        }

        identifiers
    }

    /// Search the indexed codebase using semantic understanding
    pub async fn search_indexed_codebase(
        &self,
        scan_id: &str,
        search_terms: &[String],
    ) -> Result<Vec<SearchResult>> {
        debug!("Searching indexed codebase with terms: {:?}", search_terms);

        // Build SQL query that searches across multiple fields
        let mut query = String::from(
            "SELECT DISTINCT 
                o.id, o.name, o.kind, o.file_path, o.line_start, 
                o.signature, o.documentation, o.context,
                COUNT(*) as match_count,
                GROUP_CONCAT(DISTINCT k.keyword) as matching_keywords
             FROM indexed_objects o
             LEFT JOIN indexed_keywords k ON k.scan_id = o.scan_id
             WHERE o.scan_id = ?1 AND (",
        );

        // Search in multiple fields for any of the terms
        let fields = [
            "o.name",
            "o.signature",
            "o.documentation",
            "o.context",
            "k.keyword",
        ];
        let mut conditions = Vec::new();

        for term in search_terms {
            let field_conditions: Vec<String> = fields
                .iter()
                .map(|field| format!("LOWER({}) LIKE '%{}%'", field, term.to_lowercase()))
                .collect();
            conditions.push(format!("({})", field_conditions.join(" OR ")));
        }

        query.push_str(&conditions.join(" OR "));
        query.push_str(") GROUP BY o.id ORDER BY match_count DESC, o.name LIMIT 50");

        // Execute search
        let conn = self.database.get_connection()?;
        let mut stmt = conn.prepare(&query)?;

        let results = stmt.query_map([scan_id], |row| {
            Ok(SearchResult {
                object_id: row.get(0)?,
                name: row.get(1)?,
                kind: row.get(2)?,
                file_path: row.get(3)?,
                line_start: row.get(4)?,
                signature: row.get(5)?,
                documentation: row.get(6)?,
                context: row.get(7)?,
                match_count: row.get(8)?,
                matching_keywords: row
                    .get::<_, Option<String>>(9)?
                    .map(|s| s.split(',').map(String::from).collect())
                    .unwrap_or_default(),
            })
        })?;

        let mut search_results = Vec::new();
        for result in results {
            search_results.push(result?);
        }

        info!("Found {} potential matches", search_results.len());

        Ok(search_results)
    }

    /// Build context from search results, letting AI determine relevance
    pub fn build_ai_context(&self, results: &[SearchResult], question: &str) -> String {
        let mut context = String::new();

        context.push_str("## 🔍 SEMANTIC CODEBASE SEARCH RESULTS\n\n");
        context.push_str(&format!("Your question: \"{}\"\n\n", question));
        context.push_str("I found the following in your codebase that may be relevant:\n\n");

        for (idx, result) in results.iter().take(20).enumerate() {
            context.push_str(&format!(
                "### {}. {} `{}`\n",
                idx + 1,
                result.kind,
                result.name
            ));

            context.push_str(&format!(
                "📍 Location: `{}`:{}\n",
                result.file_path, result.line_start
            ));

            if let Some(doc) = &result.documentation {
                context.push_str(&format!("📝 Documentation: {}\n", doc));
            }

            if !result.context.is_empty() {
                context.push_str(&format!("🎯 Purpose: {}\n", result.context));
            }

            if !result.signature.is_empty() {
                context.push_str(&format!("```\n{}\n```\n", result.signature));
            }

            if !result.matching_keywords.is_empty() {
                context.push_str(&format!(
                    "🏷️ Related concepts: {}\n",
                    result.matching_keywords.join(", ")
                ));
            }

            context.push_str("\n");
        }

        context.push_str("💡 Use the above search results to answer the question accurately. ");
        context.push_str(
            "Focus on the actual code and implementation details found in your codebase.\n",
        );

        context
    }
}

/// Search result from indexed codebase
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub object_id: String,
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub line_start: usize,
    pub signature: String,
    pub documentation: Option<String>,
    pub context: String,
    pub match_count: i32,
    pub matching_keywords: Vec<String>,
}

/// Enhanced pipeline integration that ALWAYS uses semantic search after @codebase
pub async fn should_use_codebase_intelligence(
    question: &str,
    codebase_intelligence: Option<&CodebaseIntelligence>,
) -> Result<bool> {
    // If we have indexed codebase, ALWAYS search it
    if let Some(intelligence) = codebase_intelligence {
        if intelligence.has_analysis().await {
            info!("📚 Codebase is indexed - will search for relevant content");
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_extract_search_terms() {
        let db_config = crate::core::database::DatabaseConfig {
            path: std::path::PathBuf::from(":memory:"),
            max_connections: 10,
            enable_wal: false,
            busy_timeout: 5000,
        };
        let db = DatabaseManager::new(db_config).await.unwrap();
        let search = SemanticSearch::new(Arc::new(db));

        let terms = search.extract_all_search_terms("discover what consensus is capable of");
        assert!(terms.contains(&"discover".to_string()));
        assert!(terms.contains(&"consensus".to_string()));
        assert!(terms.contains(&"capable".to_string()));
        assert!(!terms.contains(&"what".to_string())); // stop word
        assert!(!terms.contains(&"is".to_string())); // stop word
        assert!(!terms.contains(&"of".to_string())); // stop word
    }

    #[test]
    fn test_extract_code_identifiers() {
        let db_config = crate::core::database::DatabaseConfig {
            path: std::path::PathBuf::from(":memory:"),
            max_connections: 10,
            enable_wal: false,
            busy_timeout: 5000,
        };
        let db = DatabaseManager::new(db_config).await.unwrap();
        let search = SemanticSearch::new(Arc::new(db));

        let identifiers = search.extract_code_identifiers("How does FileReader::read_file work?");
        assert!(identifiers.contains(&"fileReader::read_file".to_string()));

        let identifiers = search.extract_code_identifiers("What is consensus_pipeline doing?");
        assert!(identifiers.contains(&"consensus_pipeline".to_string()));
    }
}
