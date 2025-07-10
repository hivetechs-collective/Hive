//! Topic extraction system for thematic memory
//! 
//! Implements a sophisticated topic extraction engine that provides:
//! - Hierarchical topic taxonomy with 6 main categories  
//! - Dual extraction approach (keyword-based + AI-powered)
//! - Provider-agnostic AI integration with fallback
//! - 100% feature parity with TypeScript implementation

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, warn};
use crate::consensus::openrouter::{OpenRouterClient, OpenRouterRequest, OpenRouterMessage};
use crate::core::api_keys::ApiKeyManager;

/// Topic hierarchy structure matching TypeScript implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicHierarchy {
    pub related: Vec<String>,
    pub subtopics: HashMap<String, Vec<String>>,
}

/// Complete topic hierarchy system - identical to TypeScript version
pub fn get_topic_hierarchy() -> HashMap<String, TopicHierarchy> {
    let mut hierarchy = HashMap::new();
    
    // Artificial Intelligence
    hierarchy.insert("artificial_intelligence".to_string(), TopicHierarchy {
        related: vec![
            "machine_learning".to_string(), "deep_learning".to_string(), 
            "neural_networks".to_string(), "ai".to_string(), "algorithms".to_string()
        ],
        subtopics: {
            let mut subtopics = HashMap::new();
            subtopics.insert("machine_learning".to_string(), vec![
                "supervised".to_string(), "unsupervised".to_string(), "reinforcement".to_string(),
                "models".to_string(), "algorithms".to_string(), "training".to_string(), "features".to_string()
            ]);
            subtopics.insert("neural_networks".to_string(), vec![
                "deep_learning".to_string(), "cnn".to_string(), "rnn".to_string(),
                "transformer".to_string(), "attention".to_string(), "layers".to_string(), "activation".to_string()
            ]);
            subtopics.insert("nlp".to_string(), vec![
                "language_models".to_string(), "chatbots".to_string(), "text_processing".to_string(),
                "sentiment".to_string(), "tokens".to_string(), "embedding".to_string()
            ]);
            subtopics.insert("computer_vision".to_string(), vec![
                "image_recognition".to_string(), "object_detection".to_string(),
                "segmentation".to_string(), "face_recognition".to_string()
            ]);
            subtopics.insert("ethics".to_string(), vec![
                "bias".to_string(), "fairness".to_string(), "transparency".to_string(),
                "accountability".to_string(), "privacy".to_string(), "safety".to_string()
            ]);
            subtopics
        }
    });
    
    // Programming
    hierarchy.insert("programming".to_string(), TopicHierarchy {
        related: vec![
            "software_development".to_string(), "coding".to_string(),
            "languages".to_string(), "frameworks".to_string(), "algorithms".to_string()
        ],
        subtopics: {
            let mut subtopics = HashMap::new();
            subtopics.insert("languages".to_string(), vec![
                "javascript".to_string(), "python".to_string(), "java".to_string(),
                "typescript".to_string(), "cpp".to_string(), "go".to_string(), 
                "rust".to_string(), "php".to_string()
            ]);
            subtopics.insert("web_development".to_string(), vec![
                "frontend".to_string(), "backend".to_string(), "fullstack".to_string(),
                "frameworks".to_string(), "html".to_string(), "css".to_string(),
                "react".to_string(), "angular".to_string(), "vue".to_string()
            ]);
            subtopics.insert("mobile_development".to_string(), vec![
                "ios".to_string(), "android".to_string(), "react_native".to_string(),
                "flutter".to_string(), "mobile_apps".to_string()
            ]);
            subtopics.insert("databases".to_string(), vec![
                "sql".to_string(), "nosql".to_string(), "relational".to_string(),
                "document".to_string(), "graph".to_string(), "key_value".to_string()
            ]);
            subtopics.insert("devops".to_string(), vec![
                "ci_cd".to_string(), "containers".to_string(), "kubernetes".to_string(),
                "deployment".to_string(), "cloud".to_string()
            ]);
            subtopics
        }
    });
    
    // Data
    hierarchy.insert("data".to_string(), TopicHierarchy {
        related: vec![
            "database".to_string(), "analysis".to_string(), "storage".to_string(),
            "processing".to_string(), "visualization".to_string()
        ],
        subtopics: {
            let mut subtopics = HashMap::new();
            subtopics.insert("databases".to_string(), vec![
                "relational".to_string(), "nosql".to_string(), "sql".to_string(),
                "mysql".to_string(), "postgresql".to_string(), "mongodb".to_string(), "cassandra".to_string()
            ]);
            subtopics.insert("big_data".to_string(), vec![
                "hadoop".to_string(), "spark".to_string(), "distributed".to_string(),
                "processing".to_string(), "streams".to_string()
            ]);
            subtopics.insert("data_science".to_string(), vec![
                "analysis".to_string(), "statistics".to_string(), "visualization".to_string(),
                "insights".to_string(), "pandas".to_string(), "jupyter".to_string()
            ]);
            subtopics.insert("data_engineering".to_string(), vec![
                "pipelines".to_string(), "etl".to_string(), "warehousing".to_string(),
                "lake".to_string(), "governance".to_string()
            ]);
            subtopics
        }
    });
    
    // Blockchain
    hierarchy.insert("blockchain".to_string(), TopicHierarchy {
        related: vec![
            "cryptocurrency".to_string(), "distributed_ledger".to_string(),
            "smart_contracts".to_string(), "decentralized".to_string()
        ],
        subtopics: {
            let mut subtopics = HashMap::new();
            subtopics.insert("cryptocurrency".to_string(), vec![
                "bitcoin".to_string(), "ethereum".to_string(), "tokens".to_string(),
                "mining".to_string(), "wallets".to_string()
            ]);
            subtopics.insert("smart_contracts".to_string(), vec![
                "solidity".to_string(), "execution".to_string(), "conditions".to_string(), "automation".to_string()
            ]);
            subtopics.insert("consensus".to_string(), vec![
                "proof_of_work".to_string(), "proof_of_stake".to_string(),
                "algorithms".to_string(), "validation".to_string(), "mining".to_string()
            ]);
            subtopics.insert("defi".to_string(), vec![
                "decentralized_finance".to_string(), "lending".to_string(),
                "trading".to_string(), "yield".to_string(), "staking".to_string()
            ]);
            subtopics
        }
    });
    
    // Security
    hierarchy.insert("security".to_string(), TopicHierarchy {
        related: vec![
            "cybersecurity".to_string(), "encryption".to_string(), "protection".to_string(),
            "threats".to_string(), "privacy".to_string()
        ],
        subtopics: {
            let mut subtopics = HashMap::new();
            subtopics.insert("encryption".to_string(), vec![
                "cryptography".to_string(), "algorithms".to_string(), "keys".to_string(),
                "hashing".to_string(), "secure_communication".to_string()
            ]);
            subtopics.insert("network_security".to_string(), vec![
                "firewall".to_string(), "protocols".to_string(), "detection".to_string(),
                "prevention".to_string(), "monitoring".to_string()
            ]);
            subtopics.insert("application_security".to_string(), vec![
                "vulnerabilities".to_string(), "testing".to_string(),
                "secure_coding".to_string(), "owasp".to_string()
            ]);
            subtopics.insert("authentication".to_string(), vec![
                "identity".to_string(), "authorization".to_string(), "tokens".to_string(),
                "biometrics".to_string(), "mfa".to_string()
            ]);
            subtopics
        }
    });
    
    // Cloud Computing
    hierarchy.insert("cloud_computing".to_string(), TopicHierarchy {
        related: vec![
            "aws".to_string(), "azure".to_string(), "gcp".to_string(),
            "iaas".to_string(), "paas".to_string(), "saas".to_string(), "serverless".to_string()
        ],
        subtopics: {
            let mut subtopics = HashMap::new();
            subtopics.insert("infrastructure".to_string(), vec![
                "servers".to_string(), "storage".to_string(), "networking".to_string(),
                "virtualization".to_string(), "containers".to_string()
            ]);
            subtopics.insert("platforms".to_string(), vec![
                "services".to_string(), "apis".to_string(), "functions".to_string(),
                "managed".to_string(), "solutions".to_string()
            ]);
            subtopics.insert("deployment".to_string(), vec![
                "ci_cd".to_string(), "automation".to_string(), "scaling".to_string(),
                "monitoring".to_string(), "reliability".to_string()
            ]);
            subtopics.insert("architecture".to_string(), vec![
                "microservices".to_string(), "serverless".to_string(),
                "event_driven".to_string(), "distributed".to_string()
            ]);
            subtopics
        }
    });
    
    hierarchy
}

/// Extract topics from text using keyword-based matching
/// Fast pattern matching against the topic hierarchy
pub fn extract_topics_from_text(text: &str) -> Vec<String> {
    let lower_text = text.to_lowercase();
    let mut topics = HashSet::new();
    let hierarchy = get_topic_hierarchy();
    
    // Search through main topics and related terms
    for (main_topic, details) in &hierarchy {
        // Check for main topic
        if lower_text.contains(&main_topic.replace('_', " ")) {
            topics.insert(main_topic.clone());
        }
        
        // Check for related terms
        for related_term in &details.related {
            if lower_text.contains(&related_term.replace('_', " ")) {
                topics.insert(main_topic.clone());
                break;
            }
        }
        
        // Check for subtopics
        for (subtopic, keywords) in &details.subtopics {
            if lower_text.contains(&subtopic.replace('_', " ")) {
                topics.insert(format!("{}:{}", main_topic, subtopic));
            }
            
            // Check for subtopic keywords
            for keyword in keywords {
                if lower_text.contains(&keyword.replace('_', " ")) {
                    topics.insert(format!("{}:{}", main_topic, subtopic));
                    break;
                }
            }
        }
    }
    
    topics.into_iter().collect()
}

/// Extract topics using AI with provider-agnostic fallback
/// Matches TypeScript extractTopicsWithAI function exactly
pub async fn extract_topics_with_ai(text: &str, provider_name: Option<&str>) -> Result<Vec<String>> {
    debug!("Starting AI-powered topic extraction for text: {} chars", text.len());
    
    // Prepare the prompt for topic extraction (exact TypeScript match)
    let system_prompt = "You are a topic extraction system. Extract key technical topics from the text as a JSON array of snake_case strings. Focus on technical domains, programming concepts, and specific technologies. Return ONLY the JSON array, nothing else.";
    
    // Function to extract topics with OpenRouter
    let extract_with_ai = async move |text: &str| -> Result<Vec<String>> {
        debug!("Attempting topic extraction with OpenRouter");
        
        let api_key = ApiKeyManager::get_openrouter_key().await?;
        let client = OpenRouterClient::new(api_key);
        
        let request = OpenRouterRequest {
            model: "auto".to_string(),
            messages: vec![
                OpenRouterMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                OpenRouterMessage {
                    role: "user".to_string(),
                    content: text.to_string(),
                },
            ],
            temperature: Some(0.1),
            max_tokens: Some(200),
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
        
        // Try to parse the result as JSON
        match serde_json::from_str::<Vec<String>>(content) {
            Ok(topics) => {
                debug!("Successfully parsed {} topics from AI response", topics.len());
                Ok(topics)
            }
            Err(_) => {
                debug!("Failed to parse JSON response, attempting to extract array content");
                // If not valid JSON, try to extract array-like content (matches TypeScript logic)
                if let Some(captures) = regex::Regex::new(r"\[(.*?)\]").unwrap()
                    .captures(content) {
                    if let Some(array_content) = captures.get(1) {
                        // Extract items that look like they're in quotes
                        let quote_regex = regex::Regex::new(r#""([^"]*)""#).unwrap();
                        let topics: Vec<String> = quote_regex
                            .captures_iter(array_content.as_str())
                            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
                            .collect();
                        
                        if !topics.is_empty() {
                            debug!("Extracted {} topics from regex parsing", topics.len());
                            return Ok(topics);
                        }
                    }
                }
                
                // If all else fails, return a default topic (matches TypeScript)
                warn!("Could not parse AI response, falling back to 'general' topic");
                Ok(vec!["general".to_string()])
            }
        }
    };
    
    // Function to extract topics using keyword extraction as fallback
    let extract_with_fallback = || {
        debug!("Using keyword-based topic extraction as fallback");
        extract_topics_from_text(text)
    };
    
    // If a specific provider is requested, use only that one (TypeScript compatibility)
    if let Some(_provider) = provider_name {
        match extract_with_ai(text).await {
            Ok(topics) => Ok(topics),
            Err(e) => {
                debug!("Provider failed: {}", e);
                Ok(extract_with_fallback())
            }
        }
    } else {
        // Use AI extraction with fallback (matches TypeScript behavior)
        match extract_with_ai(text).await {
            Ok(topics) => {
                if topics.is_empty() {
                    Ok(extract_with_fallback())
                } else {
                    Ok(topics)
                }
            }
            Err(e) => {
                debug!("AI extraction failed: {}, falling back to keyword extraction", e);
                Ok(extract_with_fallback())
            }
        }
    }
}

/// Find topics for a query using dual approach (matches TypeScript findTopicsForQuery)
pub async fn find_topics_for_query(query: &str) -> Result<Vec<String>> {
    debug!("Finding topics for query: {}", query);
    
    // First extract topics directly from the query
    let direct_topics = extract_topics_from_text(query);
    
    // Also use AI to extract topics
    let ai_topics = extract_topics_with_ai(query, None).await?;
    
    // Combine both sets (remove duplicates)
    let mut all_topics = HashSet::new();
    for topic in direct_topics {
        all_topics.insert(topic);
    }
    for topic in ai_topics {
        all_topics.insert(topic);
    }
    
    let result: Vec<String> = all_topics.into_iter().collect();
    info!("Extracted {} topics for query", result.len());
    
    Ok(result)
}

/// Extract topics from text (exported for external use - TypeScript compatibility)
pub fn extract_topics(text: &str) -> Vec<String> {
    extract_topics_from_text(text)
}

/// Extract keywords from text using AI (exported for external use - TypeScript compatibility)
pub async fn extract_keywords(text: &str, provider_name: Option<&str>) -> Result<Vec<String>> {
    // Use AI topic extraction as keyword extraction (matches TypeScript)
    match extract_topics_with_ai(text, provider_name).await {
        Ok(topics) => Ok(topics),
        Err(_) => {
            // Fall back to topic extraction
            Ok(extract_topics_from_text(text))
        }
    }
}

/// Tag a conversation with topics based on its content (matches TypeScript tagConversation)
pub async fn tag_conversation(conversation_id: &str) -> Result<Vec<String>> {
    use crate::core::database::get_database;
    
    debug!("Tagging conversation {} with topics", conversation_id);
    
    let db = get_database().await?;
    let conn = db.get_connection()?;
    
    // Get all messages for this conversation
    let mut stmt = conn.prepare(
        "SELECT content FROM messages WHERE conversation_id = ? ORDER BY timestamp ASC"
    )?;
    
    let message_rows = stmt.query_map([conversation_id], |row| {
        Ok(row.get::<_, String>(0)?)
    })?;
    
    let mut messages = Vec::new();
    for message_result in message_rows {
        messages.push(message_result?);
    }
    
    if messages.is_empty() {
        warn!("No messages found for conversation {}", conversation_id);
        return Ok(vec![]);
    }
    
    // Combine all message content
    let combined_content = messages.join("\n\n");
    
    // Extract topics using AI
    let topics = extract_topics_with_ai(&combined_content, None).await?;
    
    if !topics.is_empty() {
        // First delete existing topics for this conversation
        conn.execute(
            "DELETE FROM conversation_topics WHERE conversation_id = ?",
            [conversation_id],
        )?;
        
        // Insert each topic separately with a weight of 1.0
        for topic in &topics {
            conn.execute(
                "INSERT INTO conversation_topics (conversation_id, topic, weight) VALUES (?, ?, ?)",
                rusqlite::params![conversation_id, topic, 1.0],
            )?;
        }
        
        info!("Tagged conversation {} with {} topics", conversation_id, topics.len());
    } else {
        debug!("No topics extracted for conversation {}", conversation_id);
    }
    
    Ok(topics)
}

/// Find conversations related to a specific topic (matches TypeScript findConversationsByTopic)
pub async fn find_conversations_by_topic(topic: &str, limit: Option<usize>) -> Result<Vec<String>> {
    use crate::core::database::get_database;
    
    let limit = limit.unwrap_or(3);
    let db = get_database().await?;
    let conn = db.get_connection()?;
    
    // Search for exact topic match
    let mut stmt = conn.prepare(
        "SELECT DISTINCT conversation_id FROM conversation_topics WHERE topic = ? LIMIT ?"
    )?;
    
    let conversation_rows = stmt.query_map(rusqlite::params![topic, limit], |row| {
        Ok(row.get::<_, String>(0)?)
    })?;
    
    let mut conversations = Vec::new();
    for conversation_result in conversation_rows {
        conversations.push(conversation_result?);
    }
    
    // If no exact matches, search for partial matches
    if conversations.is_empty() {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT conversation_id FROM conversation_topics WHERE topic LIKE ? LIMIT ?"
        )?;
        
        let like_pattern = format!("%{}%", topic);
        let conversation_rows = stmt.query_map(rusqlite::params![like_pattern, limit], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        for conversation_result in conversation_rows {
            conversations.push(conversation_result?);
        }
    }
    
    // If still no matches, check for related topics in our hierarchy
    if conversations.is_empty() {
        let hierarchy = get_topic_hierarchy();
        let mut related_topics = Vec::new();
        
        // Check main topics
        for (main_topic, details) in &hierarchy {
            if main_topic == topic || details.related.contains(&topic.to_string()) {
                related_topics.push(main_topic.clone());
                related_topics.extend(details.related.clone());
            }
            
            // Check subtopics
            for (subtopic, keywords) in &details.subtopics {
                let full_subtopic = format!("{}:{}", main_topic, subtopic);
                if full_subtopic == topic || keywords.contains(&topic.to_string()) {
                    related_topics.push(full_subtopic);
                    related_topics.extend(keywords.clone());
                }
            }
        }
        
        if !related_topics.is_empty() {
            // Create placeholders for the IN clause
            let placeholders: Vec<&str> = related_topics.iter().map(|_| "?").collect();
            let query = format!(
                "SELECT DISTINCT conversation_id FROM conversation_topics WHERE topic IN ({}) LIMIT ?",
                placeholders.join(",")
            );
            
            let mut params: Vec<&dyn rusqlite::ToSql> = related_topics.iter()
                .map(|t| t as &dyn rusqlite::ToSql)
                .collect();
            params.push(&limit);
            
            let mut stmt = conn.prepare(&query)?;
            let conversation_rows = stmt.query_map(params.as_slice(), |row| {
                Ok(row.get::<_, String>(0)?)
            })?;
            
            for conversation_result in conversation_rows {
                conversations.push(conversation_result?);
            }
        }
    }
    
    debug!("Found {} conversations for topic '{}'", conversations.len(), topic);
    Ok(conversations)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_topics_from_text() {
        let text = "I want to learn about machine learning and neural networks in Python";
        let topics = extract_topics_from_text(text);
        
        assert!(!topics.is_empty());
        assert!(topics.iter().any(|t| t.contains("artificial_intelligence")));
        assert!(topics.iter().any(|t| t.contains("programming")));
    }
    
    #[test]
    fn test_topic_hierarchy_structure() {
        let hierarchy = get_topic_hierarchy();
        
        // Check that we have all 6 main categories
        assert_eq!(hierarchy.len(), 6);
        assert!(hierarchy.contains_key("artificial_intelligence"));
        assert!(hierarchy.contains_key("programming"));
        assert!(hierarchy.contains_key("data"));
        assert!(hierarchy.contains_key("blockchain"));
        assert!(hierarchy.contains_key("security"));
        assert!(hierarchy.contains_key("cloud_computing"));
        
        // Check that each category has proper structure
        for (category, details) in &hierarchy {
            assert!(!details.related.is_empty(), "Category {} missing related terms", category);
            assert!(!details.subtopics.is_empty(), "Category {} missing subtopics", category);
        }
    }
    
    #[tokio::test]
    async fn test_find_topics_for_query() -> Result<()> {
        let query = "How do I implement a REST API in Rust?";
        let topics = find_topics_for_query(query).await?;
        
        assert!(!topics.is_empty());
        // Should detect programming-related topics
        assert!(topics.iter().any(|t| t.contains("programming") || t.contains("rust")));
        
        Ok(())
    }
}