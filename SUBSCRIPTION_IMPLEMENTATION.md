# Subscription System Implementation Plan

## Overview
This document outlines the complete implementation plan for migrating the subscription authentication system from TypeScript (hive.ai) to Rust (hive), maintaining full compatibility with the existing D1 backend.

## Key Components to Implement

### 1. Conversation Gateway (Pre/Post Authorization)
**TypeScript Reference**: `/src/auth/conversation-gateway.ts`
- Pre-conversation authorization with D1
- Post-conversation verification with HMAC
- Question hash generation for verification
- Device fingerprinting and installation tracking

### 2. Usage Tracking System
**TypeScript Reference**: `/src/core/usage-tracker.ts`
- Real-time usage monitoring
- Progressive warning thresholds (50%, 75%, 90%)
- Credit pack consumption when limits reached
- Visual progress bars for usage display

### 3. License Validation
**TypeScript Reference**: `/src/subscription/validator.ts`
- 24-hour cache duration for subscription info
- Fallback to cached data on network failure
- API endpoint configuration

### 4. Subscription Tiers (from website research)
- **Free**: 10 conversations/day
- **Basic**: $5/month - 50 daily, 1,000 monthly
- **Standard**: $10/month - 100 daily, 2,000 monthly  
- **Premium**: $20/month - 200 daily, 4,000 monthly
- **Unlimited**: $30/month - Unlimited
- **Team**: $115/month - 5 developers, 600 daily, 12,000 monthly

### 5. Credit Packs
- **Starter**: 25 credits for $3
- **Value**: 75 credits for $7
- **Power**: 200 credits for $15

## Implementation Phases

### Phase 1: Core Data Structures and D1 Integration
```rust
// src/subscription/mod.rs
pub struct SubscriptionInfo {
    pub user_id: String,
    pub email: String,
    pub tier: SubscriptionTier,
    pub daily_limit: u32,
    pub monthly_limit: u32,
    pub expires_at: DateTime<Utc>,
    pub trial_ends_at: Option<DateTime<Utc>>,
    pub credits_remaining: u32,
}

pub enum SubscriptionTier {
    Free,
    Basic,
    Standard,
    Premium,
    Unlimited,
    Team,
}

// src/auth/conversation_gateway.rs
pub struct ConversationGateway {
    api_url: String,
    http_client: reqwest::Client,
}

pub struct ConversationAuthorization {
    pub conversation_token: String,
    pub question_hash: String,
    pub user_id: String,
    pub remaining: u32,
    pub limit: u32,
    pub expires_at: DateTime<Utc>,
}
```

**QA Steps**:
1. Run `cargo build` to ensure structures compile
2. Write unit tests for data structure serialization/deserialization
3. Test D1 API connection with mock endpoints

### Phase 2: Pre-Conversation Authorization
Implement the pre-conversation flow that checks usage limits before allowing consensus:

```rust
impl ConversationGateway {
    pub async fn request_conversation_authorization(&self, question: &str) -> Result<ConversationAuthorization> {
        // 1. Get license key from local storage
        // 2. Generate question hash
        // 3. Get installation ID (device fingerprint)
        // 4. Call D1 /auth/pre-conversation endpoint
        // 5. Handle usage limit errors with proper messages
        // 6. Return authorization token
    }
}
```

**QA Steps**:
1. Test with valid license key - should authorize
2. Test with exhausted daily limit - should show upgrade prompt
3. Test with invalid license - should show configuration prompt
4. Verify question hash generation is consistent

### Phase 3: Usage Tracking and Warnings
Port the usage tracking system with progressive warnings:

```rust
pub struct UsageTracker {
    subscription: SubscriptionInfo,
    daily_used: u32,
    monthly_used: u32,
}

impl UsageTracker {
    pub fn check_usage_before_conversation(&self) -> (bool, Option<UsageNotification>) {
        // Calculate usage percentages
        // Return appropriate warnings at 50%, 75%, 90%
        // Check credit usage when limits exceeded
    }
}
```

**QA Steps**:
1. Test warning messages at each threshold
2. Verify credit consumption logic
3. Test visual progress bar generation
4. Ensure free tier gets moderate usage prompt at 50%

### Phase 4: Post-Conversation Verification
Implement HMAC-based verification after consensus completion:

```rust
impl ConversationGateway {
    pub async fn report_conversation_completion(
        &self,
        token: &str,
        conversation_id: &str,
        question_hash: &str
    ) -> Result<ConversationVerification> {
        // Generate HMAC proof
        // Call D1 /auth/post-conversation
        // Update local usage counters
    }
}
```

**QA Steps**:
1. Verify HMAC generation matches TypeScript
2. Test successful conversation reporting
3. Test network failure handling (should queue for retry)
4. Ensure usage counters update correctly

### Phase 5: Email Reminder System
Since email system isn't in TypeScript codebase, we'll implement a new system:

```rust
pub struct SubscriptionReminder {
    pub user_id: String,
    pub email: String,
    pub expires_at: DateTime<Utc>,
    pub reminder_sent: HashSet<u8>, // Track 3, 2, 1 day reminders
}

impl SubscriptionReminder {
    pub async fn check_and_send_reminders(&self) -> Result<()> {
        // Calculate days until expiration
        // Send appropriate reminder if not already sent
        // Update reminder_sent tracking
    }
}
```

**QA Steps**:
1. Test reminder triggers at 3, 2, 1 days
2. Verify emails aren't sent multiple times
3. Test email template rendering
4. Check reminder persistence across restarts

### Phase 6: Bottom Bar UI Integration
Add subscription info to the TUI bottom bar:

```rust
// In src/tui/components/bottom_bar.rs
pub struct SubscriptionDisplay {
    username: String,
    tier: SubscriptionTier,
    daily_remaining: u32,
    daily_limit: u32,
}

impl SubscriptionDisplay {
    pub fn render(&self) -> String {
        // Format: "user@email.com | Premium | 180/200 daily"
        // Special handling for unlimited tier
        // Show credits if using them
    }
}
```

**QA Steps**:
1. Test display for each subscription tier
2. Verify real-time updates after conversations
3. Test credit display when over limit
4. Ensure proper truncation for long emails

### Phase 7: Daily License Check
Implement background license validation:

```rust
pub struct LicenseMonitor {
    last_check: DateTime<Utc>,
    check_interval: Duration,
}

impl LicenseMonitor {
    pub async fn check_for_changes(&self) -> Result<LicenseChange> {
        // Compare cached license with fresh validation
        // Detect: renewal, upgrade, downgrade, cancellation
        // Update local cache
        // Trigger UI updates if needed
    }
}
```

**QA Steps**:
1. Test plan upgrade detection
2. Test downgrade handling
3. Test cancellation to free tier transition
4. Verify UI updates on plan change

### Phase 8: Trial Period Logic
Implement 7-day unlimited trial:

```rust
impl SubscriptionInfo {
    pub fn is_in_trial(&self) -> bool {
        // Check if trial_ends_at is in future
        // All paid tiers get 7-day trial
    }
    
    pub fn effective_daily_limit(&self) -> u32 {
        if self.is_in_trial() {
            u32::MAX // Unlimited during trial
        } else {
            self.daily_limit
        }
    }
}
```

**QA Steps**:
1. Test trial activation on new subscription
2. Verify unlimited access during trial
3. Test transition from trial to regular limits
4. Ensure trial status shows in UI

## Migration Strategy

### Step 1: Database Schema
Ensure our SQLite schema matches what D1 expects:
```sql
-- Subscription cache table
CREATE TABLE IF NOT EXISTS subscription_cache (
    user_id TEXT PRIMARY KEY,
    subscription_data TEXT NOT NULL,
    cached_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL
);

-- Usage tracking table  
CREATE TABLE IF NOT EXISTS usage_tracking (
    date TEXT NOT NULL,
    conversations_used INTEGER DEFAULT 0,
    credits_used INTEGER DEFAULT 0,
    PRIMARY KEY (date)
);
```

### Step 2: Configuration Migration
Read existing TypeScript config and migrate:
```rust
pub async fn migrate_typescript_config() -> Result<()> {
    // Read ~/.hive/config.json
    // Extract license key
    // Save to new Rust config format
    // Preserve all settings
}
```

### Step 3: API Endpoint Configuration
Use the same endpoints as TypeScript:
- General API: Configured endpoint
- Pre-conversation: `/auth/pre-conversation`
- Post-conversation: `/auth/post-conversation`
- License validation: `/v1/session/validate`

## Testing Plan

### Integration Tests
1. **Full Conversation Flow**:
   - Authorize → Run consensus → Verify completion
   - Test with different subscription tiers
   - Test limit exhaustion scenarios

2. **Network Failure Handling**:
   - Test offline mode with cached data
   - Test retry logic for failed verifications
   - Test graceful degradation

3. **Plan Change Scenarios**:
   - Upgrade from free to paid
   - Downgrade from paid to free
   - Credit pack purchase and usage

### Performance Requirements
- Pre-authorization: < 200ms (excluding network)
- Cache lookups: < 5ms
- UI updates: < 16ms (60 FPS)

## Commit Plan
After each successful QA phase:
1. Phase 1: `feat(subscription): add core data structures and D1 integration`
2. Phase 2: `feat(auth): implement pre-conversation authorization flow`
3. Phase 3: `feat(usage): add usage tracking with progressive warnings`
4. Phase 4: `feat(auth): implement post-conversation verification`
5. Phase 5: `feat(subscription): add email reminder system`
6. Phase 6: `feat(ui): add subscription info to bottom bar`
7. Phase 7: `feat(subscription): implement daily license monitoring`
8. Phase 8: `feat(subscription): add 7-day unlimited trial logic`

## Security Considerations
1. Never log license keys or tokens
2. Use secure storage for sensitive data
3. Validate all D1 responses
4. Implement rate limiting for API calls
5. Clear sensitive data from memory after use

## Notes from TypeScript Implementation
1. Device fingerprinting uses platform, arch, CPUs, and memory
2. HMAC verification uses SHA256 with conversation token as key
3. Question normalization: trim and lowercase before hashing
4. 24-hour cache duration for subscription info
5. Conversation tokens expire after 1 hour
6. Installation ID is first 16 chars of device hash
7. Usage proof format: `{conversation_id}:{question_hash}`

This implementation maintains 100% compatibility with the existing D1 backend while improving performance and adding the missing email reminder system.