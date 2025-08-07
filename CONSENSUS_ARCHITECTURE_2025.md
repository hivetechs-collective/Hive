# Consensus Architecture 2025 - High Performance Design

## 🎯 Goal
Create a blazing-fast, non-blocking consensus system that keeps the UI 100% responsive while maximizing throughput.

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         UI Thread (Dioxus)                   │
│  - Renders UI at 60+ FPS                                     │
│  - Receives progress updates via channels                    │
│  - Sends commands via channels                               │
│  - NEVER blocks, NEVER computes                              │
└────────────┬───────────────────────────┬────────────────────┘
             │ mpsc::channel              │ watch::channel
             ↓                            ↑
┌─────────────────────────────────────────────────────────────┐
│               Consensus Coordinator (Tokio Runtime)          │
│  - Owns the actual ConsensusEngine                           │
│  - Manages all 4 stages                                      │
│  - Handles OpenRouter communication                          │
│  - Runs on dedicated thread pool                             │
└────────────┬───────────────────────────┬────────────────────┘
             │                            │
             ↓                            ↓
┌──────────────────────┐    ┌──────────────────────────────┐
│  AI Helper Workers   │    │   Streaming Processor        │
│  (spawn_blocking)    │    │   (Dedicated Thread)         │
│  - Pattern matching  │    │   - Token processing         │
│  - Context retrieval │    │   - Markdown rendering       │
│  - Quality analysis  │    │   - Batched UI updates      │
└──────────────────────┘    └──────────────────────────────┘
```

## 📐 Detailed Design

### 1. Separate ConsensusEngine from UI State

```rust
// Old (BROKEN): Non-Send, tied to UI
pub struct DesktopConsensusManager {
    engine: Arc<Mutex<ConsensusEngine>>,
    app_state: Signal<AppState>,  // ❌ Non-Send!
}

// New (FAST): Completely separated
pub struct ConsensusCoordinator {
    engine: ConsensusEngine,
    ui_tx: mpsc::Sender<UIMessage>,
    cmd_rx: mpsc::Receiver<Command>,
}

pub struct ConsensusUIBridge {
    ui_rx: mpsc::Receiver<UIMessage>,
    cmd_tx: mpsc::Sender<Command>,
    app_state: Signal<AppState>,
}
```

### 2. Message-Based Communication

```rust
// Commands from UI to Consensus
enum Command {
    StartConsensus { query: String },
    CancelConsensus,
    UpdateProfile { profile: ConsensusProfile },
}

// Updates from Consensus to UI  
enum UIMessage {
    StageStarted { stage: Stage, model: String },
    TokenReceived { token: String },
    StageCompleted { stage: Stage, cost: f64 },
    ConsensusCompleted { result: String },
    Error { message: String },
}
```

### 3. High-Performance Token Streaming

```rust
// Instead of updating UI for every token (slow!)
// Batch tokens and update at 60 FPS max

struct StreamingProcessor {
    token_buffer: String,
    last_update: Instant,
    update_interval: Duration, // 16ms for 60 FPS
}

impl StreamingProcessor {
    async fn process_token(&mut self, token: String) {
        self.token_buffer.push_str(&token);
        
        if self.last_update.elapsed() > self.update_interval {
            // Send batched update to UI
            self.ui_tx.send(UIMessage::TokenBatch(
                self.token_buffer.clone()
            )).await?;
            self.token_buffer.clear();
            self.last_update = Instant::now();
        }
    }
}
```

### 4. Parallel Stage Execution (When Possible)

```rust
// Instead of sequential stages, run what we can in parallel
async fn run_consensus_optimized(&mut self, query: &str) -> Result<String> {
    // Stage 1: Generator (must run first)
    let generator_result = self.run_generator(query).await?;
    
    // Stages 2 & 3: Can run in parallel!
    let (refiner_result, validator_result) = tokio::join!(
        self.run_refiner(&generator_result),
        self.run_validator(&generator_result)
    );
    
    // Stage 4: Curator (needs all previous results)
    let curator_result = self.run_curator(
        &generator_result,
        &refiner_result?,
        &validator_result?
    ).await?;
    
    Ok(curator_result)
}
```

### 5. Zero-Copy Token Processing

```rust
// Use Bytes and BytesMut for zero-copy streaming
use bytes::{Bytes, BytesMut};

struct TokenStream {
    buffer: BytesMut,
}

impl TokenStream {
    fn append(&mut self, data: Bytes) {
        // Zero-copy append
        self.buffer.extend_from_slice(&data);
    }
    
    fn take_complete_tokens(&mut self) -> Vec<String> {
        // Extract complete tokens without copying
        // until necessary
    }
}
```

### 6. CPU Resource Management

```rust
// Use Tokio's cooperative scheduling
async fn cpu_intensive_work() {
    for chunk in large_dataset.chunks(1000) {
        process_chunk(chunk);
        
        // Yield to scheduler every 1000 items
        tokio::task::yield_now().await;
    }
}

// Use dedicated thread pools for different workloads
lazy_static! {
    static ref CPU_POOL: ThreadPool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get() - 2) // Leave 2 cores for UI/system
        .build()
        .unwrap();
}
```

## 🔥 Performance Optimizations

### 1. **Connection Pooling**
```rust
// Reuse HTTP connections to OpenRouter
static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .pool_max_idle_per_host(4)
        .pool_idle_timeout(Duration::from_secs(90))
        .http2_prior_knowledge() // Use HTTP/2 for multiplexing
        .build()
        .unwrap()
});
```

### 2. **Smart Caching**
```rust
// Cache recent consensus results
struct ConsensusCache {
    cache: moka::Cache<String, String>, // High-performance concurrent cache
}

impl ConsensusCache {
    async fn get_or_compute(&self, query: &str) -> Result<String> {
        if let Some(cached) = self.cache.get(query) {
            return Ok(cached);
        }
        
        let result = self.run_consensus(query).await?;
        self.cache.insert(query.to_string(), result.clone());
        Ok(result)
    }
}
```

### 3. **Lazy AI Helpers**
```rust
// Initialize AI helpers only when actually needed
struct LazyAIHelpers {
    helpers: OnceCell<Arc<AIHelperEcosystem>>,
}

impl LazyAIHelpers {
    async fn get(&self) -> Result<&Arc<AIHelperEcosystem>> {
        self.helpers.get_or_try_init(|| async {
            // Initialize only when first needed
            AIHelperEcosystem::new().await
        }).await
    }
}
```

## 📊 Expected Performance Gains

| Metric | Current | Optimized | Improvement |
|--------|---------|-----------|-------------|
| UI Responsiveness | Freezes during consensus | 60+ FPS always | ∞ |
| CPU Usage | 100% single core | 40-60% multi-core | 2-3x efficiency |
| Token Latency | 50-100ms | <16ms | 3-6x faster |
| Memory Usage | 180MB baseline | 80MB baseline | 2.25x reduction |
| Startup Time | 2-3s | <200ms | 10-15x faster |

## 🛠️ Implementation Plan

### Phase 1: Decouple Engine from UI (2-3 days)
1. Extract ConsensusEngine to separate module
2. Create ConsensusCoordinator with Tokio runtime
3. Implement message passing bridge
4. Update UI to use channels instead of direct calls

### Phase 2: Optimize Streaming (1-2 days)
1. Implement batched token updates
2. Add frame rate limiting
3. Use zero-copy buffers
4. Add streaming benchmarks

### Phase 3: Parallelize Operations (1-2 days)
1. Run Refiner and Validator in parallel
2. Add connection pooling
3. Implement caching layer
4. Add lazy initialization

### Phase 4: Fine-tune Performance (1 day)
1. Profile with flamegraph
2. Optimize hot paths
3. Tune thread pool sizes
4. Add performance monitoring

## 🎯 Success Metrics

1. **UI Never Freezes**: Can click, type, scroll during consensus
2. **Fans Don't Spin Up**: CPU stays under 60% total usage
3. **Instant Response**: <16ms latency for any UI interaction
4. **Fast Consensus**: No performance regression in consensus speed
5. **Low Memory**: Under 100MB baseline memory usage

## 🚀 Bonus: WebGPU Acceleration (Future)

For even more performance, we could offload token processing to GPU:

```rust
// Use wgpu for parallel token processing
struct GPUTokenProcessor {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl GPUTokenProcessor {
    async fn process_tokens_batch(&self, tokens: &[String]) -> Vec<ProcessedToken> {
        // Parallel processing on GPU
        // 100x faster for large batches
    }
}
```

---

This architecture represents best practices for 2025:
- **Actor model** for clean separation of concerns
- **Message passing** for loose coupling
- **Zero-copy** operations where possible
- **Parallel execution** of independent operations
- **Lazy initialization** to reduce startup time
- **Connection pooling** for network efficiency
- **Frame rate limiting** for smooth UI
- **Resource governance** through thread pool management

The result: A consensus system that's fast, responsive, and efficient.