//! Thematic clustering system for conversation grouping and knowledge retrieval
//! 
//! Implements intelligent conversation clustering based on topics and themes:
//! - Stage-specific knowledge retrieval for consensus pipeline
//! - Conversation continuity and follow-up detection 
//! - Curator knowledge base with authoritative answers
//! - 100% feature parity with TypeScript thematic system

use anyhow::{Context, Result};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, warn};
use crate::memory::topic_extraction::{find_topics_for_query, find_conversations_by_topic};
use crate::consensus::openrouter::{OpenRouterClient, OpenRouterRequest, OpenRouterMessage};
use crate::core::api_keys::ApiKeyManager;

/// Conversation thread type for relationship tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThreadType {
    FollowUp,
    Clarification,
    ListReference,
    Continuation,
}

impl std::fmt::Display for ThreadType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreadType::FollowUp => write!(f, "follow_up"),
            ThreadType::Clarification => write!(f, "clarification"),
            ThreadType::ListReference => write!(f, "list_reference"),
            ThreadType::Continuation => write!(f, "continuation"),
        }
    }
}

/// Conversation thread relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationThread {
    pub child_conversation_id: String,
    pub parent_conversation_id: String,
    pub thread_type: ThreadType,
    pub reference_text: Option<String>,
    pub confidence_score: f64,
}

/// Message importance scoring factors
#[derive(Debug, Clone)]
pub struct MessageImportance {
    pub keyword_score: f64,    // Up to 3 points for important keywords
    pub length_score: f64,     // Up to 2 points for longer messages
    pub recency_score: f64,    // Up to 3 points for recent messages
    pub role_score: f64,       // 1 point for user messages
    pub total_score: f64,
}

/// Knowledge source for thematic retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSource {
    pub conversation_id: String,
    pub question: String,
    pub curator_output: String,
    pub topics: Vec<String>,
    pub confidence_score: f64,
    pub created_at: DateTime<Utc>,
}

/// Follow-up detection patterns (matches TypeScript implementation)
const FOLLOW_UP_PATTERNS: &[&str] = &[
    r"give me (\d+|\w+) examples?",
    r"tell me more",
    r"can you explain",
    r"show me",
    r"what about", 
    r"\b(those|these|that|this|them|they|it)\b",  // Implicit references
    r"\b(the \d+(?:st|nd|rd|th)|first|second|third|last)\b",  // List references
    r"go deeper",
    r"elaborate",
    r"more details",
    r"expand on",
];

/// Thematic clustering engine for conversation organization
#[derive(Debug)]
pub struct ThematicCluster {
    /// OpenRouter client for AI-powered analysis
    client: Option<OpenRouterClient>,
}

impl ThematicCluster {
    /// Create a new thematic clustering engine
    pub async fn new() -> Result<Self> {
        // Get API key for OpenRouter
        let client = match ApiKeyManager::get_openrouter_key().await {
            Ok(api_key) => Some(OpenRouterClient::new(api_key)),
            Err(_) => {
                warn!("OpenRouter API key not available, AI features will be limited");
                None
            }
        };
        Ok(Self { client })
    }

    /// Find relevant knowledge for AI stage - TEMPORAL PRIORITY SYSTEM (matches TypeScript)
    /// Priority: 1. Recent conversations (24h) 2. Follow-up detection 3. Thematic search 4. AI knowledge
    pub async fn find_relevant_knowledge_for_ai(&self, query: &str, stage: &str) -> Result<String> {
        info!("Finding relevant knowledge for {} stage with temporal priority", stage);
        
        // STEP 1: Check for follow-up patterns first (highest priority)
        let is_follow_up = self.is_follow_up_query(query);
        
        // STEP 2: Get recent curator knowledge base (past 24 hours) - PRIMARY SOURCE
        let recent_knowledge = self.get_recent_curator_knowledge(query).await?;
        if !recent_knowledge.is_empty() {
            info!("Found recent curator knowledge (24h) for {} stage - using as authoritative context", stage);
            return Ok(self.format_stage_context(&recent_knowledge, stage, "recent conversations"));
        }
        
        // STEP 3: If follow-up detected but no recent knowledge, try broader recent search
        if is_follow_up {
            let broader_recent = self.get_recent_conversations(query, 72).await?; // 3 days
            if !broader_recent.is_empty() {
                info!("Follow-up detected - found broader recent context for {} stage", stage);
                return Ok(self.format_stage_context(&broader_recent, stage, "recent follow-up context"));
            }
        }
        
        // STEP 4: Fall back to thematic search if no recent context
        info!("No recent context found, falling back to thematic search for {} stage", stage);
        let thematic_knowledge = self.get_thematic_knowledge(query, stage).await?;
        if !thematic_knowledge.is_empty() {
            return Ok(self.format_stage_context(&thematic_knowledge, stage, "thematic matches"));
        }
        
        // STEP 5: No relevant context found
        info!("No relevant context found for query in {} stage", stage);
        Ok(String::new())
    }

    /// Detect if a query is a follow-up to a previous conversation
    pub async fn detect_follow_up(&self, query: &str, recent_conversations: &[String]) -> Result<Option<ConversationThread>> {
        debug!("Detecting follow-up patterns in query: {}", query);
        
        // First, check for explicit follow-up patterns using regex
        let pattern_confidence = self.check_follow_up_patterns(query);
        
        if pattern_confidence > 0.3 {
            // Use AI to analyze the follow-up with high confidence
            return self.ai_analyze_follow_up(query, recent_conversations, pattern_confidence).await;
        }
        
        // If no clear patterns, check if there are implicit references
        if self.has_implicit_references(query) && !recent_conversations.is_empty() {
            // Use AI for implicit reference analysis
            return self.ai_analyze_follow_up(query, recent_conversations, 0.5).await;
        }
        
        Ok(None)
    }

    /// Check for explicit follow-up patterns using regex (matches TypeScript)
    fn check_follow_up_patterns(&self, query: &str) -> f64 {
        let query_lower = query.to_lowercase();
        let mut confidence: f64 = 0.0;
        
        for pattern in FOLLOW_UP_PATTERNS {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if regex.is_match(&query_lower) {
                    confidence += match pattern {
                        p if p.contains("examples") => 0.8,
                        p if p.contains("tell me more") => 0.9,
                        p if p.contains("explain") => 0.7,
                        p if p.contains("show me") => 0.7,
                        p if p.contains("those|these|that|this") => 0.6,
                        p if p.contains("first|second|third") => 0.8,
                        _ => 0.5,
                    };
                }
            }
        }
        
        // Cap at 1.0
        confidence.min(1.0)
    }

    /// Check for implicit references (pronouns, demonstratives)
    fn has_implicit_references(&self, query: &str) -> bool {
        let implicit_words = ["it", "this", "that", "these", "those", "them", "they"];
        let query_lower = query.to_lowercase();
        
        implicit_words.iter().any(|word| {
            query_lower.contains(&format!(" {} ", word)) || 
            query_lower.starts_with(&format!("{} ", word))
        })
    }

    /// Use AI to analyze potential follow-up relationships
    async fn ai_analyze_follow_up(
        &self, 
        query: &str, 
        recent_conversations: &[String], 
        base_confidence: f64
    ) -> Result<Option<ConversationThread>> {
        if recent_conversations.is_empty() {
            return Ok(None);
        }
        
        // Get the most recent conversation for analysis
        let recent_conversation_id = &recent_conversations[0];
        
        // Get conversation context
        let conversation_context = self.get_conversation_context(recent_conversation_id).await?;
        
        // Prepare AI prompt for follow-up analysis
        let system_prompt = format!(
            "Analyze if the current query is a follow-up to the previous conversation. \
            Rate the likelihood on a scale of 0-10 where:\n\
            0-2: Not a follow-up\n\
            3-5: Possible follow-up\n\
            6-8: Likely follow-up\n\
            9-10: Definite follow-up\n\n\
            Previous conversation:\n{}\n\n\
            Current query: {}\n\n\
            Respond with just the number (0-10).",
            conversation_context, query
        );
        
        let client = match &self.client {
            Some(client) => client,
            None => {
                warn!("No OpenRouter client available for AI analysis");
                return Ok(None);
            }
        };
        
        let request = OpenRouterRequest {
            model: "auto".to_string(),
            messages: vec![
                OpenRouterMessage {
                    role: "user".to_string(),
                    content: system_prompt,
                },
            ],
            temperature: Some(0.1),
            max_tokens: Some(50),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stream: None,
            provider: None,
        };
        
        let response = client.chat_completion(request).await?;
        let content = response.choices.first()
            .and_then(|choice| choice.message.as_ref())
            .map(|message| message.content.as_str())
            .unwrap_or("");
        
        // Parse the AI confidence score
        if let Ok(ai_score) = content.trim().parse::<f64>() {
            let ai_confidence = ai_score / 10.0; // Normalize to 0-1
            let combined_confidence = (base_confidence + ai_confidence) / 2.0;
            
            debug!("Follow-up analysis: pattern={:.2}, ai={:.2}, combined={:.2}", 
                   base_confidence, ai_confidence, combined_confidence);
            
            if combined_confidence > 0.5 {
                // Determine thread type based on content
                let thread_type = self.determine_thread_type(query);
                
                return Ok(Some(ConversationThread {
                    child_conversation_id: String::new(), // Will be filled by caller
                    parent_conversation_id: recent_conversation_id.clone(),
                    thread_type,
                    reference_text: Some(query.to_string()),
                    confidence_score: combined_confidence,
                }));
            }
        }
        
        Ok(None)
    }

    /// Get conversation context for AI analysis
    async fn get_conversation_context(&self, conversation_id: &str) -> Result<String> {
        use crate::core::database::get_database;
        
        let db = get_database().await?;
        let conn = db.get_connection()?;
        
        // Get recent messages from the conversation
        let mut stmt = conn.prepare(
            "SELECT role, content FROM messages 
             WHERE conversation_id = ? 
             ORDER BY timestamp DESC 
             LIMIT 5"
        )?;
        
        let message_rows = stmt.query_map([conversation_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        
        let mut messages = Vec::new();
        for message_result in message_rows {
            messages.push(message_result?);
        }
        
        // Reverse to get chronological order
        messages.reverse();
        
        let context = messages
            .iter()
            .map(|(role, content)| format!("{}: {}", role.to_uppercase(), content))
            .collect::<Vec<_>>()
            .join("\n");
        
        Ok(context)
    }

    /// Determine the type of thread relationship
    fn determine_thread_type(&self, query: &str) -> ThreadType {
        let query_lower = query.to_lowercase();
        
        if query_lower.contains("example") || query_lower.contains("show me") {
            ThreadType::ListReference
        } else if query_lower.contains("explain") || query_lower.contains("clarify") {
            ThreadType::Clarification
        } else if query_lower.contains("more") || query_lower.contains("continue") {
            ThreadType::Continuation
        } else {
            ThreadType::FollowUp
        }
    }

    /// Store conversation thread relationship in database
    pub async fn store_conversation_thread(&self, thread: &ConversationThread) -> Result<()> {
        use crate::core::database::get_database;
        
        let db = get_database().await?;
        let conn = db.get_connection()?;
        
        conn.execute(
            "INSERT INTO conversation_threads 
             (child_conversation_id, parent_conversation_id, thread_type, reference_text, confidence_score) 
             VALUES (?, ?, ?, ?, ?)",
            rusqlite::params![
                thread.child_conversation_id,
                thread.parent_conversation_id,
                thread.thread_type.to_string(),
                thread.reference_text,
                thread.confidence_score
            ],
        )?;
        
        info!("Stored conversation thread: {} -> {} ({})", 
              thread.parent_conversation_id, 
              thread.child_conversation_id, 
              thread.thread_type);
        
        Ok(())
    }

    /// Get curator knowledge base for a query (matches TypeScript getCuratorKnowledgeBase)
    pub async fn get_curator_knowledge_base(&self, query: &str) -> Result<String> {
        // This is the main entry point - delegates to find_relevant_knowledge_for_ai
        // which now implements the proper temporal priority system
        self.find_relevant_knowledge_for_ai(query, "curator").await
    }
    
    /// Check if query contains follow-up patterns (enhanced from TypeScript)
    fn is_follow_up_query(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        
        // Enhanced follow-up patterns from TypeScript analysis
        let follow_up_patterns = [
            "give me", "tell me more", "can you explain", "show me", "what about",
            "examples", "code example", "how does", "can you guess", "try to list",
            "based on", "following up", "as i mentioned", "as we discussed",
            "earlier", "previous", "before", "from earlier", "that we",
            "it", "that", "this", "those", "these", "them", "they"
        ];
        
        follow_up_patterns.iter().any(|pattern| query_lower.contains(pattern))
    }
    
    /// Get recent curator knowledge (past 24 hours) - HIGHEST PRIORITY
    async fn get_recent_curator_knowledge(&self, _query: &str) -> Result<String> {
        use crate::core::database::get_database;
        
        debug!("Getting recent curator knowledge (24 hours)");
        
        let db = get_database().await?;
        let conn = db.get_connection()?;
        
        // Get recent verified curator outputs (within 24 hours) - NEWEST FIRST
        let twenty_four_hours_ago = Utc::now() - Duration::hours(24);
        
        let mut stmt = conn.prepare(
            "SELECT c.question, c.final_answer, c.source_of_truth, c.created_at 
             FROM conversations c
             WHERE c.created_at > ?
             ORDER BY c.created_at DESC
             LIMIT 3"
        )?;
        
        let result_rows = stmt.query_map([twenty_four_hours_ago.to_rfc3339()], |row| {
            Ok((
                row.get::<_, String>(0)?, // question
                row.get::<_, String>(1)?, // final_answer
                row.get::<_, String>(2)?, // source_of_truth (curator output)
                row.get::<_, String>(3)?, // created_at
            ))
        })?;
        
        let mut recent_answers = Vec::new();
        for result in result_rows {
            recent_answers.push(result?);
        }
        
        if recent_answers.is_empty() {
            debug!("No recent curator knowledge found (24h)");
            return Ok(String::new());
        }
        
        // Format as authoritative articles - NEWEST FIRST
        let formatted_knowledge = recent_answers
            .iter()
            .enumerate()
            .map(|(index, (question, _final_answer, curator_output, created_at))| {
                let date = DateTime::parse_from_rfc3339(created_at)
                    .unwrap_or_else(|_| Utc::now().into())
                    .format("%Y-%m-%d %H:%M");
                
                format!(
                    "AUTHORITATIVE ARTICLE {} (from {}):\nOriginal Question: {}\n\nVerified Answer:\n{}\n\n---",
                    index + 1, date, question, curator_output
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        
        info!("Found {} recent curator answers (24h) - using as authoritative context", recent_answers.len());
        Ok(formatted_knowledge)
    }
    
    /// Get recent conversations (broader time window for follow-ups)
    async fn get_recent_conversations(&self, _query: &str, hours: i64) -> Result<String> {
        use crate::core::database::get_database;
        
        debug!("Getting recent conversations ({} hours)", hours);
        
        let db = get_database().await?;
        let conn = db.get_connection()?;
        
        let cutoff_time = Utc::now() - Duration::hours(hours);
        
        let mut stmt = conn.prepare(
            "SELECT question, final_answer, source_of_truth, created_at 
             FROM conversations 
             WHERE created_at > ?
             ORDER BY created_at DESC
             LIMIT 5"
        )?;
        
        let result_rows = stmt.query_map([cutoff_time.to_rfc3339()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        
        let mut conversations = Vec::new();
        for result in result_rows {
            conversations.push(result?);
        }
        
        if conversations.is_empty() {
            return Ok(String::new());
        }
        
        let formatted = conversations
            .iter()
            .enumerate()
            .map(|(index, (question, _answer, curator_output, created_at))| {
                let date = DateTime::parse_from_rfc3339(created_at)
                    .unwrap_or_else(|_| Utc::now().into())
                    .format("%Y-%m-%d %H:%M");
                
                format!(
                    "RECENT CONVERSATION {} (from {}):\nQuestion: {}\n\nAnswer:\n{}\n\n---",
                    index + 1, date, question, curator_output
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        
        info!("Found {} recent conversations ({} hours)", conversations.len(), hours);
        Ok(formatted)
    }
    
    /// Get thematic knowledge (fallback when no recent context) - LOWEST PRIORITY
    async fn get_thematic_knowledge(&self, query: &str, stage: &str) -> Result<String> {
        debug!("Falling back to thematic knowledge search");
        
        // Extract topics from the query
        let query_topics = find_topics_for_query(query).await?;
        if query_topics.is_empty() {
            return Ok(String::new());
        }
        
        // Find conversations related to these topics
        let mut all_conversations = HashSet::new();
        for topic in &query_topics {
            let topic_conversations = find_conversations_by_topic(topic, Some(3)).await?;
            for conv in topic_conversations {
                all_conversations.insert(conv);
            }
        }
        
        if all_conversations.is_empty() {
            return Ok(String::new());
        }
        
        use crate::core::database::get_database;
        let db = get_database().await?;
        let conn = db.get_connection()?;
        
        let mut thematic_knowledge = String::new();
        for conversation_id in all_conversations {
            // Get conversation from our schema
            let mut stmt = conn.prepare(
                "SELECT question, source_of_truth, created_at FROM conversations WHERE id = ?"
            )?;
            
            let conversation_result = stmt.query_row([&conversation_id], |row| {
                Ok((
                    row.get::<_, String>(0)?, // question
                    row.get::<_, String>(1)?, // source_of_truth (curator output)
                    row.get::<_, String>(2)?, // created_at
                ))
            });
            
            if let Ok((question, curator_output, created_at)) = conversation_result {
                let date = DateTime::parse_from_rfc3339(&created_at)
                    .unwrap_or_else(|_| Utc::now().into())
                    .format("%Y-%m-%d");
                
                thematic_knowledge += &format!(
                    "\n\nTHEMATIC MATCH (from {}):\nQuestion: {}\n\nAnswer:\n{}\n",
                    date, question, curator_output
                );
            }
        }
        
        if !thematic_knowledge.is_empty() {
            info!("Found thematic knowledge for {} topics", query_topics.len());
        }
        
        Ok(thematic_knowledge)
    }
    
    /// Format context for specific stage
    fn format_stage_context(&self, knowledge: &str, stage: &str, source_type: &str) -> String {
        match stage {
            "generator" => format!(
                "Here is authoritative information from {} to inform your response:\n{}\n\nUse this context as the foundation for your response. Build upon this authoritative information.",
                source_type, knowledge
            ),
            "refiner" => format!(
                "Here is authoritative information from {} to guide your refinement:\n{}\n\nRespect this authoritative context and enhance the response while staying consistent with it.",
                source_type, knowledge
            ),
            "validator" => format!(
                "Here is authoritative information from {} for fact-checking:\n{}\n\nUse this as your primary validation source. This information should be considered authoritative.",
                source_type, knowledge
            ),
            "curator" => format!(
                "Here is authoritative information from {} to guide your curation:\n{}\n\nThis represents our established knowledge base. Ensure consistency with this authoritative context.",
                source_type, knowledge
            ),
            _ => knowledge.to_string(),
        }
    }

    /// Calculate message importance score (matches TypeScript algorithm)
    pub fn calculate_message_importance(&self, role: &str, content: &str, timestamp: DateTime<Utc>) -> MessageImportance {
        let content_lower = content.to_lowercase();
        
        // Important keywords scoring (up to 3 points)
        let important_keywords = [
            "error", "issue", "problem", "help", "question", "how", "what", "why", "when", "where",
            "explain", "show", "example", "tutorial", "guide", "documentation", "api", "code",
            "implementation", "solution", "fix", "debug", "optimize", "performance"
        ];
        
        let keyword_matches = important_keywords
            .iter()
            .filter(|&keyword| content_lower.contains(keyword))
            .count();
        let keyword_score = (keyword_matches as f64 * 0.5).min(3.0);
        
        // Length scoring (up to 2 points) - longer messages often contain more context
        let length_score = if content.len() > 200 { 2.0 }
                          else if content.len() > 100 { 1.5 }
                          else if content.len() > 50 { 1.0 }
                          else { 0.5 };
        
        // Recency scoring (up to 3 points) - more recent messages are more relevant
        let hours_ago = (Utc::now() - timestamp).num_hours() as f64;
        let recency_score = if hours_ago < 1.0 { 3.0 }
                           else if hours_ago < 6.0 { 2.5 }
                           else if hours_ago < 24.0 { 2.0 }
                           else if hours_ago < 168.0 { 1.0 } // 1 week
                           else { 0.0 };
        
        // Role scoring (1 point for user messages)
        let role_score = if role.to_lowercase() == "user" { 1.0 } else { 0.0 };
        
        let total_score = keyword_score + length_score + recency_score + role_score;
        
        MessageImportance {
            keyword_score,
            length_score,
            recency_score,
            role_score,
            total_score,
        }
    }

    /// Get conversation clusters by topic similarity
    pub async fn get_conversation_clusters(&self, limit: Option<usize>) -> Result<HashMap<String, Vec<String>>> {
        use crate::core::database::get_database;
        
        let limit = limit.unwrap_or(50);
        let db = get_database().await?;
        let conn = db.get_connection()?;
        
        // Get conversations grouped by topic
        let mut stmt = conn.prepare(
            "SELECT topic, conversation_id 
             FROM conversation_topics 
             ORDER BY topic, conversation_id 
             LIMIT ?"
        )?;
        
        let topic_rows = stmt.query_map([limit], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        
        let mut clusters: HashMap<String, Vec<String>> = HashMap::new();
        for topic_result in topic_rows {
            let (topic, conversation_id) = topic_result?;
            clusters.entry(topic).or_insert_with(Vec::new).push(conversation_id);
        }
        
        debug!("Found {} topic clusters with {} conversations", clusters.len(), limit);
        Ok(clusters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_follow_up_pattern_detection() {
        let cluster = ThematicCluster::new().await.unwrap();
        
        // Test explicit patterns
        assert!(cluster.check_follow_up_patterns("give me 5 examples") > 0.5);
        assert!(cluster.check_follow_up_patterns("tell me more about this") > 0.5);
        assert!(cluster.check_follow_up_patterns("can you explain that?") > 0.5);
        
        // Test implicit references
        assert!(cluster.has_implicit_references("show me how to implement it"));
        assert!(cluster.has_implicit_references("what about those other options?"));
        assert!(!cluster.has_implicit_references("how do I start programming?"));
    }
    
    #[tokio::test]
    async fn test_thread_type_determination() {
        let cluster = ThematicCluster::new().await.unwrap();
        
        assert_eq!(cluster.determine_thread_type("give me examples"), ThreadType::ListReference);
        assert_eq!(cluster.determine_thread_type("can you explain that?"), ThreadType::Clarification);
        assert_eq!(cluster.determine_thread_type("tell me more"), ThreadType::Continuation);
        assert_eq!(cluster.determine_thread_type("what about this?"), ThreadType::FollowUp);
    }
    
    #[tokio::test]
    async fn test_message_importance_calculation() {
        let cluster = ThematicCluster::new().await.unwrap();
        let timestamp = Utc::now() - Duration::hours(2);
        
        let importance = cluster.calculate_message_importance(
            "user",
            "I have an error in my code. Can you help me debug this issue with the API implementation?",
            timestamp
        );
        
        assert!(importance.keyword_score > 0.0); // Should detect "error", "help", "debug", etc.
        assert!(importance.role_score > 0.0);    // User message gets points
        assert!(importance.recency_score > 0.0); // Recent message
        assert!(importance.total_score > 2.0);   // Should be considered important
    }
    
    #[test]
    fn test_thread_type_display() {
        assert_eq!(ThreadType::FollowUp.to_string(), "follow_up");
        assert_eq!(ThreadType::Clarification.to_string(), "clarification");
        assert_eq!(ThreadType::ListReference.to_string(), "list_reference");
        assert_eq!(ThreadType::Continuation.to_string(), "continuation");
    }
}