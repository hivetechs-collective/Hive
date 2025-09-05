/**
 * Enhanced Hive.AI Cloudflare Worker
 * Implements secure usage tracking with anti-tampering measures
 */

// Use Web Crypto API for Cloudflare Workers compatibility
import { createHmac, createHash, randomBytes } from './crypto-utils.js';
import jwt from '@tsndr/cloudflare-worker-jwt';

// JWT Service Authentication Validation
async function validateServiceJWT(request, env) {
  try {
    const authHeader = request.headers.get('Authorization');
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      return { valid: false, error: 'Missing or invalid Authorization header' };
    }
    
    const token = authHeader.replace('Bearer ', '');
    
    // Verify JWT token
    const isValid = await jwt.verify(token, env.JWT_SERVICE_SECRET);
    if (!isValid) {
      return { valid: false, error: 'Invalid JWT signature' };
    }
    
    // Decode payload
    const decoded = jwt.decode(token);
    
    // Validate issuer and audience
    if (decoded.payload.iss !== 'hivetechs-website' || 
        decoded.payload.aud !== 'gateway.hivetechs.io') {
      return { valid: false, error: 'Invalid issuer or audience' };
    }
    
    // Validate service and permissions
    if (decoded.payload.service !== 'hivetechs-website') {
      return { valid: false, error: 'Invalid service identifier' };
    }
    
    return { valid: true, decoded: decoded.payload };
  } catch (error) {
    console.error('JWT validation error:', error);
    return { valid: false, error: 'Invalid JWT token' };
  }
}

export default {
  async fetch(request, env, ctx) {
    return await handleRequest(request, env, ctx);
  }
};

async function handleRequest(request, env, ctx) {
  const url = new URL(request.url);
  const path = url.pathname;
  
  // CORS headers
  const corsHeaders = {
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type, Authorization',
  };

  // Handle CORS preflight
  if (request.method === 'OPTIONS') {
    return new Response(null, { headers: corsHeaders });
  }

  try {
    let response;
    
    switch (path) {
      case '/auth/install':
        response = await handleInstallRegistration(request, env);
        break;
      case '/auth/pre-conversation':
        response = await handlePreConversation(request, env);
        break;
      case '/auth/post-conversation':
        response = await handlePostConversation(request, env);
        break;
      case '/v1/session/validate':
        response = await handleSessionValidation(request, env);
        break;
      case '/verify-subscription':
        response = await handleVerifySubscription(request, env);
        break;
      case '/sync-usage':
        response = await handleSyncUsage(request, env);
        break;
      case '/create-checkout':
        response = await handleCreateCheckout(request, env);
        break;
      case '/webhooks/paddle':
        response = await handlePaddleWebhook(request, env);
        break;
      case '/api/license/retrieve':
        response = await handleLicenseRetrieve(request, env);
        break;
      case '/api/users/create':
        response = await handleUserCreate(request, env);
        break;
      case '/api/users/getByLicense':
        response = await handleGetUserByLicense(request, env);
        break;
      default:
        response = new Response('Not Found', { status: 404 });
    }

    // Add CORS headers to response
    Object.entries(corsHeaders).forEach(([key, value]) => {
      response.headers.set(key, value);
    });

    // Log the request
    await logApiRequest(request, env, response.status);

    return response;
  } catch (error) {
    console.error('Worker error:', error);
    await logSecurityEvent(env, null, null, 'worker_error', 'medium', error.message, getClientIP(request));
    
    const errorResponse = new Response(JSON.stringify({ error: 'Internal server error' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json', ...corsHeaders }
    });
    
    return errorResponse;
  }
}

// AUTHENTICATION VERIFICATION
async function verifyAuth(request, env) {
  try {
    const authHeader = request.headers.get('Authorization');
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      return { valid: false, error: 'Missing or invalid Authorization header' };
    }
    
    const token = authHeader.replace('Bearer ', '');
    
    // Validate license key against database (don't filter by status here)
    const user = await env.DB.prepare(
      'SELECT license_key, id, account_status FROM users WHERE license_key = ?'
    ).bind(token).first();
    
    // Check if account exists and is not suspended/deleted
    if (!user || user.account_status === 'suspended' || user.account_status === 'deleted') {
      return { valid: false, error: 'Invalid or inactive license' };
    }
    
    return { 
      valid: true, 
      license_key: user.license_key, 
      user_id: user.id 
    };
  } catch (error) {
    console.error('Auth verification error:', error);
    return { valid: false, error: 'Authentication failed' };
  }
}

// INSTALLATION REGISTRATION
async function handleInstallRegistration(request, env) {
  if (request.method !== 'POST') {
    return new Response('Method not allowed', { status: 405 });
  }

  const { license_key, device_fingerprint, machine_id } = await request.json();
  
  if (!license_key || !device_fingerprint || !machine_id) {
    return new Response(JSON.stringify({ error: 'Missing required fields' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' }
    });
  }

  // Verify license key exists and is active
  const user = await getUserByLicense(env.DB, license_key);
  if (!user || user.account_status !== 'active') {
    await logSecurityEvent(env, license_key, null, 'invalid_license', 'high', 'Installation attempted with invalid license', getClientIP(request));
    return new Response(JSON.stringify({ error: 'Invalid license key' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' }
    });
  }

  // Check installation limits
  const existingInstallations = await getActiveInstallations(env.DB, license_key);
  if (existingInstallations.length >= user.max_installations) {
    await logSecurityEvent(env, license_key, null, 'max_installations_exceeded', 'medium', `${existingInstallations.length} installations`, getClientIP(request));
    return new Response(JSON.stringify({ 
      error: 'Maximum installations reached',
      max_allowed: user.max_installations,
      current_count: existingInstallations.length
    }), {
      status: 403,
      headers: { 'Content-Type': 'application/json' }
    });
  }

  // Check for existing installation with same fingerprint
  const existingInstall = await getInstallationByFingerprint(env.DB, license_key, device_fingerprint);
  let installationId;
  
  if (existingInstall) {
    installationId = existingInstall.id;
    // Update last seen
    await updateInstallationLastSeen(env.DB, installationId, getClientIP(request));
  } else {
    // Create new installation
    installationId = generateSecureId();
    await createInstallation(env.DB, {
      id: installationId,
      license_key,
      device_fingerprint,
      machine_id,
      last_ip: getClientIP(request),
      platform: getPlatformFromUserAgent(request.headers.get('User-Agent'))
    });
  }

  // Generate auth token
  const authToken = await generateJWT(env, { license_key, installation_id: installationId });

  return new Response(JSON.stringify({
    installation_id: installationId,
    auth_token: authToken,
    success: true
  }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' }
  });
}

// PRE-CONVERSATION AUTHORIZATION
async function handlePreConversation(request, env) {
  if (request.method !== 'POST') {
    return new Response('Method not allowed', { status: 405 });
  }

  // Verify authorization
  const authResult = await verifyAuth(request, env);
  if (!authResult.valid) {
    return new Response(JSON.stringify({ error: authResult.error }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' }
    });
  }

  const requestBody = await request.json();
  
  // Use license_key from auth result (from Authorization header)
  const license_key = authResult.license_key;
  
  // Provide defaults for optional fields
  const installation_id = requestBody.installation_id || 'default-installation';
  
  // Generate a hash from the question if not provided
  const conversation_request_hash = requestBody.conversation_request_hash || 
    (requestBody.question ? await createHash('sha256', requestBody.question) : generateSecureToken());

  // Rate limiting check
  const rateLimitOk = await checkRateLimit(env.DB, license_key, 'conversations');
  if (!rateLimitOk) {
    await logSecurityEvent(env, license_key, installation_id, 'rate_limit_exceeded', 'medium', 'Conversation rate limit', getClientIP(request));
    return new Response(JSON.stringify({ error: 'Rate limit exceeded' }), {
      status: 429,
      headers: { 'Content-Type': 'application/json' }
    });
  }

  // Check usage limits
  const usageCheck = await checkUsageAllowed(env.DB, license_key);
  
  // Get user info for response regardless of usage status
  const user = await getUserByLicense(env.DB, license_key);
  
  if (!usageCheck.allowed) {
    return new Response(JSON.stringify({
      allowed: false,
      error: usageCheck.error,
      plan: usageCheck.plan,
      used_conversations: usageCheck.used,
      plan_limit: usageCheck.limit,
      type: usageCheck.type,
      // Include user information even when limit reached
      user: {
        id: user.id,
        email: user.email,
        subscription_tier: user.subscription_plan_id || 'free'
      },
      user_id: user.id,
      email: user.email,
      remaining: 0,
      remaining_conversations: 0,
      limits: { daily: usageCheck.limit }
    }), {
      status: 403,
      headers: { 'Content-Type': 'application/json' }
    });
  }

  // Generate conversation token
  const conversationToken = generateSecureToken();
  const expiresAt = new Date(Date.now() + 30 * 60 * 1000); // 30 minutes

  // Store the token
  await storeUsageToken(env.DB, {
    id: generateSecureId(),
    license_key,
    installation_id,
    token: conversationToken,
    expires_at: expiresAt.toISOString(),
    question_hash: conversation_request_hash
  });

  return new Response(JSON.stringify({
    allowed: true,
    conversation_token: conversationToken,
    remaining_conversations: usageCheck.remaining,
    plan_limit: usageCheck.limit,
    expires_at: expiresAt.toISOString(),
    // Include user information in success response
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
}

// POST-CONVERSATION VERIFICATION
async function handlePostConversation(request, env) {
  if (request.method !== 'POST') {
    return new Response('Method not allowed', { status: 405 });
  }

  const { conversation_token, conversation_id, usage_proof } = await request.json();

  if (!conversation_token || !conversation_id || !usage_proof) {
    return new Response(JSON.stringify({ error: 'Missing required fields' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' }
    });
  }

  // Verify the token exists and hasn't been used
  const tokenData = await getUsageToken(env.DB, conversation_token);
  if (!tokenData) {
    await logSecurityEvent(env, null, null, 'invalid_conversation_token', 'high', 'Invalid token used', getClientIP(request));
    return new Response(JSON.stringify({ error: 'Invalid conversation token' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' }
    });
  }

  if (tokenData.used) {
    await logSecurityEvent(env, tokenData.license_key, tokenData.installation_id, 'token_reuse', 'high', 'Token reuse attempt', getClientIP(request));
    return new Response(JSON.stringify({ error: 'Token already used' }), {
      status: 403,
      headers: { 'Content-Type': 'application/json' }
    });
  }

  if (new Date() > new Date(tokenData.expires_at)) {
    return new Response(JSON.stringify({ error: 'Token expired' }), {
      status: 403,
      headers: { 'Content-Type': 'application/json' }
    });
  }

  // Verify usage proof HMAC
  const expectedProof = await createHmac('sha256', conversation_token, `${conversation_id}:${tokenData.question_hash}`);

  if (usage_proof !== expectedProof) {
    await logSecurityEvent(env, tokenData.license_key, tokenData.installation_id, 'invalid_usage_proof', 'high', 'Invalid HMAC proof', getClientIP(request));
    return new Response(JSON.stringify({ error: 'Invalid usage proof' }), {
      status: 403,
      headers: { 'Content-Type': 'application/json' }
    });
  }

  // Mark token as used and record conversation
  await markTokenUsed(env.DB, conversation_token, conversation_id);
  await recordConversationUsage(env.DB, {
    id: generateSecureId(),
    license_key: tokenData.license_key,
    installation_id: tokenData.installation_id,
    conversation_id,
    conversation_token,
    verified: true,
    usage_proof,
    question_hash: tokenData.question_hash
  });

  // Update usage counters
  await incrementUsageCounter(env.DB, tokenData.license_key, env);

  // Get updated usage
  const usageCheck = await checkUsageAllowed(env.DB, tokenData.license_key);

  return new Response(JSON.stringify({
    verified: true,
    remaining: usageCheck.remaining,
    success: true
  }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' }
  });
}

// UTILITY FUNCTIONS

async function getUserByLicense(db, licenseKey) {
  const result = await db.prepare('SELECT * FROM users WHERE license_key = ?').bind(licenseKey).first();
  return result;
}

async function getActiveInstallations(db, licenseKey) {
  const result = await db.prepare('SELECT * FROM installations WHERE license_key = ? AND is_active = 1').bind(licenseKey).all();
  return result.results || [];
}

async function getInstallationByFingerprint(db, licenseKey, deviceFingerprint) {
  const result = await db.prepare(
    'SELECT * FROM installations WHERE license_key = ? AND device_fingerprint = ? AND is_active = 1'
  ).bind(licenseKey, deviceFingerprint).first();
  return result;
}

async function updateInstallationLastSeen(db, installationId, ipAddress) {
  await db.prepare(
    'UPDATE installations SET last_seen = ?, last_ip = ? WHERE id = ?'
  ).bind(new Date().toISOString(), ipAddress, installationId).run();
}

async function createInstallation(db, installationData) {
  await db.prepare(`
    INSERT INTO installations (id, license_key, device_fingerprint, machine_id, last_ip, platform, is_active, installation_date, last_seen)
    VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?)
  `).bind(
    installationData.id,
    installationData.license_key,
    installationData.device_fingerprint,
    installationData.machine_id,
    installationData.last_ip,
    installationData.platform,
    new Date().toISOString(),
    new Date().toISOString()
  ).run();
}

async function generateJWT(env, payload) {
  const jwtPayload = {
    ...payload,
    iss: 'hive-secure-worker',
    aud: 'hive-cli',
    exp: Math.floor(Date.now() / 1000) + (24 * 60 * 60), // 24 hours
    iat: Math.floor(Date.now() / 1000)
  };
  
  return await jwt.sign(jwtPayload, env.JWT_SECRET || env.JWT_SERVICE_SECRET);
}

async function getUsageToken(db, conversationToken) {
  const result = await db.prepare(
    'SELECT * FROM usage_tokens WHERE token = ?'
  ).bind(conversationToken).first();
  return result;
}

async function storeUsageToken(db, tokenData) {
  await db.prepare(`
    INSERT INTO usage_tokens (id, license_key, installation_id, token, expires_at, question_hash, used, created_at)
    VALUES (?, ?, ?, ?, ?, ?, 0, ?)
  `).bind(
    tokenData.id,
    tokenData.license_key,
    tokenData.installation_id,
    tokenData.token,
    tokenData.expires_at,
    tokenData.question_hash,
    new Date().toISOString()
  ).run();
}

async function markTokenUsed(db, conversationToken, conversationId) {
  await db.prepare(
    'UPDATE usage_tokens SET used = 1, conversation_id = ?, used_at = ? WHERE token = ?'
  ).bind(conversationId, new Date().toISOString(), conversationToken).run();
}

async function recordConversationUsage(db, usageData) {
  await db.prepare(`
    INSERT INTO conversation_usage (id, license_key, installation_id, conversation_id, conversation_token, verified, usage_proof, question_hash, timestamp)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
  `).bind(
    usageData.id,
    usageData.license_key,
    usageData.installation_id,
    usageData.conversation_id,
    usageData.conversation_token,
    usageData.verified ? 1 : 0,
    usageData.usage_proof,
    usageData.question_hash,
    new Date().toISOString()
  ).run();
}

async function checkUsageAllowed(db, licenseKey) {
  const user = await getUserByLicense(db, licenseKey);
  if (!user) return { allowed: false, error: 'User not found' };

  // FIRST: Check if user is in 7-day trial period (unlimited conversations)
  if (user.trial_active && user.trial_days_remaining > 0) {
    // Check if trial has actually expired by date
    const now = new Date();
    const trialEnd = new Date(user.trial_end_date);
    
    if (now < trialEnd) {
      return {
        allowed: true,
        type: 'trial',
        plan: user.subscription_plan_id || 'free',
        used: 0,
        limit: 'unlimited',
        remaining: 'unlimited',
        trial_days_remaining: user.trial_days_remaining,
        trial_end_date: user.trial_end_date
      };
    } else {
      // Trial has expired, should update trial_active to 0
      await db.prepare(`
        UPDATE users SET trial_active = 0, trial_days_remaining = 0 
        WHERE license_key = ?
      `).bind(licenseKey).run();
    }
  }

  // SECOND: Check subscription limits after trial period
  if (user.subscription_plan_id && user.subscription_plan_id !== 'credit-pack') {
    return await checkSubscriptionUsage(db, user);
  } else {
    return await checkCreditPackUsage(db, user);
  }
}

async function checkSubscriptionUsage(db, user) {
  const currentDay = new Date().toISOString().substring(0, 10); // YYYY-MM-DD
  const usagePeriod = await db.prepare(
    'SELECT * FROM usage_periods WHERE license_key = ? AND period_type = ? AND period_key = ?'
  ).bind(user.license_key, 'daily', currentDay).first();

  // Correct daily limits based on actual pricing page
  const planLimits = {
    'free': 10,
    'basic': 50,
    'standard': 100,
    'premium': 200,
    'unlimited': 999999,
    'team': 999999
  };

  const limit = user.daily_limit || planLimits[user.subscription_plan_id] || 10;
  const used = usagePeriod ? usagePeriod.conversations_used : 0;
  const remaining = Math.max(0, limit - used);
  
  // Check if user has credits to use after daily limit
  const hasCredits = user.credits_balance > 0;
  const allowedViaCredits = remaining <= 0 && hasCredits;

  return {
    allowed: remaining > 0 || allowedViaCredits,
    type: 'subscription',
    plan: user.subscription_plan_id,
    used,
    limit,
    remaining,
    credits_available: user.credits_balance || 0,
    will_use_credits: allowedViaCredits,
    error: remaining <= 0 && !hasCredits ? 'Daily conversation limit reached and no credits available' : null
  };
}

async function checkCreditPackUsage(db, user) {
  // This function should never be called since we don't have 'credit-pack' as a subscription plan
  // Credits are now part of regular subscriptions, but keeping for backward compatibility
  const hasCredits = user.credits_balance > 0;
  
  return {
    allowed: hasCredits,
    type: 'credits',
    plan: user.subscription_plan_id || 'free',
    used: 0,
    limit: user.credits_balance || 0,
    remaining: user.credits_balance || 0,
    error: !hasCredits ? 'No credits remaining' : null
  };
}

async function incrementUsageCounter(db, licenseKey, env) {
  const user = await getUserByLicense(db, licenseKey);
  
  // Skip usage counting during trial period (unlimited conversations)
  if (user.trial_active && user.trial_days_remaining > 0) {
    const now = new Date();
    const trialEnd = new Date(user.trial_end_date);
    if (now < trialEnd) {
      // Still in trial, no usage tracking needed
      return;
    }
  }
  
  // Check if we need to use credits or count against daily limit
  const currentDay = new Date().toISOString().substring(0, 10); // YYYY-MM-DD
  const usagePeriod = await db.prepare(
    'SELECT * FROM usage_periods WHERE license_key = ? AND period_type = ? AND period_key = ?'
  ).bind(licenseKey, 'daily', currentDay).first();
  
  const limit = user.daily_limit || await getSubscriptionLimit(user.subscription_plan_id, env);
  const used = usagePeriod ? usagePeriod.conversations_used : 0;
  
  if (used >= limit && user.credits_balance > 0) {
    // Consume a credit instead of incrementing daily usage
    await db.prepare(`
      UPDATE users SET credits_balance = credits_balance - 1 
      WHERE license_key = ?
    `).bind(licenseKey).run();
  } else {
    // Increment daily usage counter
    await db.prepare(`
      INSERT INTO usage_periods (id, license_key, period_type, period_key, conversations_used, conversations_limit)
      VALUES (?, ?, 'daily', ?, 1, ?)
      ON CONFLICT(license_key, period_type, period_key)
      DO UPDATE SET conversations_used = conversations_used + 1, updated_at = CURRENT_TIMESTAMP
    `).bind(generateSecureId(), licenseKey, currentDay, limit).run();
  }
}

async function getSubscriptionLimit(plan, env) {
  try {
    // Query the subscription_plans table for the actual limit
    const result = await env.DB.prepare(
      'SELECT daily_limit FROM subscription_plans WHERE id = ? AND active = 1'
    ).bind(plan).first();
    
    if (result) {
      return result.daily_limit;
    }
  } catch (error) {
    console.error('Error fetching subscription limit from D1:', error);
  }
  
  // Fallback limits with correct values
  const limits = { 
    'free': 10,  // Correct: 10, not 5
    'basic': 50, 
    'standard': 100, 
    'premium': 200, 
    'unlimited': 999999, 
    'team': 500,
    'team-unlimited': 999999 
  };
  return limits[plan] || 10;
}

async function checkRateLimit(db, identifier, limitType) {
  const windowDuration = 3600; // 1 hour
  const maxRequests = 100; // 100 requests per hour
  
  const now = new Date();
  const windowStart = new Date(now.getTime() - windowDuration * 1000);
  
  // Get current count
  const current = await db.prepare(
    'SELECT count FROM rate_limits WHERE identifier = ? AND limit_type = ? AND window_start > ?'
  ).bind(identifier, limitType, windowStart.toISOString()).first();
  
  if (current && current.count >= maxRequests) {
    return false;
  }
  
  // Update count
  await db.prepare(`
    INSERT INTO rate_limits (id, identifier, limit_type, count, window_start)
    VALUES (?, ?, ?, 1, ?)
    ON CONFLICT(identifier, limit_type)
    DO UPDATE SET count = count + 1
  `).bind(generateSecureId(), identifier, limitType, now.toISOString()).run();
  
  return true;
}

async function logSecurityEvent(env, licenseKey, installationId, eventType, severity, description, ipAddress) {
  if (!env.DB) return;
  
  try {
    await env.DB.prepare(`
      INSERT INTO security_events (id, license_key, installation_id, event_type, severity, description, ip_address)
      VALUES (?, ?, ?, ?, ?, ?, ?)
    `).bind(generateSecureId(), licenseKey, installationId, eventType, severity, description, ipAddress).run();
  } catch (error) {
    console.error('Failed to log security event:', error);
  }
}

async function logApiRequest(request, env, status) {
  if (!env.DB) return;
  
  try {
    const url = new URL(request.url);
    await env.DB.prepare(`
      INSERT INTO api_audit_log (id, endpoint, method, ip_address, user_agent, response_status)
      VALUES (?, ?, ?, ?, ?, ?)
    `).bind(
      generateSecureId(),
      url.pathname,
      request.method,
      getClientIP(request),
      request.headers.get('User-Agent') || '',
      status
    ).run();
  } catch (error) {
    console.error('Failed to log API request:', error);
  }
}

function generateSecureId() {
  return crypto.randomUUID();
}

function generateSecureToken() {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  return Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');
}

function getClientIP(request) {
  return request.headers.get('CF-Connecting-IP') || 
         request.headers.get('X-Forwarded-For') || 
         'unknown';
}

function getPlatformFromUserAgent(userAgent) {
  if (!userAgent) return 'unknown';
  if (userAgent.includes('Windows')) return 'windows';
  if (userAgent.includes('Mac')) return 'macos';
  if (userAgent.includes('Linux')) return 'linux';
  return 'unknown';
}

// Utility functions for 2025 security model
function getFeaturesByTier(tier) {
  const features = {
    free: ['setup_wizard', 'configure_provider', 'test_providers', 'help', 'browse_models', 'single_model_query', 'basic_analytics'],
    premium: ['consensus_query', 'multi_model_query', 'dashboard', 'analytics', 'export_data', 'cost_tracking', 'benchmarking', 'advanced_features']
  };
  
  if (tier === 'premium') {
    return [...features.free, ...features.premium];
  }
  return features.free;
}

// Get tier limits from D1 database dynamically
async function getTierLimits(tier, env) {
  try {
    // Query the subscription_plans table for the actual limits
    const result = await env.DB.prepare(
      'SELECT daily_limit FROM subscription_plans WHERE id = ? AND active = 1'
    ).bind(tier).first();
    
    if (result) {
      return { 
        daily: result.daily_limit,
        // Note: monthly limits are deprecated, only using daily limits now
        monthly: result.daily_limit * 30 // Approximate monthly for legacy compatibility
      };
    }
  } catch (error) {
    console.error('Error fetching tier limits from D1:', error);
  }
  
  // Fallback to defaults if DB query fails (but with correct free tier limit)
  const fallbackLimits = {
    free: { monthly: 300, daily: 10 },  // Fixed: was 5, now 10
    basic: { monthly: 1500, daily: 50 },
    standard: { monthly: 3000, daily: 100 },
    premium: { monthly: 6000, daily: 200 },
    team: { monthly: 15000, daily: 500 },
    unlimited: { monthly: 999999, daily: 999999 },
    'team-unlimited': { monthly: 999999, daily: 999999 }
  };
  return fallbackLimits[tier] || fallbackLimits.free;
}

// Implement remaining handler functions (existing endpoints)
async function handleSessionValidation(request, env) {
  try {
    // Parse request body (2025 format)
    const data = await request.json();
    
    // Validate 2025 security request format
    if (!data.client_id || !data.session_token || !data.fingerprint || !data.nonce) {
      return new Response(JSON.stringify({
        error: 'Invalid session parameters'
      }), {
        status: 400,
        headers: { 'Content-Type': 'application/json' }
      });
    }

    // Validate client_id
    if (data.client_id !== 'hive-tools') {
      return new Response(JSON.stringify({
        error: 'Unauthorized client'
      }), {
        status: 401,
        headers: { 'Content-Type': 'application/json' }
      });
    }

    // Look up user by session_token (license key)
    const user = await env.DB.prepare(
      'SELECT * FROM users WHERE license_key = ?'
    ).bind(data.session_token).first();
    
    if (!user || user.account_status === 'suspended' || user.account_status === 'deleted') {
      return new Response(JSON.stringify({
        error: 'Invalid session token'
      }), {
        status: 401,
        headers: { 'Content-Type': 'application/json' }
      });
    }

    // Log device fingerprint for security monitoring
    try {
      await env.DB.prepare(
        'INSERT OR REPLACE INTO installations_secure (id, license_key, device_fingerprint, machine_id, last_seen, platform) VALUES (?, ?, ?, ?, ?, ?)'
      ).bind(
        crypto.randomUUID(),
        user.license_key,
        data.fingerprint,
        data.fingerprint, // using fingerprint as machine_id for now
        new Date().toISOString(),
        'unknown'
      ).run();
    } catch (error) {
      console.error('Error logging device fingerprint:', error);
    }

    // Get usage information for the user
    const usageCheck = await checkUsageAllowed(env.DB, user.license_key);
    
    // Always return full user profile with current usage information
    return new Response(JSON.stringify({
      valid: true,
      user_id: user.id,
      email: user.email,
      tier: user.subscription_plan_id || user.subscription_tier || 'free',
      // Return actual limits from usageCheck, not hardcoded
      limits: {
        daily: usageCheck.limit === 'unlimited' ? 999999 : (usageCheck.limit || 10)
      },
      // Include current usage status
      usage: {
        remaining: usageCheck.remaining === 'unlimited' ? 999999 : (usageCheck.remaining || 0),
        limit: usageCheck.limit === 'unlimited' ? 999999 : (usageCheck.limit || 10)
      },
      features: getFeaturesByTier(user.subscription_tier || 'free'),
      status: 'active',
      daily_limit: usageCheck.limit === 'unlimited' ? 999999 : (usageCheck.limit || 10),
      // Legacy session_info for compatibility
      session_info: {
        user_id: user.id,
        subscription_status: user.subscription_tier || 'free',
        conversations_remaining: usageCheck.remaining === 'unlimited' ? 999999 : (usageCheck.remaining || 0),
        daily_limit: (await getTierLimits(user.subscription_tier || 'free', env)).daily,
        max_devices: user.max_devices || 2,
        active_devices: 1
      }
    }), {
      status: 200,
      headers: { 'Content-Type': 'application/json' }
    });

  } catch (error) {
    console.error('Session validation error:', error);
    return new Response(JSON.stringify({
      error: 'Internal server error'
    }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' }
    });
  }
}

// PADDLE WEBHOOK HANDLER
async function handlePaddleWebhook(request, env) {
  if (request.method !== 'POST') {
    return new Response('Method not allowed', { status: 405 });
  }

  try {
    const data = await request.json();
    
    // Verify Paddle webhook signature (you'll need to implement signature verification)
    // const isValid = await verifyPaddleSignature(request, data, env.PADDLE_WEBHOOK_SECRET);
    // if (!isValid) {
    //   return new Response('Invalid signature', { status: 401 });
    // }

    // Handle different Paddle events
    switch (data.event_type) {
      case 'subscription_created':
      case 'transaction_completed':
        await handlePaddlePaymentSuccess(data, env);
        break;
      case 'subscription_cancelled':
        await handlePaddlePaymentCancelled(data, env);
        break;
      case 'subscription_updated':
        await handlePaddlePaymentUpdated(data, env);
        break;
      default:
        console.log(`Unhandled Paddle event: ${data.event_type}`);
    }

    return new Response(JSON.stringify({ success: true }), {
      status: 200,
      headers: { 'Content-Type': 'application/json' }
    });
  } catch (error) {
    console.error('Paddle webhook error:', error);
    await logSecurityEvent(env, null, null, 'paddle_webhook_error', 'medium', error.message, getClientIP(request));
    
    return new Response(JSON.stringify({ error: 'Webhook processing failed' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' }
    });
  }
}

async function handlePaddlePaymentSuccess(data, env) {
  try {
    // Generate license key
    const licenseKey = generateLicenseKey();
    
    // Extract subscription/transaction data
    const email = data.data?.customer?.email || data.data?.billing_details?.email;
    const transactionId = data.data?.id;
    const productId = data.data?.items?.[0]?.product?.id;
    
    // Map Paddle product ID to subscription tier
    const tier = mapPaddleProductToTier(productId);
    
    // Create user in D1 database
    await env.DB.prepare(`
      INSERT INTO users (id, email, license_key, subscription_tier, account_status, 
                        paddle_customer_id, paddle_subscription_id, created_at)
      VALUES (?, ?, ?, ?, 'active', ?, ?, ?)
    `).bind(
      generateSecureId(),
      email,
      licenseKey,
      tier,
      data.data?.customer?.id,
      data.data?.subscription?.id,
      new Date().toISOString()
    ).run();

    // Log the successful provisioning
    await logSecurityEvent(env, licenseKey, null, 'user_provisioned', 'low', 
      `User created via Paddle: ${email}`, getClientIP({ headers: new Headers() }));

    console.log(`User provisioned: ${email} with license: ${licenseKey}`);
  } catch (error) {
    console.error('Error provisioning user:', error);
    throw error;
  }
}

async function handlePaddlePaymentCancelled(data, env) {
  const customerId = data.data?.customer?.id;
  if (customerId) {
    await env.DB.prepare(
      'UPDATE users SET account_status = ? WHERE paddle_customer_id = ?'
    ).bind('cancelled', customerId).run();
  }
}

async function handlePaddlePaymentUpdated(data, env) {
  const customerId = data.data?.customer?.id;
  const productId = data.data?.items?.[0]?.product?.id;
  
  if (customerId && productId) {
    const tier = mapPaddleProductToTier(productId);
    await env.DB.prepare(
      'UPDATE users SET subscription_tier = ? WHERE paddle_customer_id = ?'
    ).bind(tier, customerId).run();
  }
}

function mapPaddleProductToTier(productId) {
  // You'll need to map your actual Paddle product IDs to tiers
  const productTierMap = {
    'pro_basic': 'basic',
    'pro_standard': 'standard', 
    'pro_premium': 'premium',
    'pro_unlimited': 'unlimited',
    'pro_team': 'team'
  };
  return productTierMap[productId] || 'basic';
}

function generateLicenseKey() {
  const segments = [];
  for (let i = 0; i < 4; i++) {
    const segment = Math.random().toString(36).substr(2, 4).toUpperCase();
    segments.push(segment);
  }
  return `HIVE-${segments.join('-')}`;
}

// LICENSE RETRIEVAL HANDLER
async function handleLicenseRetrieve(request, env) {
  if (request.method !== 'POST') {
    return new Response('Method not allowed', { status: 405 });
  }

  try {
    const { transaction_id } = await request.json();
    
    if (!transaction_id) {
      return new Response(JSON.stringify({ error: 'Transaction ID required' }), {
        status: 400,
        headers: { 'Content-Type': 'application/json' }
      });
    }

    // Look up user by Paddle transaction ID
    // Note: You'll need to add a paddle_transaction_id field to your users table
    const user = await env.DB.prepare(
      'SELECT license_key FROM users WHERE paddle_transaction_id = ? OR paddle_subscription_id = ?'
    ).bind(transaction_id, transaction_id).first();

    if (user) {
      return new Response(JSON.stringify({ license_key: user.license_key }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      });
    } else {
      return new Response(JSON.stringify({ error: 'Transaction not found' }), {
        status: 404,
        headers: { 'Content-Type': 'application/json' }
      });
    }
  } catch (error) {
    console.error('License retrieval error:', error);
    return new Response(JSON.stringify({ error: 'Internal server error' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' }
    });
  }
}

// USER CREATION HANDLER
async function handleUserCreate(request, env) {
  if (request.method !== 'POST') {
    return new Response('Method not allowed', { status: 405 });
  }

  // Validate JWT authentication
  const authResult = await validateServiceJWT(request, env);
  if (!authResult.valid) {
    console.log('❌ Unauthorized user creation attempt:', authResult.error);
    return new Response(JSON.stringify({ 
      error: 'Unauthorized',
      message: 'Valid service authentication required' 
    }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' }
    });
  }

  // Check if service has user:create permission
  if (!authResult.decoded.permissions?.includes('user:create')) {
    return new Response(JSON.stringify({ 
      error: 'Insufficient permissions',
      message: 'Service does not have user:create permission' 
    }), {
      status: 403,
      headers: { 'Content-Type': 'application/json' }
    });
  }

  try {
    const user = await request.json();
    
    // Basic validation
    if (!user.id || !user.email || !user.license_key) {
      return new Response(JSON.stringify({ error: 'Missing required fields' }), {
        status: 400,
        headers: { 'Content-Type': 'application/json' }
      });
    }

    // Calculate trial end date (7 days from creation)
    const createdAt = user.created_at || new Date().toISOString();
    const trialEndDate = new Date(new Date(createdAt).getTime() + (7 * 24 * 60 * 60 * 1000)).toISOString();

    // Insert user into D1 database with proper trial initialization
    await env.DB.prepare(`
      INSERT INTO users (
        id, email, name, license_key, subscription_plan_id, 
        daily_limit, account_status, created_at, max_devices,
        trial_active, trial_days_remaining, trial_end_date,
        credits_balance, paddle_customer_id, paddle_subscription_id
      )
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `).bind(
      user.id,
      user.email,
      user.name || '',
      user.license_key,
      user.subscription_plan_id || 'free',
      user.daily_limit || 10, // Free tier gets 10 daily conversations
      user.account_status || 'active',
      createdAt,
      user.max_devices || 3,
      1, // trial_active = 1 (active trial)
      7, // trial_days_remaining = 7 (full trial period)
      trialEndDate, // trial_end_date = created_at + 7 days
      user.credits_balance || 0, // credits_balance = 0 (no initial credits)
      user.paddle_customer_id || null,
      user.paddle_subscription_id || null
    ).run();

    // Log the successful user creation
    await logSecurityEvent(env, user.license_key, null, 'user_created', 'low', 
      `User created via website: ${user.email}`, getClientIP(request));

    console.log(`User created in D1: ${user.email} with license: ${user.license_key}`);

    return new Response(JSON.stringify({ success: true, user_id: user.id }), {
      status: 200,
      headers: { 'Content-Type': 'application/json' }
    });

  } catch (error) {
    console.error('User creation error:', error);
    return new Response(JSON.stringify({ error: 'Failed to create user' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' }
    });
  }
}

// GET USER BY LICENSE HANDLER
async function handleGetUserByLicense(request, env) {
  if (request.method !== 'POST') {
    return new Response('Method not allowed', { status: 405 });
  }

  // Validate JWT authentication
  const authResult = await validateServiceJWT(request, env);
  if (!authResult.valid) {
    console.log('❌ Unauthorized user lookup attempt:', authResult.error);
    return new Response(JSON.stringify({ 
      error: 'Unauthorized',
      message: 'Valid service authentication required' 
    }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' }
    });
  }

  // Check if service has user:read permission
  if (!authResult.decoded.permissions?.includes('user:read')) {
    return new Response(JSON.stringify({ 
      error: 'Insufficient permissions',
      message: 'Service does not have user:read permission' 
    }), {
      status: 403,
      headers: { 'Content-Type': 'application/json' }
    });
  }

  try {
    const { licenseKey } = await request.json();
    
    if (!licenseKey) {
      return new Response(JSON.stringify({ error: 'License key is required' }), {
        status: 400,
        headers: { 'Content-Type': 'application/json' }
      });
    }

    const user = await env.DB.prepare(
      'SELECT * FROM users WHERE license_key = ?'
    ).bind(licenseKey).first();

    console.log(`✅ User lookup: ${licenseKey.substring(0, 8)}... ${user ? 'found' : 'not found'}`);

    return new Response(JSON.stringify({ 
      success: true, 
      user: user || null 
    }), {
      status: 200,
      headers: { 'Content-Type': 'application/json' }
    });

  } catch (error) {
    console.error('❌ User lookup failed:', error);
    return new Response(JSON.stringify({ error: 'Failed to lookup user' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' }
    });
  }
}

async function handleVerifySubscription(request, env) {
  // Keep existing endpoint for backward compatibility
  // Enhanced version of existing function with security checks
  // ... implementation
}

async function handleSyncUsage(request, env) {
  // Enhanced version of existing function
  // ... implementation  
}

async function handleCreateCheckout(request, env) {
  // Enhanced version of existing function
  // ... implementation
}


