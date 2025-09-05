# Universal Memory Infrastructure (UMI)
## The Shared Developer Brain for All AI Tools

### Executive Summary

The Universal Memory Infrastructure transforms Hive's memory system from an internal consensus component into a **memory monopoly** - a centralized intelligence service that enhances every AI development tool. By providing memory-as-a-service to Claude Code CLI, Gemini CLI, OpenAI CLI, Qwen CLI, and other tools, we create a compound intelligence effect where each tool's learnings benefit all others.

## 🎯 Vision

Create a local memory daemon that:
- Serves as the single source of truth for all developer knowledge
- Automatically enriches every AI tool with contextual memory
- Learns from every interaction across all tools
- Creates a network effect where more tools = smarter system

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Developer Machine                         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │          Hive Memory Service (Port 3457)              │  │
│  │  ┌──────────────────────────────────────────────┐    │  │
│  │  │  Memory Core (SQLite + Embeddings)           │    │  │
│  │  │  - Code Patterns                             │    │  │
│  │  │  - Error Solutions                           │    │  │
│  │  │  - Architecture Decisions                    │    │  │
│  │  │  - Team Preferences                          │    │  │
│  │  └──────────────────────────────────────────────┘    │  │
│  │                                                        │  │
│  │  ┌──────────────────────────────────────────────┐    │  │
│  │  │  API Layer                                    │    │  │
│  │  │  - REST API (HTTP)                           │    │  │
│  │  │  - WebSocket (Real-time)                     │    │  │
│  │  │  - Unix Socket (Low latency)                 │    │  │
│  │  └──────────────────────────────────────────────┘    │  │
│  └──────────────────────────────────────────────────────┘  │
│                           ▲                                 │
│                           │                                 │
│        ┌──────────────────┼──────────────────┐             │
│        │                  │                   │             │
│  ┌─────▼─────┐    ┌──────▼──────┐    ┌──────▼──────┐     │
│  │Claude Code│    │  Gemini CLI  │    │ OpenAI CLI  │     │
│  └───────────┘    └──────────────┘    └──────────────┘     │
│                                                              │
│  ┌───────────┐    ┌──────────────┐    ┌──────────────┐     │
│  │ Qwen CLI  │    │    Cursor     │    │  Continue   │     │
│  └───────────┘    └──────────────┘    └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

## 🔌 API Specification

### REST API Endpoints

#### Query Memory
```http
POST http://localhost:3457/api/v1/memory/query
Authorization: Bearer {tool-specific-token}
Content-Type: application/json

{
  "client": "claude-code-cli",
  "context": {
    "file": "src/auth/oauth.js",
    "line": 45,
    "repository": "/Users/dev/project",
    "branch": "feature/oauth"
  },
  "query": "OAuth implementation patterns",
  "options": {
    "limit": 5,
    "thematic": true,
    "include_patterns": true,
    "min_confidence": 0.7
  }
}

Response:
{
  "memories": [
    {
      "id": "mem_abc123",
      "content": "OAuth flow with PKCE implementation",
      "code_snippet": "const challenge = base64url(crypto.randomBytes(32))...",
      "confidence": 0.92,
      "usage_count": 23,
      "last_used": "2024-01-15T10:30:00Z",
      "source_tools": ["claude-code-cli", "cursor"],
      "related_files": ["src/auth/previous.js:234"]
    }
  ],
  "patterns": [
    {
      "name": "async-error-handling",
      "description": "Consistent async/await with try-catch",
      "frequency": 47
    }
  ],
  "suggestions": [
    "Consider implementing refresh token rotation",
    "Add rate limiting to token endpoint"
  ],
  "metadata": {
    "query_time_ms": 12,
    "memories_scanned": 1247,
    "confidence_score": 0.89
  }
}
```

#### Contribute Learning
```http
POST http://localhost:3457/api/v1/memory/contribute
Authorization: Bearer {tool-specific-token}

{
  "source": "gemini-cli",
  "learning": {
    "type": "pattern",
    "category": "error-handling",
    "content": "Graceful WebSocket reconnection",
    "code": "class ReconnectingWebSocket {...}",
    "context": {
      "file": "websocket-manager.js",
      "success": true,
      "performance_impact": "positive"
    },
    "metadata": {
      "language": "javascript",
      "framework": "node.js",
      "tags": ["websocket", "reconnection", "resilience"]
    }
  }
}
```

#### Get Thematic Clusters
```http
GET http://localhost:3457/api/v1/memory/themes?category=authentication

Response:
{
  "themes": [
    {
      "id": "theme_oauth",
      "name": "OAuth Implementations",
      "memory_count": 34,
      "common_patterns": ["PKCE", "refresh-tokens", "JWT"],
      "last_accessed": "2024-01-15T14:00:00Z"
    }
  ]
}
```

### WebSocket Streams

#### Real-time Memory Activity
```javascript
const ws = new WebSocket('ws://localhost:3457/stream');

ws.on('message', (data) => {
  const event = JSON.parse(data);
  // Types: query, contribution, fusion, optimization
  console.log(`[${event.timestamp}] ${event.tool}: ${event.action}`);
});

// Subscribe to specific events
ws.send(JSON.stringify({
  action: 'subscribe',
  events: ['query', 'contribution']
}));
```

### Unix Socket (High Performance)
```bash
# For low-latency local communication
echo '{"action":"query","prompt":"implement auth"}' | \
  nc -U /tmp/hive-memory.sock
```

## 🔧 Integration with CLI Tools

### Auto-Configuration on Install

When Hive installs, it automatically configures each CLI tool:

#### Claude Code CLI
```bash
# ~/.claude/config.json
{
  "memory": {
    "enabled": true,
    "endpoint": "http://localhost:3457",
    "token": "auto-generated-uuid",
    "auto_inject": true
  }
}
```

#### Environment Variables (Universal)
```bash
# Added to shell profile
export HIVE_MEMORY_ENDPOINT="http://localhost:3457"
export HIVE_MEMORY_TOKEN="per-tool-token"
export HIVE_MEMORY_ENABLED=true
```

### Wrapper Scripts

For tools that don't natively support memory injection:

```bash
#!/bin/bash
# /usr/local/bin/hive-wrapped-gemini

# Original command
TOOL="gemini"
ARGS="$@"

# Fetch relevant memory context
CONTEXT=$(curl -s -X POST http://localhost:3457/api/v1/memory/context \
  -H "Authorization: Bearer $HIVE_MEMORY_TOKEN" \
  -d "{
    \"tool\": \"$TOOL\",
    \"prompt\": \"$ARGS\",
    \"working_directory\": \"$(pwd)\"
  }")

# Inject context into prompt
ENRICHED_PROMPT="Context from previous sessions: $CONTEXT\n\nCurrent request: $ARGS"

# Execute with enriched prompt
$TOOL "$ENRICHED_PROMPT"

# Capture and learn from response
RESPONSE=$?
curl -s -X POST http://localhost:3457/api/v1/memory/learn \
  -H "Authorization: Bearer $HIVE_MEMORY_TOKEN" \
  -d "{
    \"tool\": \"$TOOL\",
    \"prompt\": \"$ARGS\",
    \"success\": $RESPONSE
  }"
```

## 📊 Memory Service Dashboard

The Memory button in the Electron app opens a comprehensive dashboard:

### Dashboard Components

1. **Service Status**
   - Active/Inactive indicator
   - Memory daemon health
   - Database size and performance metrics

2. **Connected Tools Monitor**
   ```
   Tool            Status    Queries Today    Contributions    Hit Rate
   Claude Code     ✅        247             12               94.3%
   Gemini CLI      ✅        89              8                91.2%
   OpenAI CLI      ✅        156             23               88.7%
   Qwen CLI        ✅        34              5                92.1%
   Cursor          ✅        412             45               95.8%
   ```

3. **Live Activity Stream**
   - Real-time queries and responses
   - Memory fusion events
   - Pattern recognition alerts
   - Cross-tool learning notifications

4. **Memory Analytics**
   - Most accessed memories
   - Pattern evolution over time
   - Tool collaboration matrix
   - Learning velocity graphs

5. **Management Controls**
   - Export memory database
   - Import team memories
   - Clear specific categories
   - Configure retention policies

## 🚀 Implementation Phases

### Phase 1: Core Memory Service (Week 1-2)
- [ ] Memory daemon with SQLite backend
- [ ] REST API implementation
- [ ] Basic authentication system
- [ ] Query and contribution endpoints

### Phase 2: Tool Integration (Week 3-4)
- [ ] CLI wrapper scripts
- [ ] Native integration patches
- [ ] Auto-configuration system
- [ ] Token management

### Phase 3: Dashboard UI (Week 5-6)
- [ ] Electron dashboard component
- [ ] Real-time WebSocket streams
- [ ] Analytics visualizations
- [ ] Management controls

### Phase 4: Advanced Features (Week 7-8)
- [ ] Memory fusion algorithm
- [ ] Thematic clustering improvements
- [ ] Cross-tool pattern recognition
- [ ] Performance optimizations

## 🔒 Security Considerations

1. **Local-Only by Default**
   - Service binds to localhost only
   - No external network access without explicit configuration

2. **Token-Based Authentication**
   - Unique token per tool
   - Tokens auto-rotate monthly
   - Revocation capability

3. **Memory Isolation**
   - Optional memory namespaces per project
   - Sensitive data filtering
   - Configurable retention policies

4. **Audit Logging**
   - All queries and contributions logged
   - Suspicious pattern detection
   - Regular security reports

## 📈 Success Metrics

1. **Adoption Metrics**
   - Number of connected tools
   - Daily active queries
   - Memory contribution rate

2. **Quality Metrics**
   - Query hit rate (target: >90%)
   - Response time (target: <20ms)
   - Relevance score (target: >0.85)

3. **Value Metrics**
   - Developer time saved
   - Error reduction rate
   - Code quality improvements

## 🔄 Migration from Current System

### Preserving Existing Functionality
1. Current consensus Memory stage continues unchanged
2. Memory Service reads from same SQLite database
3. Gradual migration with feature flags
4. Zero downtime deployment

### Enhancement Path
```
Current: Memory → Generator → Refiner → Validator → Curator
Future:  Memory Service ↘
                         Memory → Generator → Refiner → Validator → Curator
         External Tools ↗
```

## 🎯 Long-term Vision

### Year 1: Local Dominance
- Become the default memory layer for all local AI tools
- 100K+ developers using the system
- 10M+ memories accumulated

### Year 2: Team Collaboration
- Secure memory sharing between team members
- Organizational knowledge bases
- Enterprise memory management

### Year 3: Global Network
- Optional cloud sync
- Cross-organization pattern sharing
- Industry-specific memory packages

## 🤝 Developer Benefits

1. **Consistency**: Same patterns across all tools
2. **Learning**: Every tool gets smarter over time
3. **Efficiency**: No repeated mistakes
4. **Collaboration**: Team knowledge sharing
5. **Portability**: Take your memory anywhere

## 📝 Configuration Reference

### hive-memory.config.toml
```toml
[service]
port = 3457
socket = "/tmp/hive-memory.sock"
database = "~/.hive/memory.db"

[api]
rate_limit = 1000  # requests per minute
max_query_size = 10000  # characters
response_timeout = 5000  # milliseconds

[memory]
max_memories = 1000000
embedding_model = "local-bert"
clustering_algorithm = "dbscan"
fusion_threshold = 0.85

[tools]
auto_configure = true
wrapper_directory = "/usr/local/bin"
token_rotation_days = 30

[security]
local_only = true
audit_log = true
sensitive_filter = true
```

## 🐛 Troubleshooting

### Common Issues

1. **Memory Service Not Starting**
   ```bash
   # Check if port is in use
   lsof -i :3457
   
   # Restart service
   hive-memory restart
   ```

2. **Tool Can't Connect**
   ```bash
   # Verify token
   hive-memory verify-token <tool-name>
   
   # Regenerate if needed
   hive-memory regenerate-token <tool-name>
   ```

3. **High Memory Usage**
   ```bash
   # Compact database
   hive-memory compact
   
   # Clear old memories
   hive-memory clean --older-than 90d
   ```

## 📚 API Client Libraries

### JavaScript/TypeScript
```typescript
import { HiveMemory } from '@hive/memory-client';

const memory = new HiveMemory({
  endpoint: 'http://localhost:3457',
  token: process.env.HIVE_MEMORY_TOKEN
});

const memories = await memory.query({
  prompt: 'implement caching',
  context: { file: 'cache.js' }
});
```

### Python
```python
from hive_memory import MemoryClient

client = MemoryClient()
memories = client.query("implement caching")
```

### Rust
```rust
use hive_memory::Client;

let client = Client::new()?;
let memories = client.query("implement caching").await?;
```

## 🚦 Status Codes

| Code | Meaning |
|------|---------|
| 200 | Success |
| 401 | Invalid token |
| 429 | Rate limited |
| 503 | Memory service unavailable |

## 📖 Related Documentation

- [Memory Architecture](./MEMORY_ARCHITECTURE.md)
- [Consensus Pipeline](./CONSENSUS_PIPELINE.md)
- [Security Model](./SECURITY_MODEL.md)
- [API Reference](./API_REFERENCE.md)

---

*The Universal Memory Infrastructure transforms every AI tool into a learning system, creating compound intelligence that makes developers exponentially more productive.*