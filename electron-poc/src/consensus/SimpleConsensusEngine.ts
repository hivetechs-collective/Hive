import { BrowserWindow } from 'electron';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';
import { MemoryContextDatabase, Memory, ContextFramework, MemoryContextLog } from '../database/MemoryContextDatabase';
import { OptimizedMemoryService } from '../database/OptimizedMemoryService';

interface ConversationMessage {
  speaker: 'generator' | 'refiner' | 'validator';
  content: string;
  round: number;
  consensus_opinion?: 'YES' | 'NO';
}

interface Conversation {
  consensus_id: string;  // Unique ID for this consensus conversation
  user_question: string;
  messages: ConversationMessage[];
  rounds_completed: number;
  consensus_achieved: boolean;
  total_tokens: number;
  total_cost: number;
}

export class SimpleConsensusEngine {
  // Consensus configuration
  private static readonly MAJORITY_THRESHOLD = 2;
  
  private db: any;
  private modelCosts: Map<string, {input: number, output: number}> = new Map();
  private costsLoaded: Promise<void>;
  private conversation: Conversation | null = null;
  private memoryDb: MemoryContextDatabase;
  private optimizedMemory: OptimizedMemoryService;
  private conversationId: string | null = null;
  private userMessageId: string | null = null;
  private consensusType: 'unanimous' | 'majority' | 'curator_override' | 'pending' = 'pending';
  private maxConsensusRounds: number = 3; // Default, will be overridden by profile

  constructor(database: any) {
    this.db = database;
    this.modelCosts = new Map();
    this.costsLoaded = this.loadModelCosts();
    this.memoryDb = new MemoryContextDatabase(database);
    this.optimizedMemory = new OptimizedMemoryService(database);
  }

  private async loadModelCosts(): Promise<void> {
    return new Promise((resolve) => {
      this.db.all(
        'SELECT openrouter_id, pricing_input, pricing_output FROM openrouter_models WHERE is_active = 1',
        [],
        (err: any, rows: any[]) => {
          if (err) {
            console.error('❌ Error loading model costs:', err);
            resolve();
            return;
          }
          
          if (!rows || rows.length === 0) {
            console.warn('⚠️ No model costs found in database');
            resolve();
            return;
          }
          
          console.log(`📊 Got ${rows.length} models from database`);
          
          rows.forEach(row => {
            // Store with the exact openrouter_id as the key
            if (row.openrouter_id && (row.pricing_input !== null || row.pricing_output !== null)) {
              const input = parseFloat(row.pricing_input) || 0;
              const output = parseFloat(row.pricing_output) || 0;
              this.modelCosts.set(row.openrouter_id, {
                input: input,
                output: output
              });
            }
          });
          
          console.log(`📊 Loaded costs for ${this.modelCosts.size} models into Map`);
          
          // Log some specific models we care about
          const testModels = ['anthropic/claude-sonnet-4', 'openai/chatgpt-4o-latest', 'x-ai/grok-4', 'anthropic/claude-opus-4'];
          testModels.forEach(model => {
            const cost = this.modelCosts.get(model);
            if (cost) {
              console.log(`   ✓ ${model}: input=$${cost.input}/M, output=$${cost.output}/M`);
            } else {
              console.log(`   ✗ ${model}: NOT FOUND in Map`);
            }
          });
          
          resolve();
        }
      );
    });
  }

  private calculateCost(model: string, usage: any): number {
    const costs = this.modelCosts.get(model);
    if (!costs) {
      console.log(`⚠️ No cost data for model: ${model}`);
      console.log(`📊 Map size: ${this.modelCosts.size}`);
      console.log(`📊 Sample keys: ${Array.from(this.modelCosts.keys()).slice(0, 5).join(', ')}...`);
      return 0;
    }

    const inputTokens = usage?.prompt_tokens || 0;
    const outputTokens = usage?.completion_tokens || 0;
    
    // Costs are already in dollars per token (e.g., 0.000003 = $3/M tokens)
    // So multiply tokens by cost directly
    const inputCost = inputTokens * costs.input;
    const outputCost = outputTokens * costs.output;
    const totalCost = inputCost + outputCost;
    
    console.log(`💰 Cost for ${model}:`);
    console.log(`   Input: ${inputTokens} tokens × $${costs.input} = $${inputCost.toFixed(6)}`);
    console.log(`   Output: ${outputTokens} tokens × $${costs.output} = $${outputCost.toFixed(6)}`);
    console.log(`   Total: $${totalCost.toFixed(6)}`);
    
    return totalCost;
  }

  async processConsensus(request: any): Promise<any> {
    console.log('🎯 SimpleConsensusEngine.processConsensus called');
    const startTime = Date.now();
    
    // Ensure model costs are loaded before processing
    await this.costsLoaded;
    console.log('💰 Model costs ready, processing consensus...');
    
    try {
      // Get API key from database
      const apiKey = await this.getApiKey();
      if (!apiKey) {
        throw new Error('No OpenRouter API key configured');
      }

      // Get the active profile
      const profile = await this.getProfile(request.profileName);
      if (!profile) {
        throw new Error('No profile found');
      }

      // Set max consensus rounds from profile (defaults to 3 if not set)
      this.maxConsensusRounds = profile.max_consensus_rounds || 3;

      console.log('🎯 Using profile:', profile.profile_name);
      console.log('🎯 Models:', {
        generator: profile.generator_model,
        refiner: profile.refiner_model,
        validator: profile.validator_model,
        curator: profile.curator_model
      });
      console.log('🎯 Max Consensus Rounds:', this.maxConsensusRounds);

      // Initialize conversation for iterative deliberation
      // Generate unique consensus_id (timestamp + random)
      const consensusId = `consensus_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      
      // Generate conversation ID if not provided
      this.conversationId = request.conversationId || `conv_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      
      this.conversation = {
        consensus_id: consensusId,
        user_question: request.query,
        messages: [],
        rounds_completed: 0,
        consensus_achieved: false,
        total_tokens: 0,
        total_cost: 0
      };
      
      // Store the user's question in the database using optimized service
      try {
        console.log('🚀 Storing user message via optimized service');
        this.userMessageId = await this.optimizedMemory.storeMessage({
          conversationId: this.conversationId,
          role: 'user',
          content: request.query
        });
        console.log(`💾 Stored user question with ID: ${this.userMessageId}`);
        
        // Insert into conversation_usage for analytics and memory service tracking
        const userId = '3034c561-e193-4968-a575-f1b165d31a5b'; // sales@hivetechs.io user ID
        await this.recordConversationUsage(userId, this.conversationId);
        console.log(`📊 Recorded conversation usage for analytics`);
      } catch (error) {
        console.error('❌ Failed to store user message:', error);
      }

      // MEMORY STAGE - Retrieve relevant past conversations
      console.log('\n🧠 MEMORY STAGE - Retrieving relevant memories');
      this.sendStageUpdate('memory', 'running');
      this.sendProgressUpdate(request.requestId, 'Searching memory for relevant context...', 0.02);
      
      const relevantMemories = await this.retrieveRelevantMemories(request.query);
      console.log(`📚 Found ${relevantMemories.length} relevant memories`);

      // CONTEXT STAGE - Build contextual framework
      console.log('\n🔍 CONTEXT STAGE - Building contextual framework');
      // Mark memory complete and context running at the same time
      this.sendStageUpdate('memory', 'completed');
      this.sendStageUpdate('context', 'running');
      this.sendProgressUpdate(request.requestId, 'Analyzing context and patterns...', 0.04);
      
      const contextFramework = await this.buildContextFramework(request.query, relevantMemories);
      console.log(`📝 Context framework built with ${contextFramework.patterns.length} patterns identified`);

      // Save context framework to database for review
      const contextLogId = `ctxlog_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      await this.memoryDb.logMemoryContextOperation({
        log_id: contextLogId,
        request_id: request.requestId,
        conversation_id: this.conversationId || undefined,
        memories_retrieved: {
          recent: relevantMemories.filter(m => m.recency_score === 4).length,
          today: relevantMemories.filter(m => m.recency_score === 3).length,
          week: relevantMemories.filter(m => m.recency_score === 2).length,
          semantic: relevantMemories.filter(m => m.recency_score === 1).length
        },
        context_summary: contextFramework.summary,
        patterns_identified: contextFramework.patterns,
        topics_extracted: contextFramework.relevantTopics,
        performance_ms: {
          memory: 0, // Already logged in memory retrieval
          context: 0 // Will be updated separately
        }
      });
      console.log(`💾 Context framework saved to database with ID: ${contextLogId}`);

      // ROUTING STAGE - Determine if question is simple or complex
      console.log('\n🔄 ROUTING STAGE - Determining question complexity');
      // Mark context complete and route running at the same time
      this.sendStageUpdate('context', 'completed');
      this.sendStageUpdate('route', 'running');
      this.sendProgressUpdate(request.requestId, 'Analyzing question complexity...', 0.05);
      
      const routingPrompt = `Analyze this question and determine if it requires simple or complex reasoning.

${contextFramework.summary ? `Context from past conversations:\n${contextFramework.summary}\n` : ''}

A SIMPLE question:
- Has a straightforward, factual answer
- Requires basic explanation or definition
- Can be answered in a few sentences
- Examples: "What is Python?", "How do I create a variable?", "What time is it?"

A COMPLEX question:
- Requires multi-step reasoning or analysis
- Needs code generation or detailed technical explanation
- Requires synthesis of multiple concepts
- Examples: "Create a full application", "Debug this complex code", "Design a system architecture"

Question: ${request.query}

Respond with ONLY one word: SIMPLE or COMPLEX`;

      const routingResult = await this.callOpenRouter(apiKey, profile.generator_model, routingPrompt);
      const routingDecision = routingResult.content.trim().toUpperCase();
      
      console.log(`📊 Routing Decision: ${routingDecision}`);
      
      // Update the context log with routing decision
      await this.updateContextLogRouting(contextLogId, routingDecision);
      
      this.sendStageUpdate('route', 'completed');
      
      // Add routing cost to conversation
      this.conversation.total_tokens += routingResult.usage.total_tokens;
      this.conversation.total_cost += this.calculateCost(profile.generator_model, routingResult.usage);

      // SIMPLE PATH - Single response from Generator
      if (routingDecision === 'SIMPLE' || routingDecision.includes('SIMPLE')) {
        console.log('✨ SIMPLE QUESTION - Using direct response path');
        this.sendProgressUpdate(request.requestId, 'Generating direct response...', 0.2);
        
        // Get single response from Generator with context
        this.sendStageUpdate('generator', 'running');
        
        // Build context-enhanced prompt
        let enhancedPrompt = request.query;
        if (contextFramework.summary) {
          enhancedPrompt = `Context from previous conversations: ${contextFramework.summary}\n\nCurrent question: ${request.query}`;
        }
        
        const simpleResult = await this.callOpenRouter(apiKey, profile.generator_model, enhancedPrompt);
        this.sendStageUpdate('generator', 'completed');
        
        // Update conversation stats
        this.conversation.total_tokens += simpleResult.usage.total_tokens;
        this.conversation.total_cost += this.calculateCost(profile.generator_model, simpleResult.usage);
        this.conversation.messages.push({
          speaker: 'generator',
          content: simpleResult.content,
          round: 1
        });
        this.conversation.rounds_completed = 1;
        this.conversation.consensus_achieved = true;
        
        // Log the simple response
        await this.logConsensusResponse(
          1,
          'generator',
          profile.generator_model,
          request.query,
          simpleResult.content,
          undefined
        );
        
        console.log('\n📊 SIMPLE PATH COMPLETE');
        console.log('  Total tokens:', this.conversation.total_tokens);
        console.log('  Total cost: $' + this.conversation.total_cost.toFixed(4));
        
        // Store the assistant's response in the database using optimized service
        try {
          console.log('🚀 Storing assistant response via optimized service');
          const assistantMessageId = await this.optimizedMemory.storeMessage({
            conversationId: this.conversationId,
            role: 'assistant',
            content: simpleResult.content,
            model: profile.generator_model,
            tokensUsed: this.conversation.total_tokens,
            cost: this.conversation.total_cost,
            consensusPath: 'SIMPLE',
            consensusRounds: 1,
            parentMessageId: this.userMessageId || undefined
          });
          console.log(`💾 Stored assistant response with ID: ${assistantMessageId}`);
        } catch (error) {
          console.error('❌ Failed to store assistant message:', error);
        }
        
        // Send completion
        this.sendConsensusComplete({
          response: simpleResult.content,
          totalTokens: this.conversation.total_tokens,
          totalCost: this.conversation.total_cost,
          conversationId: this.conversationId,
          rounds: 1,
          consensusAchieved: true
        });
        
        // Mark all stages as completed for UI
        this.sendStageUpdate('refiner', 'completed');
        this.sendStageUpdate('validator', 'completed');
        this.sendStageUpdate('curator', 'completed');
        
        return;
      }

      // COMPLEX PATH - Full consensus pipeline
      console.log('🧩 COMPLEX QUESTION - Using full consensus pipeline');
      
      // ITERATIVE DELIBERATION LOOP (max rounds from profile - let consensus happen naturally)
      while (!this.conversation.consensus_achieved && this.conversation.rounds_completed < this.maxConsensusRounds) {
        this.conversation.rounds_completed++;
        console.log(`\n🔄 Starting Round ${this.conversation.rounds_completed}`);
        
        // Reset stages to 'ready' for new round (visual sync)
        if (this.conversation.rounds_completed > 1) {
          console.log('🔄 Resetting stages for new round');
          this.sendStageUpdate('generator', 'ready');
          this.sendStageUpdate('refiner', 'ready');
          this.sendStageUpdate('validator', 'ready');
        }
        
        // Send round update to renderer
        this.sendRoundUpdate(this.conversation.rounds_completed);
        
        // Execute one round of Generator → Refiner → Validator with context
        await this.executeDeliberationRound(apiKey, profile, contextFramework);
        
        // Check consensus after each round
        await this.checkConsensus(apiKey, profile);
        
        console.log(`📊 Round ${this.conversation.rounds_completed} complete. Consensus: ${this.conversation.consensus_achieved ? 'YES' : 'NO'}`);
      }

      // Check why we exited the loop
      console.log('\n📊 LOOP EXIT STATUS:');
      console.log(`  Rounds completed: ${this.conversation.rounds_completed}`);
      console.log(`  Consensus achieved: ${this.conversation.consensus_achieved}`);
      console.log(`  Max rounds (${this.maxConsensusRounds}): ${this.conversation.rounds_completed >= this.maxConsensusRounds ? 'REACHED' : 'Not reached'}`);
      
      if (!this.conversation.consensus_achieved) {
        console.log(`\n⚠️ Maximum rounds (${this.maxConsensusRounds}) reached without consensus!`);
        console.log('📝 Using last validator response as final (no Curator)');
      }

      // CURATOR - Always runs, but with different roles based on consensus type
      let finalResponse: string;
      console.log('\n🎯 Stage 4: Curator');
      
      if (this.consensusType === 'unanimous') {
        // Unanimous consensus - curator just polishes the agreed response
        console.log('✅ UNANIMOUS CONSENSUS - Curator will polish the agreed response');
        const curatorResult = await this.curateConsensusResponse(apiKey, profile, 'polish', contextFramework);
        finalResponse = curatorResult.content;
      } else if (this.consensusType === 'majority') {
        // Majority consensus - curator polishes the majority-agreed response
        console.log('🤝 MAJORITY CONSENSUS - Curator will polish the majority response');
        const curatorResult = await this.curateConsensusResponse(apiKey, profile, 'polish', contextFramework);
        finalResponse = curatorResult.content;
      } else if (this.consensusType === 'curator_override') {
        // No consensus - curator must choose from all 3 responses
        console.log('👨‍⚖️ NO CONSENSUS - Curator will review all 3 responses and choose the best');
        const curatorResult = await this.curateConsensusResponse(apiKey, profile, 'choose', contextFramework);
        finalResponse = curatorResult.content;
      } else {
        // Fallback (shouldn't happen)
        console.log('⚠️ Unexpected consensus type - using last response');
        const lastMessage = this.conversation.messages[this.conversation.messages.length - 1];
        finalResponse = lastMessage.content;
      }
      
      console.log('\n📊 FINAL STATISTICS:');
      console.log('  Total tokens:', this.conversation.total_tokens);
      console.log('  Total cost: $' + this.conversation.total_cost.toFixed(4));
      console.log('  Rounds:', this.conversation.rounds_completed);
      console.log('  Consensus:', this.conversation.consensus_achieved ? 'YES' : 'NO');

      // Store the assistant's response in the database for future memory retrieval
      try {
        const modelUsed = this.conversation.consensus_achieved ? profile.curator_model : profile.validator_model;
        console.log('🚀 Storing consensus response via optimized service');
        const assistantMessageId = await this.optimizedMemory.storeMessage({
          conversationId: this.conversationId,
          role: 'assistant',
          content: finalResponse,
          model: modelUsed,
          tokensUsed: this.conversation.total_tokens,
          cost: this.conversation.total_cost,
          consensusPath: 'COMPLEX',
          consensusRounds: this.conversation.rounds_completed,
          parentMessageId: this.userMessageId || undefined
        });
        console.log(`💾 Stored assistant response with ID: ${assistantMessageId}`);
      } catch (error) {
        console.error('❌ Failed to store assistant message:', error);
      }

      // Send to renderer
      this.sendConsensusComplete({
        response: finalResponse,
        totalTokens: this.conversation.total_tokens,
        totalCost: this.conversation.total_cost,
        conversationId: this.conversationId,
        rounds: this.conversation.rounds_completed,
        consensusAchieved: this.conversation.consensus_achieved
      });

      // Return in expected format
      const stagesCompleted = ['generator', 'refiner', 'validator'];
      if (this.conversation.consensus_achieved) {
        stagesCompleted.push('curator');
      }
      
      return {
        result: finalResponse,
        response: finalResponse,
        success: true,
        mode: 'consensus',
        stages_completed: stagesCompleted,
        rounds_completed: this.conversation.rounds_completed,
        tokens_used: this.conversation.total_tokens,
        cost: this.conversation.total_cost,
        duration_ms: Date.now() - startTime
      };

    } catch (error: any) {
      console.error('🔴 SimpleConsensusEngine error:', error);
      throw error;
    }
  }

  private async getApiKey(): Promise<string | null> {
    return new Promise((resolve) => {
      this.db.get(
        'SELECT value FROM configurations WHERE key = ?',
        ['openrouter_api_key'],
        (err: any, row: any) => {
          if (err || !row) {
            resolve(null);
          } else {
            resolve(row.value);
          }
        }
      );
    });
  }

  private async logConsensusResponse(
    round: number,
    stage: string,
    model: string,
    prompt: string,
    response: string,
    consensusVote?: 'YES' | 'NO'
  ): Promise<void> {
    try {
      // Create log directory if it doesn't exist
      const logDir = path.join(os.homedir(), '.hive', 'consensus-logs');
      if (!fs.existsSync(logDir)) {
        fs.mkdirSync(logDir, { recursive: true });
      }

      // Create log file for this consensus session
      const logFile = path.join(logDir, `consensus_${this.conversation!.consensus_id}.json`);
      
      // Read existing log or create new
      let logData: any = { 
        consensus_id: this.conversation!.consensus_id,
        user_question: this.conversation!.user_question,
        started: new Date().toISOString(),
        rounds: []
      };
      
      if (fs.existsSync(logFile)) {
        const existing = fs.readFileSync(logFile, 'utf-8');
        logData = JSON.parse(existing);
      }

      // Find or create round entry
      let roundData = logData.rounds.find((r: any) => r.round === round);
      if (!roundData) {
        roundData = { round, responses: [] };
        logData.rounds.push(roundData);
      }

      // Add this response
      roundData.responses.push({
        timestamp: new Date().toISOString(),
        stage,
        model,
        prompt: prompt.substring(0, 500) + (prompt.length > 500 ? '...' : ''), // Truncate long prompts
        response: response.substring(0, 2000) + (response.length > 2000 ? '...' : ''), // Truncate long responses
        fullResponse: response, // Keep full response for analysis
        consensusVote
      });

      // Write updated log
      fs.writeFileSync(logFile, JSON.stringify(logData, null, 2));
      console.log(`📝 Logged ${stage} response to: ${logFile}`);

    } catch (error) {
      console.error('Failed to log consensus response:', error);
      // Don't throw - logging failure shouldn't stop consensus
    }
  }

  private async getProfile(profileName: string): Promise<any> {
    return new Promise((resolve) => {
      // Note: consensus_profiles doesn't have is_active, just get by name
      const query = 'SELECT * FROM consensus_profiles WHERE profile_name = ?';
      const params = [profileName || 'Free Also'];
      
      this.db.get(query, params, (err: any, row: any) => {
        if (err || !row) {
          resolve(null);
        } else {
          resolve(row);
        }
      });
    });
  }

  private async retrieveRelevantMemories(query: string): Promise<Memory[]> {
    const startTime = Date.now();
    
    try {
      // Use optimized parallel memory retrieval with connection pool
      console.log('🚀 Using optimized parallel memory retrieval');
      
      // Execute optimized parallel query
      const memories = await this.optimizedMemory.retrieveMemories(query, this.conversationId || undefined);
      
      const endTime = Date.now();
      console.log(`⚡ Optimized memory retrieval took ${endTime - startTime}ms`);
      
      // Get performance metrics
      const metrics = this.optimizedMemory.getPerformanceMetrics();
      console.log(`📊 Performance Stats:`);
      console.log(`  - Cache size: ${metrics.cacheSize}`);
      console.log(`  - Connection pool: ${metrics.connectionPoolSize} connections`);
      
      // Log memory counts by layer for verification
      const recentCount = memories.filter((m: any) => m.recencyScore === 4).length;
      const todayCount = memories.filter((m: any) => m.recencyScore === 3).length;
      const weekCount = memories.filter((m: any) => m.recencyScore === 2).length;
      const semanticCount = memories.filter((m: any) => m.recencyScore === 1).length;
      
      console.log(`📊 Memory Distribution:`);
      console.log(`  - Recent (2h): ${recentCount} memories`);
      console.log(`  - Today (24h): ${todayCount} memories`);
      console.log(`  - This Week: ${weekCount} memories`);
      console.log(`  - Semantic: ${semanticCount} memories`);
      
      // Log the memory retrieval operation
      const logId = `memlog_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      await this.memoryDb.logMemoryContextOperation({
        log_id: logId,
        request_id: this.conversation?.consensus_id || 'unknown',
        conversation_id: this.conversationId || undefined,
        memories_retrieved: {
          recent: recentCount,
          today: todayCount,
          week: weekCount,
          semantic: semanticCount
        },
        performance_ms: {
          memory: endTime - startTime,
          context: 0 // Will be updated in buildContextFramework
        }
      });
      
      return memories;
    } catch (error) {
      console.error('❌ Error in memory retrieval:', error);
      return [];
    }
  }
  
  private async buildContextFramework(query: string, memories: Memory[]): Promise<ContextFramework> {
    const startTime = Date.now();
    
    const framework = {
      summary: '',
      patterns: [] as string[],
      relevantTopics: [] as string[],
      userPreferences: [] as string[]
    };
    
    if (memories.length === 0) {
      console.log('📝 No memories found, using empty context framework');
      return framework;
    }
    
    // Sort memories by timestamp (newest first) for chronological context
    const sortedMemories = memories.sort((a, b) => 
      new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
    );
    
    console.log(`📚 Building context from ${memories.length} memories (sorted by timestamp)`);
    
    // Extract the actual conversation context from recent messages
    const recentContext: string[] = [];
    // Take the most recent 10 messages for context building
    sortedMemories.slice(0, 10).forEach(memory => {
      if (memory.role === 'user') {
        recentContext.push(`User asked: "${memory.content}"`);
      } else if (memory.role === 'assistant') {
        // Extract key topics from assistant responses
        const shortContent = memory.content.substring(0, 150);
        recentContext.push(`Previously discussed: ${shortContent}...`);
      }
    });
    
    // Extract patterns and topics from ALL memories
    const topics = new Set<string>();
    const patterns = new Set<string>();
    
    memories.forEach(memory => {
      // Extract meaningful topics (not just words)
      if (/PowerShell|powershell/i.test(memory.content)) topics.add('PowerShell');
      if (/Python|python/i.test(memory.content)) topics.add('Python');
      if (/JavaScript|javascript|JS/i.test(memory.content)) topics.add('JavaScript');
      if (/TypeScript|typescript|TS/i.test(memory.content)) topics.add('TypeScript');
      if (/React|Vue|Angular/i.test(memory.content)) topics.add('Web Frameworks');
      if (/database|SQL|query/i.test(memory.content)) topics.add('Databases');
      if (/API|REST|GraphQL/i.test(memory.content)) topics.add('APIs');
      if (/example|examples/i.test(memory.content)) topics.add('Examples Requested');
      
      // Identify conversation patterns
      if (memory.content.includes('?')) patterns.add('questions');
      if (/function|const|class|def|import/.test(memory.content)) patterns.add('code-related');
      if (/create|build|implement|develop/.test(memory.content)) patterns.add('creation-tasks');
      if (/fix|debug|solve|error/.test(memory.content)) patterns.add('debugging');
      if (/optimize|improve|enhance/.test(memory.content)) patterns.add('optimization');
      if (/example|show me|how to/i.test(memory.content)) patterns.add('examples-needed');
    });
    
    framework.patterns = Array.from(patterns);
    framework.relevantTopics = Array.from(topics);
    
    // Build comprehensive summary with actual conversation context
    if (recentContext.length > 0) {
      framework.summary = `Current conversation context: ${recentContext.join(' ')} `;
    }
    
    if (framework.relevantTopics.length > 0) {
      framework.summary += `Topics being discussed: ${framework.relevantTopics.join(', ')}. `;
    }
    
    if (framework.patterns.length > 0) {
      framework.summary += `Conversation patterns: ${framework.patterns.join(', ')}.`;
    }
    
    const endTime = Date.now();
    console.log(`⚡ Context building took ${endTime - startTime}ms`);
    console.log('📊 Context Framework:');
    console.log(`  - Total memories used: ${memories.length}`);
    console.log(`  - Recent context items: ${recentContext.length}`);
    console.log(`  - Patterns: ${framework.patterns.join(', ')}`);
    console.log(`  - Topics: ${framework.relevantTopics.join(', ')}`);
    console.log(`  - Summary: ${framework.summary ? framework.summary.substring(0, 200) + '...' : 'No summary'}`);
    
    return framework;
  }

  private async callOpenRouter(apiKey: string, model: string, query: string): Promise<{content: string, usage: any}> {
    console.log('🎯 Calling OpenRouter with model:', model);
    
    // Determine max_tokens based on model capabilities (2025 standards)
    let maxTokens = 8192; // Safe default for most models
    
    // Models with extended output capabilities
    if (model.includes('claude') && model.includes('sonnet')) {
      maxTokens = 16384; // Claude Sonnet supports larger outputs
    } else if (model.includes('gpt-4') || model.includes('o1')) {
      maxTokens = 16384; // GPT-4 and o1 models support larger outputs
    } else if (model.includes('gemini')) {
      maxTokens = 8192; // Gemini models have good output support
    } else if (model.includes('mistral') || model.includes('deepseek')) {
      maxTokens = 8192; // Standard for these models
    }
    
    console.log(`📊 Using max_tokens: ${maxTokens} for model: ${model}`);
    
    const response = await fetch('https://openrouter.ai/api/v1/chat/completions', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${apiKey}`,
        'Content-Type': 'application/json',
        'HTTP-Referer': 'https://hivetech.ai',
        'X-Title': 'Hive Consensus'
      },
      body: JSON.stringify({
        model: model,
        messages: [
          {
            role: 'user',
            content: query
          }
        ],
        temperature: 0.7,
        max_tokens: maxTokens
      })
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error('🔴 OpenRouter API error:', errorText);
      throw new Error(`OpenRouter API error: ${response.status}`);
    }

    const data = await response.json();
    console.log('🎯 OpenRouter response - usage:', data.usage);
    
    if (data.choices && data.choices[0] && data.choices[0].message) {
      return {
        content: data.choices[0].message.content,
        usage: data.usage || { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 }
      };
    }
    
    throw new Error('Invalid response from OpenRouter');
  }

  private async executeDeliberationRound(apiKey: string, profile: any, contextFramework?: any): Promise<void> {
    const round = this.conversation!.rounds_completed;
    
    // GENERATOR
    console.log('🎯 Stage 1: Generator');
    this.sendStageUpdate('generator', 'running');
    
    let generatorPrompt: string;
    if (round === 1) {
      // First round: original question with context if available
      generatorPrompt = this.conversation!.user_question;
      if (contextFramework && contextFramework.summary) {
        generatorPrompt = `Context from previous conversations: ${contextFramework.summary}\n\nCurrent question: ${this.conversation!.user_question}`;
      }
    } else {
      // Subsequent rounds: Generator uses consensus evaluation question
      const lastValidatorMessage = this.conversation!.messages[this.conversation!.messages.length - 1];
      generatorPrompt = `Evaluate this response for accuracy and completeness.
If it correctly answers the original question with no major errors or omissions, respond with ONLY the word: ACCEPT
If it has errors or is missing critical information, provide a corrected version.

Original question: ${this.conversation!.user_question}

Current response:
${lastValidatorMessage.content}`;
    }
    
    const generatorResult = await this.callOpenRouter(apiKey, profile.generator_model, generatorPrompt);
    this.conversation!.messages.push({
      speaker: 'generator',
      content: generatorResult.content,
      round: round
    });
    this.conversation!.total_tokens += generatorResult.usage.total_tokens;
    this.conversation!.total_cost += this.calculateCost(profile.generator_model, generatorResult.usage);
    
    // Log response for analysis
    await this.logConsensusResponse(
      round,
      'generator',
      profile.generator_model,
      generatorPrompt,
      generatorResult.content
    );
    
    // Log iteration to database (flag is 0 since this is not a consensus check)
    this.logIteration(
      this.conversation!.consensus_id,
      profile.generator_model,
      'generator',
      generatorResult.usage.total_tokens,
      0,
      round
    );
    
    // REFINER
    console.log('🎯 Stage 2: Refiner');
    // Mark generator complete and refiner running at the same time
    this.sendStageUpdate('generator', 'completed');
    this.sendStageUpdate('refiner', 'running');
    
    // Refiner evaluates the Generator's response using the consensus question
    const refinerPrompt = `Evaluate this response for accuracy and completeness.
If it correctly answers the original question with no major errors or omissions, respond with ONLY the word: ACCEPT
If it has errors or is missing critical information, provide a corrected version.

Original question: ${this.conversation!.user_question}

Current response:
${generatorResult.content}`;
    
    const refinerResult = await this.callOpenRouter(apiKey, profile.refiner_model, refinerPrompt);
    this.conversation!.messages.push({
      speaker: 'refiner',
      content: refinerResult.content,
      round: round
    });
    this.conversation!.total_tokens += refinerResult.usage.total_tokens;
    this.conversation!.total_cost += this.calculateCost(profile.refiner_model, refinerResult.usage);
    
    // Log response for analysis
    await this.logConsensusResponse(
      round,
      'refiner',
      profile.refiner_model,
      refinerPrompt,
      refinerResult.content
    );
    
    // Log iteration to database
    this.logIteration(
      this.conversation!.consensus_id,
      profile.refiner_model,
      'refiner',
      refinerResult.usage.total_tokens,
      0,
      round
    );
    
    // VALIDATOR
    console.log('🎯 Stage 3: Validator');
    // Mark refiner complete and validator running at the same time
    this.sendStageUpdate('refiner', 'completed');
    this.sendStageUpdate('validator', 'running');
    
    // Validator evaluates for consensus
    const validatorPrompt = `Evaluate this response for accuracy and completeness.
If it correctly answers the original question with no major errors or omissions, respond with ONLY the word: ACCEPT
If it has errors or is missing critical information, provide a corrected version.

Original question: ${this.conversation!.user_question}

Current response:
${refinerResult.content}`;
    
    const validatorResult = await this.callOpenRouter(apiKey, profile.validator_model, validatorPrompt);
    this.conversation!.messages.push({
      speaker: 'validator',
      content: validatorResult.content,
      round: round
    });
    this.conversation!.total_tokens += validatorResult.usage.total_tokens;
    this.conversation!.total_cost += this.calculateCost(profile.validator_model, validatorResult.usage);
    
    // Log response for analysis
    await this.logConsensusResponse(
      round,
      'validator',
      profile.validator_model,
      validatorPrompt,
      validatorResult.content
    );
    
    // Log iteration to database
    this.logIteration(
      this.conversation!.consensus_id,
      profile.validator_model,
      'validator',
      validatorResult.usage.total_tokens,
      0,
      round
    );
    
    this.sendStageUpdate('validator', 'completed');
  }

  private async checkConsensus(apiKey: string, profile: any): Promise<void> {
    // Get the last validator response (the final response of this round)
    const lastMessage = this.conversation!.messages[this.conversation!.messages.length - 1];
    const currentResponse = lastMessage.content;
    
    const consensusPrompt = `Evaluate this response for accuracy and completeness.
If it correctly answers the original question with no major errors or omissions, respond with ONLY the word: ACCEPT
If it has errors or is missing critical information, provide a corrected version.

Original question: ${this.conversation!.user_question}

Current response:
${currentResponse}`;
    
    // Ask each model for their opinion
    const opinions: ('YES' | 'NO')[] = [];
    
    // Generator's opinion
    console.log('🤔 Asking Generator for consensus opinion...');
    const genOpinion = await this.callOpenRouter(apiKey, profile.generator_model, consensusPrompt);
    console.log(`  Generator raw response: "${genOpinion.content}"`);
    const genVote = this.parseConsensusOpinion(genOpinion.content);
    opinions.push(genVote);
    console.log(`  Generator vote: ${genVote}`);
    
    // Log consensus vote for analysis
    await this.logConsensusResponse(
      this.conversation!.rounds_completed,
      'consensus_check_generator',
      profile.generator_model,
      consensusPrompt,
      genOpinion.content,
      genVote
    );
    
    // Log consensus check iteration (flag = 1 if NO, 0 if YES)
    this.logIteration(
      this.conversation!.consensus_id,
      profile.generator_model,
      'consensus_check_generator',
      genOpinion.usage.total_tokens,
      genVote === 'NO' ? 1 : 0,
      this.conversation!.rounds_completed
    );
    
    // Refiner's opinion
    console.log('🤔 Asking Refiner for consensus opinion...');
    const refOpinion = await this.callOpenRouter(apiKey, profile.refiner_model, consensusPrompt);
    console.log(`  Refiner raw response: "${refOpinion.content}"`);
    const refVote = this.parseConsensusOpinion(refOpinion.content);
    opinions.push(refVote);
    console.log(`  Refiner vote: ${refVote}`);
    
    // Log consensus vote for analysis
    await this.logConsensusResponse(
      this.conversation!.rounds_completed,
      'consensus_check_refiner',
      profile.refiner_model,
      consensusPrompt,
      refOpinion.content,
      refVote
    );
    
    // Log consensus check iteration
    this.logIteration(
      this.conversation!.consensus_id,
      profile.refiner_model,
      'consensus_check_refiner',
      refOpinion.usage.total_tokens,
      refVote === 'NO' ? 1 : 0,
      this.conversation!.rounds_completed
    );
    
    // Validator's opinion
    console.log('🤔 Asking Validator for consensus opinion...');
    const valOpinion = await this.callOpenRouter(apiKey, profile.validator_model, consensusPrompt);
    console.log(`  Validator raw response: "${valOpinion.content}"`);
    const valVote = this.parseConsensusOpinion(valOpinion.content);
    opinions.push(valVote);
    console.log(`  Validator vote: ${valVote}`);
    
    // Log consensus vote for analysis
    await this.logConsensusResponse(
      this.conversation!.rounds_completed,
      'consensus_check_validator',
      profile.validator_model,
      consensusPrompt,
      valOpinion.content,
      valVote
    );
    
    // Log consensus check iteration
    this.logIteration(
      this.conversation!.consensus_id,
      profile.validator_model,
      'consensus_check_validator',
      valOpinion.usage.total_tokens,
      valVote === 'NO' ? 1 : 0,
      this.conversation!.rounds_completed
    );
    
    // Count how many models accept the response (YES = accept, response is satisfactory)
    const acceptCount = opinions.filter(opinion => opinion === 'YES').length;
    
    // Hybrid consensus approach based on round number
    if (this.conversation!.rounds_completed < this.maxConsensusRounds) {
      // Rounds 1 to (maxRounds-1): Require unanimous consensus (all models agree response is good)
      this.conversation!.consensus_achieved = opinions.every(opinion => opinion === 'YES');
      
      if (this.conversation!.consensus_achieved) {
        console.log('✅ Unanimous consensus achieved - all models agree response is satisfactory');
        this.consensusType = 'unanimous';
      } else {
        console.log(`⏭️ No unanimous consensus in round ${this.conversation!.rounds_completed} - continuing to next round`);
      }
    } else if (this.conversation!.rounds_completed === this.maxConsensusRounds) {
      // Final round: Check unanimous first, then majority, then curator override
      if (opinions.every(opinion => opinion === 'YES')) {
        // Unanimous consensus achieved in final round
        console.log(`✅ Unanimous consensus (3/3) achieved in final round ${this.conversation!.rounds_completed}`);
        this.conversation!.consensus_achieved = true;
        this.consensusType = 'unanimous';
      } else if (acceptCount >= SimpleConsensusEngine.MAJORITY_THRESHOLD) {
        // Majority consensus achieved in final round
        console.log(`✅ Majority consensus (${acceptCount}/3) after ${this.conversation!.rounds_completed} rounds`);
        this.conversation!.consensus_achieved = true;
        this.consensusType = 'majority';
      } else {
        // No consensus - use curator judgment as fallback
        console.log(`⚠️ No consensus after ${this.maxConsensusRounds} rounds - using curator judgment`);
        this.conversation!.consensus_achieved = true;
        this.consensusType = 'curator_override';
      }
    }
    
    // Log consensus decision
    console.log(`\n📊 Consensus Check Summary:`);
    console.log(`  Generator: ${opinions[0]}`);
    console.log(`  Refiner: ${opinions[1]}`);
    console.log(`  Validator: ${opinions[2]}`);
    console.log(`  Accept Count: ${acceptCount}/3`);
    console.log(`  Round: ${this.conversation!.rounds_completed}/${this.maxConsensusRounds}`);
    console.log(`  Consensus Type: ${this.consensusType}`);
    console.log(`  Consensus Achieved: ${this.conversation!.consensus_achieved ? '✅ YES' : '❌ NO - Continue deliberation'}`);
    
    // Send consensus status to renderer only if consensus type is determined
    if (this.conversation!.consensus_achieved && this.consensusType !== 'pending') {
      this.sendConsensusStatus({
        generator: opinions[0],
        refiner: opinions[1],
        validator: opinions[2],
        achieved: this.conversation!.consensus_achieved,
        consensusType: this.consensusType,
        round: this.conversation!.rounds_completed
      });
    }
    
    // Ensure all stages show as completed after consensus check
    // (they might appear running during consensus voting)
    this.sendStageUpdate('generator', 'completed');
    this.sendStageUpdate('refiner', 'completed');
    this.sendStageUpdate('validator', 'completed');
  }

  private parseConsensusOpinion(response: string): 'YES' | 'NO' {
    const trimmed = response.trim().toUpperCase();
    
    // Log for debugging
    console.log(`    Parsing response: "${trimmed.substring(0, 100)}..."`);
    
    // Handle empty responses as ACCEPT (API failures shouldn't break consensus)
    if (!trimmed || trimmed.length === 0) {
      console.log('    → Empty response, treating as ACCEPT (NO)');
      return 'NO';
    }
    
    // Check if response contains "ACCEPT" anywhere
    // ACCEPT means the answer is good - equivalent to old "NO" (stop iterating)
    if (trimmed === 'ACCEPT' || 
        trimmed.startsWith('ACCEPT') || 
        trimmed.endsWith('ACCEPT') ||
        trimmed.includes('\nACCEPT') ||
        trimmed.includes('ACCEPT\n')) {
      console.log('    → Parsed as NO (found ACCEPT - response is complete)');
      return 'NO';
    }
    
    // Any other response means they provided corrections/improvements
    console.log('    → Parsed as YES (needs corrections/improvements)');
    return 'YES';
  }


  private async curateConsensusResponse(apiKey: string, profile: any, mode: 'polish' | 'choose', contextFramework?: any): Promise<any> {
    console.log(`🎨 CURATOR CALLED - Mode: ${mode}`);
    this.sendStageUpdate('curator', 'running');
    
    let curatorPrompt: string;
    
    if (mode === 'polish') {
      // Polish mode - consensus was reached, just polish the agreed response
      const finalMessage = this.conversation!.messages[this.conversation!.messages.length - 1];
      
      curatorPrompt = `${contextFramework && contextFramework.summary ? `Context from previous conversations:\n${contextFramework.summary}\n\n` : ''}Current question: ${this.conversation!.user_question}

Content to polish and improve:

${finalMessage.content}

Provide an enhanced version of the above content. Improve clarity, formatting, and presentation while preserving all meaning. Do not explain what you're doing or reference the polishing process.`;
    } else {
      // Choose mode - no consensus reached, curator must choose from all 3 responses
      const round3Messages = this.conversation!.messages.filter(m => m.round === 3);
      const generatorResponse = round3Messages.find(m => m.speaker === 'generator')?.content || 'No response';
      const refinerResponse = round3Messages.find(m => m.speaker === 'refiner')?.content || 'No response';
      const validatorResponse = round3Messages.find(m => m.speaker === 'validator')?.content || 'No response';
      
      curatorPrompt = `${contextFramework && contextFramework.summary ? `Context from previous conversations:\n${contextFramework.summary}\n\n` : ''}Current question: ${this.conversation!.user_question}

Reference materials from AI analysis:

[REFERENCE 1]
${generatorResponse}

[REFERENCE 2]  
${refinerResponse}

[REFERENCE 3]
${validatorResponse}

Provide a comprehensive response that synthesizes the best elements from the references above. Do not explain your reasoning, analysis process, or which references you used. Answer directly and professionally as if responding to the original question for the first time.`;
    }
    
    const curatorResult = await this.callOpenRouter(apiKey, profile.curator_model, curatorPrompt);
    this.conversation!.total_tokens += curatorResult.usage.total_tokens;
    this.conversation!.total_cost += this.calculateCost(profile.curator_model, curatorResult.usage);
    
    // Log curator response for analysis
    await this.logConsensusResponse(
      this.conversation!.rounds_completed,
      'curator',
      profile.curator_model,
      curatorPrompt,
      curatorResult.content
    );
    
    // Log curator iteration (after consensus achieved)
    this.logIteration(
      this.conversation!.consensus_id,
      profile.curator_model,
      'curator',
      curatorResult.usage.total_tokens,
      0,  // Curator doesn't vote, so flag is always 0
      this.conversation!.rounds_completed
    );
    
    this.sendStageUpdate('curator', 'completed');
    
    return curatorResult;
  }

  private sendRoundUpdate(round: number) {
    const windows = BrowserWindow.getAllWindows();
    if (windows.length > 0) {
      windows[0].webContents.send('consensus-round-update', { round });
    }
  }

  private sendStageUpdate(stage: string, status: string) {
    // NEVER send curator updates unless consensus is achieved
    if (stage === 'curator' && !this.conversation?.consensus_achieved) {
      console.log(`⚠️ BLOCKED: Attempted to update curator stage without consensus!`);
      return;
    }
    
    console.log(`📡 Stage Update: ${stage} -> ${status}`);
    const windows = BrowserWindow.getAllWindows();
    if (windows.length > 0) {
      windows[0].webContents.send('consensus-stage-update', { stage, status });
    }
  }

  private sendProgressUpdate(requestId: string, message: string, progress: number) {
    const windows = BrowserWindow.getAllWindows();
    if (windows.length > 0) {
      windows[0].webContents.send('progress-update', { requestId, message, progress });
    }
  }

  private sendConsensusStatus(status: any) {
    const windows = BrowserWindow.getAllWindows();
    if (windows.length > 0) {
      windows[0].webContents.send('consensus-status', status);
    }
  }

  private sendConsensusComplete(data: any) {
    const allWindows = BrowserWindow.getAllWindows();
    allWindows.forEach(window => {
      console.log('✅ Sending consensus-complete to renderer');
      window.webContents.send('consensus-complete', data);
    });
  }

  private logIteration(
    consensusId: string,
    modelId: string,
    stageName: string,
    tokensUsed: number,
    flag: 0 | 1,  // 1 if model said NO, 0 otherwise
    roundNumber: number
  ): void {
    try {
      const stmt = this.db.prepare(`
        INSERT INTO consensus_iterations (
          consensus_id, model_id, stage_name, tokens_used, flag, round_number
        ) VALUES (?, ?, ?, ?, ?, ?)
      `);
      
      stmt.run(consensusId, modelId, stageName, tokensUsed, flag, roundNumber);
      console.log(`📝 Logged iteration: ${stageName} - ${modelId} (Round ${roundNumber}, Flag: ${flag})`);
    } catch (error) {
      console.error('❌ Failed to log iteration:', error);
    }
  }

  /**
   * Cleanup resources when engine is destroyed
   */
  async cleanup() {
    console.log('🧹 Cleaning up SimpleConsensusEngine resources');
    
    // Cleanup optimized memory service
    if (this.optimizedMemory) {
      this.optimizedMemory.cleanup();
    }
    
    console.log('✅ SimpleConsensusEngine cleanup complete');
  }
  
  private async updateContextLogRouting(logId: string, routingDecision: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const sql = `
        UPDATE memory_context_logs 
        SET routing_decision = ?, 
            context_influenced_routing = 1
        WHERE log_id = ?
      `;
      
      this.db.run(sql, [routingDecision, logId], (err: Error | null) => {
        if (err) {
          console.error('❌ Error updating context log with routing:', err);
          reject(err);
        } else {
          console.log(`✅ Updated context log ${logId} with routing: ${routingDecision}`);
          resolve();
        }
      });
    });
  }

  private async recordConversationUsage(userId: string, conversationId: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const timestamp = new Date().toISOString();
      const sql = `
        INSERT INTO conversation_usage (
          user_id, conversation_id, timestamp
        ) VALUES (?, ?, ?)
      `;
      
      this.db.run(sql, [userId, conversationId, timestamp], function(err: Error | null) {
        if (err) {
          console.error('❌ Failed to record conversation usage:', err);
          // Don't reject - just log the error and continue
          resolve();
        } else {
          console.log(`✅ Conversation usage recorded for analytics`);
          resolve();
        }
      });
    });
  }
}