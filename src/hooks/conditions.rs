//! Hook Conditions - Conditional evaluation for hook triggering

use std::path::PathBuf;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use regex::Regex;
use super::ExecutionContext;

/// Conditions that must be met for a hook to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookCondition {
    /// File pattern matching
    FilePattern {
        pattern: String,
        #[serde(default)]
        negate: bool,
    },
    
    /// File size constraints
    FileSize {
        #[serde(flatten)]
        constraint: SizeConstraint,
    },
    
    /// Environment variable check
    EnvironmentVariable {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<String>,
        #[serde(default)]
        exists: bool,
    },
    
    /// Context variable check
    ContextVariable {
        key: String,
        operator: ComparisonOperator,
        value: Value,
    },
    
    /// Time-based conditions
    TimeWindow {
        start_time: Option<String>, // HH:MM format
        end_time: Option<String>,   // HH:MM format
        days: Option<Vec<String>>,  // ["monday", "tuesday", ...]
        timezone: Option<String>,   // e.g., "America/New_York"
    },
    
    /// Repository conditions
    Repository {
        #[serde(skip_serializing_if = "Option::is_none")]
        has_file: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        branch_pattern: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_clean: Option<bool>,
    },
    
    /// Cost threshold
    CostThreshold {
        max_cost: f64,
        currency: String,
    },
    
    /// Custom expression
    Expression {
        expression: String, // Simple expression language
    },
    
    /// Logical operators
    And {
        conditions: Vec<HookCondition>,
    },
    Or {
        conditions: Vec<HookCondition>,
    },
    Not {
        condition: Box<HookCondition>,
    },
}

/// Size constraint for file size conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operator", rename_all = "snake_case")]
pub enum SizeConstraint {
    LessThan { size: String },
    GreaterThan { size: String },
    Between { min: String, max: String },
}

/// Comparison operators for variable conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOperator {
    Equals,
    NotEquals,
    Contains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    Matches, // Regex match
}

/// Evaluates hook conditions
pub struct ConditionEvaluator {
    file_pattern_cache: std::sync::Mutex<lru::LruCache<String, Regex>>,
}

impl ConditionEvaluator {
    pub fn new() -> Self {
        Self {
            file_pattern_cache: std::sync::Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(100).unwrap()
            )),
        }
    }
    
    /// Evaluate all conditions for a hook
    pub async fn evaluate(&self, conditions: &[HookCondition], context: &ExecutionContext) -> Result<bool> {
        if conditions.is_empty() {
            return Ok(true); // No conditions means always execute
        }
        
        // Evaluate all conditions (AND by default)
        for condition in conditions {
            if !self.evaluate_condition(condition, context).await? {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Evaluate a single condition
    async fn evaluate_condition(&self, condition: &HookCondition, context: &ExecutionContext) -> Result<bool> {
        match condition {
            HookCondition::FilePattern { pattern, negate } => {
                let result = self.evaluate_file_pattern(pattern, context)?;
                Ok(if *negate { !result } else { result })
            }
            
            HookCondition::FileSize { constraint } => {
                self.evaluate_file_size(constraint, context)
            }
            
            HookCondition::EnvironmentVariable { name, value, exists } => {
                self.evaluate_env_var(name, value.as_deref(), *exists)
            }
            
            HookCondition::ContextVariable { key, operator, value } => {
                self.evaluate_context_var(key, operator, value, context)
            }
            
            HookCondition::TimeWindow { start_time, end_time, days, timezone } => {
                self.evaluate_time_window(start_time.as_deref(), end_time.as_deref(), days.as_deref(), timezone.as_deref())
            }
            
            HookCondition::Repository { has_file, branch_pattern, is_clean } => {
                self.evaluate_repository(has_file.as_deref(), branch_pattern.as_deref(), *is_clean).await
            }
            
            HookCondition::CostThreshold { max_cost, currency } => {
                self.evaluate_cost_threshold(*max_cost, currency, context)
            }
            
            HookCondition::Expression { expression } => {
                self.evaluate_expression(expression, context)
            }
            
            HookCondition::And { conditions } => {
                for cond in conditions {
                    let result = Box::pin(self.evaluate_condition(cond, context)).await?;
                    if !result {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            
            HookCondition::Or { conditions } => {
                for cond in conditions {
                    let result = Box::pin(self.evaluate_condition(cond, context)).await?;
                    if result {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            
            HookCondition::Not { condition } => {
                let result = Box::pin(self.evaluate_condition(condition, context)).await?;
                Ok(!result)
            }
        }
    }
    
    /// Evaluate file pattern condition
    fn evaluate_file_pattern(&self, pattern: &str, context: &ExecutionContext) -> Result<bool> {
        let file_path = context.get_variable("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No file_path in context"))?;
        
        // Get or compile regex
        let regex = {
            let mut cache = self.file_pattern_cache.lock().unwrap();
            if let Some(regex) = cache.get(pattern) {
                regex.clone()
            } else {
                // Convert glob pattern to regex
                let regex_pattern = glob_to_regex(pattern);
                let regex = Regex::new(&regex_pattern)?;
                cache.put(pattern.to_string(), regex.clone());
                regex
            }
        };
        
        Ok(regex.is_match(file_path))
    }
    
    /// Evaluate file size condition
    fn evaluate_file_size(&self, constraint: &SizeConstraint, context: &ExecutionContext) -> Result<bool> {
        let file_path = context.get_variable("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No file_path in context"))?;
        
        let metadata = std::fs::metadata(file_path)?;
        let file_size = metadata.len();
        
        match constraint {
            SizeConstraint::LessThan { size } => {
                let max_size = parse_size(size)?;
                Ok(file_size < max_size)
            }
            SizeConstraint::GreaterThan { size } => {
                let min_size = parse_size(size)?;
                Ok(file_size > min_size)
            }
            SizeConstraint::Between { min, max } => {
                let min_size = parse_size(min)?;
                let max_size = parse_size(max)?;
                Ok(file_size >= min_size && file_size <= max_size)
            }
        }
    }
    
    /// Evaluate environment variable condition
    fn evaluate_env_var(&self, name: &str, value: Option<&str>, exists: bool) -> Result<bool> {
        match std::env::var(name) {
            Ok(env_value) => {
                if exists {
                    Ok(true)
                } else if let Some(expected) = value {
                    Ok(env_value == expected)
                } else {
                    Ok(true)
                }
            }
            Err(_) => Ok(!exists),
        }
    }
    
    /// Evaluate context variable condition
    fn evaluate_context_var(&self, key: &str, operator: &ComparisonOperator, expected: &Value, context: &ExecutionContext) -> Result<bool> {
        let actual = context.get_variable(key);
        
        match operator {
            ComparisonOperator::Equals => {
                Ok(actual == Some(expected))
            }
            ComparisonOperator::NotEquals => {
                Ok(actual != Some(expected))
            }
            ComparisonOperator::Contains => {
                if let (Some(Value::String(s)), Value::String(pattern)) = (actual, expected) {
                    Ok(s.contains(pattern))
                } else if let (Some(Value::Array(arr)), _) = (actual, expected) {
                    Ok(arr.contains(expected))
                } else {
                    Ok(false)
                }
            }
            ComparisonOperator::StartsWith => {
                if let (Some(Value::String(s)), Value::String(prefix)) = (actual, expected) {
                    Ok(s.starts_with(prefix))
                } else {
                    Ok(false)
                }
            }
            ComparisonOperator::EndsWith => {
                if let (Some(Value::String(s)), Value::String(suffix)) = (actual, expected) {
                    Ok(s.ends_with(suffix))
                } else {
                    Ok(false)
                }
            }
            ComparisonOperator::GreaterThan => {
                self.compare_values(actual, expected, |a, b| a > b)
            }
            ComparisonOperator::LessThan => {
                self.compare_values(actual, expected, |a, b| a < b)
            }
            ComparisonOperator::GreaterOrEqual => {
                self.compare_values(actual, expected, |a, b| a >= b)
            }
            ComparisonOperator::LessOrEqual => {
                self.compare_values(actual, expected, |a, b| a <= b)
            }
            ComparisonOperator::Matches => {
                if let (Some(Value::String(s)), Value::String(pattern)) = (actual, expected) {
                    let regex = Regex::new(pattern)?;
                    Ok(regex.is_match(s))
                } else {
                    Ok(false)
                }
            }
        }
    }
    
    /// Compare numeric values
    fn compare_values<F>(&self, actual: Option<&Value>, expected: &Value, op: F) -> Result<bool>
    where
        F: Fn(f64, f64) -> bool,
    {
        if let (Some(Value::Number(a)), Value::Number(b)) = (actual, expected) {
            if let (Some(a_f64), Some(b_f64)) = (a.as_f64(), b.as_f64()) {
                return Ok(op(a_f64, b_f64));
            }
        }
        Ok(false)
    }
    
    /// Evaluate time window condition
    fn evaluate_time_window(&self, start_time: Option<&str>, end_time: Option<&str>, days: Option<&[String]>, timezone: Option<&str>) -> Result<bool> {
        use chrono::{Local, Timelike, Datelike};
        
        let now = if let Some(tz_str) = timezone {
            let tz: chrono_tz::Tz = tz_str.parse()?;
            chrono::Utc::now().with_timezone(&tz).naive_local()
        } else {
            Local::now().naive_local()
        };
        
        // Check day of week
        if let Some(allowed_days) = days {
            let current_day = now.weekday().to_string().to_lowercase();
            if !allowed_days.iter().any(|d| d.to_lowercase() == current_day) {
                return Ok(false);
            }
        }
        
        // Check time window
        if let (Some(start), Some(end)) = (start_time, end_time) {
            let current_time = format!("{:02}:{:02}", now.time().hour(), now.time().minute());
            Ok(current_time >= start && current_time <= end)
        } else {
            Ok(true)
        }
    }
    
    /// Evaluate repository condition
    async fn evaluate_repository(&self, has_file: Option<&str>, branch_pattern: Option<&str>, is_clean: Option<bool>) -> Result<bool> {
        // Check for file existence
        if let Some(file) = has_file {
            if !PathBuf::from(file).exists() {
                return Ok(false);
            }
        }
        
        // Check branch pattern
        if let Some(pattern) = branch_pattern {
            let output = tokio::process::Command::new("git")
                .args(&["rev-parse", "--abbrev-ref", "HEAD"])
                .output()
                .await?;
            
            if output.status.success() {
                let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let regex = Regex::new(pattern)?;
                if !regex.is_match(&branch) {
                    return Ok(false);
                }
            }
        }
        
        // Check if repository is clean
        if let Some(clean) = is_clean {
            let output = tokio::process::Command::new("git")
                .args(&["status", "--porcelain"])
                .output()
                .await?;
            
            if output.status.success() {
                let is_actually_clean = output.stdout.is_empty();
                if is_actually_clean != clean {
                    return Ok(false);
                }
            }
        }
        
        Ok(true)
    }
    
    /// Evaluate cost threshold condition
    fn evaluate_cost_threshold(&self, max_cost: f64, currency: &str, context: &ExecutionContext) -> Result<bool> {
        if let Some(cost) = context.get_variable("estimated_cost").and_then(|v| v.as_f64()) {
            // In a real implementation, we'd convert currencies if needed
            Ok(cost <= max_cost)
        } else {
            Ok(true) // No cost information, allow execution
        }
    }
    
    /// Evaluate custom expression
    fn evaluate_expression(&self, expression: &str, context: &ExecutionContext) -> Result<bool> {
        // Simple expression evaluation
        // In a real implementation, this would use a proper expression parser
        
        // For now, support simple variable checks
        if expression.contains("==") {
            let parts: Vec<&str> = expression.split("==").collect();
            if parts.len() == 2 {
                let var_name = parts[0].trim().trim_start_matches('$');
                let expected = parts[1].trim().trim_matches('"');
                
                if let Some(value) = context.get_variable(var_name) {
                    if let Some(s) = value.as_str() {
                        return Ok(s == expected);
                    }
                }
            }
        }
        
        // Default to true for unsupported expressions
        Ok(true)
    }
}

/// Convert glob pattern to regex
fn glob_to_regex(pattern: &str) -> String {
    let mut regex = String::new();
    regex.push('^');
    
    for ch in pattern.chars() {
        match ch {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            '.' | '+' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '\\' => {
                regex.push('\\');
                regex.push(ch);
            }
            _ => regex.push(ch),
        }
    }
    
    regex.push('$');
    regex
}

/// Parse size string (e.g., "10MB", "1.5GB") to bytes
fn parse_size(size_str: &str) -> Result<u64> {
    let size_str = size_str.trim().to_uppercase();
    
    let (num_str, unit) = if size_str.ends_with("KB") {
        (&size_str[..size_str.len()-2], 1024u64)
    } else if size_str.ends_with("MB") {
        (&size_str[..size_str.len()-2], 1024u64 * 1024)
    } else if size_str.ends_with("GB") {
        (&size_str[..size_str.len()-2], 1024u64 * 1024 * 1024)
    } else if size_str.ends_with("TB") {
        (&size_str[..size_str.len()-2], 1024u64 * 1024 * 1024 * 1024)
    } else if size_str.ends_with("B") {
        (&size_str[..size_str.len()-1], 1u64)
    } else {
        (size_str.as_str(), 1u64)
    };
    
    let num: f64 = num_str.parse()?;
    Ok((num * unit as f64) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_glob_to_regex() {
        assert_eq!(glob_to_regex("*.rs"), r"^.*\.rs$");
        assert_eq!(glob_to_regex("test?.txt"), r"^test.\.txt$");
        assert_eq!(glob_to_regex("src/**/*.rs"), r"^src/.*/.*/.*\.rs$");
    }
    
    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("100").unwrap(), 100);
        assert_eq!(parse_size("10KB").unwrap(), 10 * 1024);
        assert_eq!(parse_size("5MB").unwrap(), 5 * 1024 * 1024);
        assert_eq!(parse_size("1.5GB").unwrap(), (1.5 * 1024.0 * 1024.0 * 1024.0) as u64);
    }
}