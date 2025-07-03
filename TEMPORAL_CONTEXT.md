# Temporal Context System: Claude Code-Inspired Web Search Enhancement

## Overview

HiveTechs Consensus implements advanced temporal context awareness inspired by Claude Code's web search enhancement. While we depend on OpenRouter models for actual web search capabilities, we significantly enhance the context we provide to make their responses more current and accurate.

## Core Philosophy

**"Give models the date context they need to search intelligently"**

Unlike Claude Code which has built-in web search, we work through OpenRouter's AI models. Our advantage is **intelligent context injection** that makes any web-search-capable model more effective.

## Architecture

### Temporal Context Provider

```rust
pub struct TemporalContextProvider {
    timezone: chrono_tz::Tz,
    business_calendar: Arc<BusinessCalendar>,
    market_calendar: Arc<MarketCalendar>,
}

impl TemporalContextProvider {
    pub async fn build_current_context(&self) -> Result<TemporalContext> {
        let now = chrono::Utc::now().with_timezone(&self.timezone);
        
        TemporalContext {
            // Direct AI instruction
            search_instruction: format!(
                "IMPORTANT: Today's date is {} ({}). When performing web searches or looking up current information, prioritize results from {} onwards and clearly indicate if information might be outdated.",
                now.format("%A, %B %d, %Y"),
                now.format("%Y-%m-%d"),
                now.year()
            ),
            
            // Temporal awareness for models
            temporal_awareness: format!(
                "Current context: {} at {} ({}). Use this when interpreting 'recent', 'latest', 'current', 'today', or any time-sensitive queries.",
                now.format("%A, %B %d, %Y"),
                now.format("%H:%M %Z"),
                now.to_rfc3339()
            ),
            
            // Rich contextual data
            current_date: now.format("%Y-%m-%d").to_string(),
            current_datetime: now.format("%A, %B %d, %Y at %H:%M:%S %Z").to_string(),
            business_context: self.build_business_context(&now).await?,
        }
    }
}
```

### Smart Detection System

The system automatically detects when queries require temporal context:

```rust
fn requires_temporal_context(&self, query: &str) -> bool {
    let temporal_indicators = [
        // Direct temporal requests
        "latest", "recent", "current", "today", "now", "this week", "this month",
        "2024", "2025", "what's new", "recent updates", "current version",
        
        // Web search indicators
        "search", "lookup", "find online", "google", "web search", "internet",
        "news", "trends", "happening", "breaking", "announcement",
        
        // Version/release indicators  
        "latest release", "new features", "just released", "recently published",
        "cutting edge", "state of the art", "bleeding edge",
        
        // Market/business indicators
        "stock price", "market", "earnings", "financial", "trading"
    ];
    
    let query_lower = query.to_lowercase();
    temporal_indicators.iter().any(|&indicator| query_lower.contains(indicator))
}
```

## Integration Points

### 1. Generator Stage Enhancement

When the consensus pipeline's Generator stage processes a query requiring current information:

```rust
// Before sending to OpenRouter models
if temporal_context.is_some() {
    prompt = format!(
        "{}\n\n{}\n\nUser Query: {}",
        temporal_context.search_instruction,
        temporal_context.temporal_awareness,
        original_query
    );
}
```

### 2. Context Builder Integration

```rust
impl ContextBuilder {
    pub async fn build_context(&self, location: Location, query: Option<&str>) -> Context {
        // ... existing context layers ...
        
        // Layer 5: Temporal context (for current information & web search)
        if let Some(query_text) = query {
            if self.requires_temporal_context(query_text) {
                let temporal_context = self.temporal_provider.build_current_context().await?;
                context.add_temporal(temporal_context);
            }
        }
        
        context
    }
}
```

## Business Context Enhancement

Beyond basic date awareness, we provide rich business context:

```rust
pub struct BusinessContext {
    pub is_business_day: bool,
    pub is_weekend: bool,
    pub market_session: MarketSession,     // Pre, Regular, Post, Closed
    pub is_holiday: bool,
    pub quarter_end_days: i64,
    pub is_earnings_season: bool,
    pub next_market_open: Option<DateTime>,
}
```

## Example Implementations

### Stock Price Query
```
User: "What's NVIDIA's current stock price?"

Temporal Context Injection:
"IMPORTANT: Today's date is Tuesday, July 2, 2024 (2024-07-02). 
Market context: Regular trading session, business day.
When looking up stock prices, get the most recent trading data 
available for this date."
```

### Technology Search
```
User: "What are the latest Rust features?"

Temporal Context Injection:
"IMPORTANT: Today's date is Tuesday, July 2, 2024 (2024-07-02).
When searching for 'latest' Rust features, prioritize information 
from 2024 onwards. Include release dates and indicate which 
features are newest relative to this date."
```

### News and Trends
```
User: "Search for recent AI developments"

Temporal Context Injection:
"IMPORTANT: Today's date is Tuesday, July 2, 2024 (2024-07-02).
When searching for 'recent' AI developments, focus on information 
from the past 3-6 months (2024) and clearly indicate the 
publication dates of sources."
```

## Hook Integration

The temporal context system integrates with the enterprise hooks system:

```json
{
  "id": "temporal-context-logger",
  "events": ["BeforeGeneratorStage"],
  "conditions": [{"ContentMatches": "temporal_context"}],
  "actions": [{
    "LogEvent": "Info",
    "message": "Temporal context injected for current information query"
  }]
}
```

## Configuration

```toml
[temporal_context]
enabled = true
timezone = "UTC"  # or "America/New_York", etc.
business_calendar = "NYSE"  # NYSE, NASDAQ, LSE, etc.

[temporal_context.detection]
# Additional keywords for temporal detection
custom_indicators = ["breaking", "update", "announcement"]

# Disable for specific model patterns
skip_models = ["gpt-3.5-turbo"]  # Models that don't benefit from temporal context

[temporal_context.market]
# Market-specific settings
include_market_hours = true
include_earnings_calendar = false  # Premium feature
```

## CLI Commands

```bash
# Test temporal context detection
hive temporal test "What's the latest in AI?"
# Output: ✅ Temporal context would be injected

# Show current temporal context
hive temporal show
# Current date: Tuesday, July 2, 2024 (2024-07-02)
# Business day: Yes
# Market session: Regular (9:30 AM - 4:00 PM EST)

# Configure temporal settings
hive config set temporal_context.timezone "America/New_York"
hive config set temporal_context.enabled true
```

## Advantages Over Claude Code

1. **Model Agnostic**: Works with any OpenRouter model that can perform web searches
2. **Business Intelligence**: Rich business calendar and market context
3. **Configurable**: Timezone, calendar, and detection customization
4. **Hook Integration**: Enterprise automation for temporal queries
5. **Multiple Model Support**: Leverage different models' web search strengths

## Performance Considerations

- **Zero Latency**: Temporal context generation is sub-millisecond
- **Caching**: Business calendar data cached for 24 hours
- **Smart Detection**: Only activates for queries that need it
- **Configurable**: Can be disabled for performance-critical applications

## Future Enhancements

1. **Regional Calendars**: Support for international business calendars
2. **Event Integration**: Holiday and event awareness
3. **Timezone Intelligence**: Automatic timezone detection from user location
4. **Historical Context**: "What was happening on this date last year?"
5. **Industry Calendars**: Earnings seasons, product launch cycles, etc.

This system ensures that HiveTechs Consensus provides the most current and contextually aware responses possible, matching and exceeding Claude Code's temporal awareness capabilities while leveraging the full power of OpenRouter's model ecosystem.