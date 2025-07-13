# D1 API Fix for Rate-Limited Users

## Problem
The D1 API currently returns 401 Unauthorized errors for valid users who have hit their daily conversation limit. This is incorrect behavior - these users have valid licenses and should receive a 200 OK response with user information and `remaining = 0`.

## Root Cause
In `secure-worker.js`, both `handlePreConversation` and `handleSessionValidation` functions are checking for `account_status = 'active'` in their SQL queries. This causes valid users to be rejected when they should just be rate-limited.

## Fix Required

### 1. Update `verifyAuth` function (line ~143)
Change from:
```javascript
const user = await env.DB.prepare(
  'SELECT license_key, id, account_status FROM users WHERE license_key = ? AND account_status = ?'
).bind(token, 'active').first();
```

To:
```javascript
const user = await env.DB.prepare(
  'SELECT license_key, id, account_status FROM users WHERE license_key = ?'
).bind(token).first();

// Check if account is truly inactive (not just rate-limited)
if (!user || (user.account_status !== 'active' && user.account_status !== 'rate_limited')) {
  return { valid: false, error: 'Invalid or inactive license' };
}
```

### 2. Update `handleSessionValidation` function (line ~760)
Change from:
```javascript
const user = await env.DB.prepare(
  'SELECT * FROM users WHERE license_key = ? AND account_status = ?'
).bind(data.session_token, 'active').first();

if (!user) {
  return new Response(JSON.stringify({
    error: 'Invalid session token'
  }), {
    status: 401,
    headers: { 'Content-Type': 'application/json' }
  });
}
```

To:
```javascript
const user = await env.DB.prepare(
  'SELECT * FROM users WHERE license_key = ?'
).bind(data.session_token).first();

if (!user) {
  return new Response(JSON.stringify({
    error: 'Invalid session token'
  }), {
    status: 401,
    headers: { 'Content-Type': 'application/json' }
  });
}

// Get usage information for the user
const usageCheck = await checkUsageAllowed(env.DB, user.license_key);

// Return full user profile with usage information
return new Response(JSON.stringify({
  valid: true,
  user_id: user.id,
  email: user.email,
  tier: user.subscription_plan_id || 'free',
  limits: {
    daily: usageCheck.limit === 'unlimited' ? 999999 : usageCheck.limit
  },
  usage: {
    remaining: usageCheck.remaining === 'unlimited' ? 999999 : usageCheck.remaining,
    limit: usageCheck.limit === 'unlimited' ? 999999 : usageCheck.limit
  },
  features: ['consensus'],
  status: 'active'
}), {
  status: 200,
  headers: { 'Content-Type': 'application/json' }
});
```

### 3. Update `handlePreConversation` to include user info (line ~250)
When usage is not allowed, include user information in the response:

Change from:
```javascript
if (!usageCheck.allowed) {
  return new Response(JSON.stringify({
    allowed: false,
    error: usageCheck.error,
    plan: usageCheck.plan,
    used_conversations: usageCheck.used,
    plan_limit: usageCheck.limit,
    type: usageCheck.type
  }), {
    status: 403,
    headers: { 'Content-Type': 'application/json' }
  });
}
```

To:
```javascript
// Get user info for response
const user = await getUserByLicense(env.DB, license_key);

if (!usageCheck.allowed) {
  return new Response(JSON.stringify({
    allowed: false,
    error: usageCheck.error,
    plan: usageCheck.plan,
    used_conversations: usageCheck.used,
    plan_limit: usageCheck.limit,
    type: usageCheck.type,
    user: {
      id: user.id,
      email: user.email,
      subscription_tier: user.subscription_plan_id || 'free'
    },
    user_id: user.id,
    email: user.email,
    remaining: 0,
    remaining_conversations: 0
  }), {
    status: 403,
    headers: { 'Content-Type': 'application/json' }
  });
}
```

### 4. Also update the success response to include user email:
```javascript
return new Response(JSON.stringify({
  allowed: true,
  conversation_token: conversationToken,
  remaining_conversations: usageCheck.remaining,
  plan_limit: usageCheck.limit,
  expires_at: expiresAt.toISOString(),
  user: {
    id: user.id,
    email: user.email,
    subscription_tier: user.subscription_plan_id || 'free'
  },
  user_id: user.id,
  email: user.email
}), {
  status: 200,
  headers: { 'Content-Type': 'application/json' }
});
```

## Expected Behavior After Fix

1. **Valid user with daily limit reached**:
   - Status: 403 Forbidden (not 401)
   - Response includes user email, tier, and usage info
   - Shows `remaining: 0` and appropriate error message

2. **Valid user with remaining conversations**:
   - Status: 200 OK
   - Response includes conversation token and user info
   - Shows correct remaining count

3. **Invalid or truly inactive user**:
   - Status: 401 Unauthorized
   - Generic error message

## Testing
After implementing these changes:
1. Deploy the updated worker
2. Test with both license keys:
   - Limited account should get 403 with user info when limit reached
   - Unlimited account should always get 200 with user info