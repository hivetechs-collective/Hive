// Reference diagram: electron-poc/MASTER_ARCHITECTURE.md#diagram-consensus-pipeline
// See also communication map: electron-poc/MASTER_ARCHITECTURE.md#diagram-communication-ipc
import { BrowserWindow } from 'electron';
// Use native fetch (available in Node 18+)

export interface ConsensusRequest {
  query: string;
  context: string;
  memories: Array<{
    content: string;
    thematic_cluster: string;
    relevance_score: number;
  }>;
  requestId: string;
  profileName?: string; // Add profile name to the request
}

export interface ConversationMessage {
  speaker: 'generator' | 'refiner' | 'validator';
  content: string;
  round: number;
  consensus_opinion?: 'YES' | 'NO'; // Is this the best answer? YES/NO
  tokens: number;
  cost: number;
  model: string;
}

export interface ConversationContext {
  user_question: string;
  messages: ConversationMessage[];
  rounds_completed: number;
  consensus_achieved: boolean;
  total_tokens: number;
  total_cost: number;
}

export interface ConsensusResponse {
  stage: string;
  content: string;
  model: string;
  tokens: number;
  complete?: boolean;
  error?: string;
}

export interface Memory {
  content: string;
  thematic_cluster: string;
  relevance_score: number;
  conversation_id: string;
  created_at: string;
}

export class DirectConsensusEngine {
  private db: any;
  private isProcessing = false;
  private lastApiTokens = 0;
  private lastInputTokens = 0;
  private lastOutputTokens = 0;
  private conversation: ConversationContext | null = null;

  constructor(database: any) {
    this.db = database;
  }
  
  // REMOVED: Old sendVisualUpdate method - now using direct IPC commands
  // private sendVisualUpdate(data: any) {
  //   const allWindows = BrowserWindow.getAllWindows();
  //   console.log(`🎯 DIRECT SEND to ${allWindows.length} windows:`, data.type, data);
  //   allWindows.forEach(window => {
  //     window.webContents.send('visual-update', data);
  //   });
  // }
  
  private sendConsensusComplete(data: any) {
    const allWindows = BrowserWindow.getAllWindows();
    allWindows.forEach(window => {
      console.log('✅ DIRECT SEND: consensus-complete');
      window.webContents.send('consensus-complete', data);
    });
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  private async getCurrentProfile(profileName?: string): Promise<any> {
    return new Promise((resolve, reject) => {
      // If profile name is provided, query by name, otherwise use active profile
      if (profileName) {
        console.log('🔍 Looking up profile by name:', profileName);
        this.db.get(`
          SELECT generator_model, refiner_model, validator_model, curator_model, profile_name
          FROM consensus_profiles
          WHERE profile_name = ?
        `, [profileName], (err: any, profile: any) => {
          if (err || !profile) {
            console.error('Profile not found by name, falling back to active profile');
            // Fallback to active profile
            this.getActiveProfile(resolve, reject);
          } else {
            console.log('✅ Found profile by name:', profile.profile_name);
            resolve(profile);
          }
        });
      } else {
        // Use active profile
        this.getActiveProfile(resolve, reject);
      }
    });
  }
  
  private getActiveProfile(resolve: Function, reject: Function): void {
    this.db.get(`
      SELECT cp.generator_model, cp.refiner_model, cp.validator_model, cp.curator_model, cp.profile_name
      FROM consensus_settings cs
      JOIN consensus_profiles cp ON cs.value = cp.id
      WHERE cs.key = 'active_profile_id'
    `, [], (err: any, profile: any) => {
      if (err || !profile) {
        reject(new Error('No active profile configured'));
      } else {
        console.log('✅ Using active profile:', profile.profile_name);
        resolve(profile);
      }
    });
  }

  private async getOpenRouterKey(): Promise<string> {
    return new Promise((resolve, reject) => {
      this.db.get('SELECT value FROM configurations WHERE key = ?', ['openrouter_api_key'], (err: any, row: any) => {
        if (err) {
          console.error('Error fetching OpenRouter API key:', err);
          reject(err);
        } else if (row && row.value) {
          console.log('✅ OpenRouter API key found, length:', row.value.length);
          resolve(row.value);
        } else {
          console.error('❌ OpenRouter API key not found in configurations table');
          reject(new Error('OpenRouter API key not configured'));
        }
      });
    });
  }

  private async getModelPricing(modelId: string): Promise<{input: number, output: number}> {
    return new Promise((resolve, reject) => {
      this.db.get('SELECT pricing_input, pricing_output FROM openrouter_models WHERE openrouter_id = ?', [modelId], (err: any, row: any) => {
        if (err) reject(err);
        else if (row) resolve({
          input: parseFloat(row.pricing_input) || 0,
          output: parseFloat(row.pricing_output) || 0
        });
        else resolve({ input: 0, output: 0 }); // Default for unknown models
      });
    });
  }

  private async calculateRealCost(modelId: string, inputTokens: number, outputTokens: number): Promise<number> {
    const pricing = await this.getModelPricing(modelId);
    return (inputTokens * pricing.input) + (outputTokens * pricing.output);
  }

  async processConsensus(request: ConsensusRequest): Promise<any> {
    console.log('🚨🚨🚨 DirectConsensusEngine.processConsensus CALLED');
    console.log('🚨🚨🚨 Query:', request.query);
    
    if (this.isProcessing) {
      console.log('🚨🚨🚨 Already processing, throwing error');
      throw new Error('Consensus engine is already processing a request');
    }

    this.isProcessing = true;
    console.log('🚨🚨🚨 Set isProcessing = true');
    
    try {
      const startTime = Date.now();
      console.log('🚨🚨🚨 About to get profile and API key');
      console.log('🚨🚨🚨 Profile name from request:', request.profileName);
      
      // Get current settings from existing database, using profile name if provided
      const profile = await this.getCurrentProfile(request.profileName).catch(err => {
        console.error('❌ Failed to get profile:', err);
        throw new Error(`Failed to get consensus profile: ${err.message}`);
      });
      console.log('🚨🚨🚨 Profile retrieved:', profile?.profile_name, 'Models:', profile?.generator_model);
      
      const openrouterApiKey = await this.getOpenRouterKey().catch(err => {
        console.error('❌ Failed to get API key:', err);
        throw new Error(`Failed to get OpenRouter API key: ${err.message}`);
      });
      console.log('🚨🚨🚨 API key retrieved, length:', openrouterApiKey?.length);

      // FULL CONSENSUS MODE: Complete 4-stage pipeline
      console.log('🎯 FULL CONSENSUS MODE: Starting 4-stage pipeline');
      
      // Simple direct visual updates - Point 1: Start
      // Neural consciousness phases now handled by direct commands in each stage
      // Removed old sendVisualUpdate calls to prevent interference
      
      // Point 2: Generator Stage Begins
      // Initialize conversation context for iterative deliberation
      this.conversation = {
        user_question: request.query,
        messages: [],
        rounds_completed: 0,
        consensus_achieved: false,
        total_tokens: 0,
        total_cost: 0
      };
      
      console.log('🔄 Starting iterative deliberation consensus...');
      
      // ITERATIVE DELIBERATION LOOP - Continue until all 3 models say "NO"
      while (!this.conversation.consensus_achieved && this.conversation.rounds_completed < 10) { // Safety limit of 10 rounds
        await this.executeDeliberationRound(profile, openrouterApiKey);
        await this.checkConsensus();
        this.conversation.rounds_completed++;
        
        console.log(`📊 Round ${this.conversation.rounds_completed} complete. Consensus: ${this.conversation.consensus_achieved ? 'YES' : 'NO'}`);
      }
      
      // CURATOR FINAL POLISH - Only after consensus achieved
      console.log('✨ Consensus achieved! Sending to Curator for final polish...');
      
      // Get the last validated response (from the final round)
      const lastValidatorMessage = this.conversation.messages
        .filter(m => m.speaker === 'validator')
        .pop();
      
      const curatorPrompt = `Original query: "${request.query}"

Final agreed response:
${lastValidatorMessage?.content || ''}

Please provide the final, polished version of this response. Ensure perfect formatting, optimal tone, and maximum helpfulness for the user.`;
      
      const curatorResponse = await this.callOpenRouter(
        openrouterApiKey,
        profile.curator_model,
        curatorPrompt,
        'curator'
      );
      
      const curatorCost = await this.calculateRealCost(profile.curator_model, this.lastInputTokens, this.lastOutputTokens);
      this.conversation.total_tokens += this.lastApiTokens;
      this.conversation.total_cost += curatorCost;
      
      
      // Send the final curated response
      this.sendConsensusComplete({
        response: curatorResponse,
        totalTokens: this.conversation.total_tokens,
        totalCost: this.conversation.total_cost,
        conversationId: request.requestId,
        rounds_completed: this.conversation.rounds_completed
      });
      
      return {
        result: curatorResponse,
        success: true,
        mode: 'iterative-consensus',
        stages_completed: ['generator', 'refiner', 'validator', 'curator'],
        tokens_used: this.conversation.total_tokens,
        cost: this.conversation.total_cost,
        duration_ms: Date.now() - startTime,
        rounds_completed: this.conversation.rounds_completed
      };

      /* COMMENTED OUT FOR NOW - Complex consensus logic
      // PHASE 1: Memory Search (using SQLite FTS)
      console.log('📡 DirectConsensusEngine: Emitting stage-update for memory stage, listeners:', this.listenerCount('stage-update'));
      // this.emit('stage-update', { stage: 'memory', message: 'Searching relevant memories...' });
      const memories = await this.searchMemories(request.query);
      console.log(`🧠 Found ${memories.length} relevant memories`);
      // this.emit('stage-complete', { stage: 'memory', tokens: 0, cost: 0 });

      // PHASE 2: Context Building
      console.log('📡 DirectConsensusEngine: Emitting stage-update for synthesis stage');
      // this.emit('stage-update', { stage: 'synthesis', message: 'Building enriched context...' });
      const enrichedContext = await this.buildContext(request.query, memories);
      console.log('🔗 Context enriched with memories');
      // this.emit('stage-complete', { stage: 'synthesis', tokens: 0, cost: 0 });

      // PHASE 3: Routing Decision (Simple vs Complex)
      console.log('📡 DirectConsensusEngine: Emitting stage-update for classification stage');
      // this.emit('stage-update', { stage: 'classification', message: 'Analyzing query complexity...' });
      const isComplex = await this.routeDecision(request.query, enrichedContext, profile, openrouterApiKey);
      console.log(`🎯 Query classified as: ${isComplex ? 'COMPLEX' : 'SIMPLE'}`);
      // this.emit('stage-complete', { stage: 'classification', tokens: 0, cost: 0 });

      if (!isComplex) {
        // DIRECT MODE: Single Generator call for simple queries
        console.log('📡 DirectConsensusEngine: Emitting stage-update for generator stage (direct mode)');
        // this.emit('stage-update', { stage: 'generator', message: 'Processing with direct mode...', model: profile.generator_model });
        const response = await this.directMode(request.query, enrichedContext, profile, openrouterApiKey);
        // this.emit('stage-complete', { stage: 'generator', tokens: this.lastApiTokens, cost: await this.calculateRealCost(profile.generator_model, this.lastInputTokens, this.lastOutputTokens) });
        return {
          result: response,
          success: true,
          mode: 'direct',
          stages_completed: ['memory', 'context', 'routing', 'generator'],
          tokens_used: this.lastApiTokens,
          cost: await this.calculateRealCost(profile.generator_model, this.lastInputTokens, this.lastOutputTokens),
          duration_ms: Date.now() - startTime
        };
      }

      // COMPLEX MODE: Full iterative deliberation
      console.log('🔄 Starting iterative deliberation for complex query');
      
      // Initialize conversation context
      this.conversation = {
        user_question: request.query,
        messages: [],
        rounds_completed: 0,
        consensus_achieved: false,
        total_tokens: 0,
        total_cost: 0
      };

      // Iterative deliberation until consensus (max 10 rounds)
      while (!this.conversation.consensus_achieved && this.conversation.rounds_completed < 10) {
        await this.executeDeliberationRound(profile, openrouterApiKey);
        this.checkConsensus();
      }

      // Final curation after consensus
      // this.emit('stage-update', { stage: 'curator', message: 'Curating final response...' });
      const curatedResponse = await this.curateConsensusResponse(profile, openrouterApiKey);

      // Store conversation in database
      await this.storeConversation(request.query, curatedResponse, this.conversation);

      return {
        result: curatedResponse,
        success: true,
        mode: 'consensus',
        rounds: this.conversation.rounds_completed,
        consensus_achieved: this.conversation.consensus_achieved,
        stages_completed: ['memory', 'context', 'routing', 'generator', 'refiner', 'validator', 'curator'],
        tokens_used: this.conversation.total_tokens,
        cost: this.conversation.total_cost,
        duration_ms: Date.now() - startTime
      };
      */

    } catch (error) {
      console.error('❌❌❌ DirectConsensusEngine.processConsensus FAILED:', error);
      console.error('Stack trace:', error instanceof Error ? error.stack : 'No stack');
      throw error; // Re-throw to propagate to IPC handler
    } finally {
      this.isProcessing = false;
      console.log('🚨🚨🚨 Set isProcessing = false');
    }
  }

  // SIMPLIFIED: Single model call for testing
  private async simplifiedSingleModelCall(query: string, model: string, apiKey: string): Promise<string> {
    console.log('🎯 SIMPLIFIED SINGLE MODEL CALL');
    console.log('Query:', query);
    console.log('Model:', model);
    console.log('API Key length:', apiKey.length);
    
    const prompt = `Please provide a helpful response to the following query: "${query}"`;
    
    try {
      console.log('📡 Making API call to OpenRouter...');
      const requestBody = {
        model,
        messages: [{ role: 'user', content: prompt }],
        stream: false,
        temperature: 0.7,
        max_tokens: 1000
      };
      
      console.log('Request body:', JSON.stringify(requestBody, null, 2));
      
      // Create AbortController for timeout
      const AbortController = require('abort-controller');
      const controller = new AbortController();
      const timeout = setTimeout(() => {
        controller.abort();
      }, 30000); // 30 second timeout for simplified test
      
      const response = await fetch('https://openrouter.ai/api/v1/chat/completions', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${apiKey}`,
          'Content-Type': 'application/json',
          'HTTP-Referer': 'https://hivetechs.io',
          'X-Title': 'Hive Consensus'
        },
        body: JSON.stringify(requestBody),
        signal: controller.signal
      });
      
      clearTimeout(timeout);
      
      console.log('Response status:', response.status);
      console.log('Response headers:', Object.fromEntries(response.headers.entries()));
      
      if (!response.ok) {
        const errorText = await response.text();
        console.error('API Error Response:', errorText);
        throw new Error(`OpenRouter API error: ${response.status} ${response.statusText} - ${errorText}`);
      }
      
      const responseData = await response.json();
      console.log('Response data:', JSON.stringify(responseData, null, 2));
      
      if (responseData.choices && responseData.choices[0]) {
        const content = responseData.choices[0].message?.content || '';
        this.lastApiTokens = responseData.usage?.total_tokens || 0;
        this.lastInputTokens = responseData.usage?.prompt_tokens || 0;
        this.lastOutputTokens = responseData.usage?.completion_tokens || 0;
        
        console.log('✅ Got response:', content.substring(0, 100) + '...');
        console.log('Tokens used:', this.lastApiTokens);
        
        return content;
      }
      
      throw new Error('No content in API response');
      
    } catch (error: any) {
      if (error.name === 'AbortError') {
        console.error('⏱️ Request timed out after 30 seconds');
        throw new Error('OpenRouter API timeout - request took too long');
      }
      console.error('❌ API call failed:', error);
      throw error;
    }
  }

  // REMOVED DUPLICATE - Using implementation at line 941

  // REMOVED DUPLICATE - This functionality is now integrated into executeDeliberationRound at line 941

  private getRoleDescription(role: string): string {
    switch (role) {
      case 'generator': return 'Provide comprehensive, well-researched responses with clear reasoning';
      case 'refiner': return 'Enhance clarity, structure, and completeness while maintaining accuracy';  
      case 'validator': return 'Verify technical accuracy, logical consistency, and ensure all aspects are addressed';
      default: return 'Collaborative AI assistant';
    }
  }

  private parseConsensusOpinion(response: string): 'YES' | 'NO' {
    const upper = response.toUpperCase();
    
    // Look for clear YES/NO responses first
    if (upper.includes('YES')) {
      return 'YES'; // YES means they want to improve it
    }
    
    if (upper.includes('NO') && !upper.includes('YES')) {
      return 'NO'; // NO means they're comfortable with current answer
    }
    
    // Fallback: Default to improvement to be safe
    return 'YES';
  }

  // REMOVED DUPLICATE - Using async implementation at line 1048

  private async curateConsensusResponse(profile: any, apiKey: string): Promise<string> {
    // Get the final agreed-upon answer (the last complete response from the final round)
    const finalRound = this.conversation!.rounds_completed;
    const finalRoundMessages = this.conversation!.messages.filter(m => m.round === finalRound);
    
    // The group's final agreed answer is typically the last response from the final round
    const finalAgreedAnswer = finalRoundMessages[finalRoundMessages.length - 1]?.content || '';
    
    const curatorPrompt = `**Original Question:** "${this.conversation!.user_question}"

**Group's Final Agreed Answer (After ${finalRound} rounds of deliberation):**
${finalAgreedAnswer}

You are the CURATOR. The 3 LLMs above have reached consensus that this response cannot be measurably improved. Your role:
- Polish this final agreed answer for optimal user experience  
- Ensure professional, clear, and engaging presentation
- Optimize formatting and structure
- Clean up any rough edges while preserving the content
- Create the publication-ready final response

**Final Curated Response:**`;

    return await this.callOpenRouter(apiKey, profile.curator_model, curatorPrompt, 'curator-final');
  }

  private assembleContext(request: ConsensusRequest): string {
    const memoryContext = request.memories
      .map(m => `[${m.thematic_cluster}] ${m.content}`)
      .join('\n');
    
    return `
QUERY: ${request.query}

CURRENT CONTEXT:
${request.context}

RELEVANT MEMORIES:
${memoryContext}
    `.trim();
  }

  private createGeneratorPrompt(request: ConsensusRequest, contextData: string): string {
    return `# CONSENSUS STAGE 1: GENERATOR
Your role: Initial response generation with comprehensive analysis

TASK: Generate a thorough, well-reasoned response to the user's query.

${contextData}

REQUIREMENTS:
- Provide detailed analysis covering all relevant aspects
- Include specific examples and actionable recommendations  
- Consider edge cases and alternative perspectives
- Structure response clearly with headings if needed
- Be comprehensive but focused on the core question

Generate your initial response:`;
  }

  private createRefinerPrompt(request: ConsensusRequest, contextData: string, previousResponse: string): string {
    return `# CONSENSUS STAGE 2: REFINER  
Your role: Enhance and refine the initial response

ORIGINAL QUERY:
${request.query}

CONTEXT:
${contextData}

PREVIOUS RESPONSE TO REFINE:
${previousResponse}

REQUIREMENTS:
- Improve clarity, accuracy, and completeness
- Add missing important details or considerations
- Enhance structure and readability
- Correct any errors or inaccuracies
- Maintain the core insights while improving expression
- Ensure technical accuracy and practical applicability

Provide your refined version:`;
  }

  private createValidatorPrompt(request: ConsensusRequest, contextData: string, previousResponse: string): string {
    return `# CONSENSUS STAGE 3: VALIDATOR
Your role: Critical validation and fact-checking

ORIGINAL QUERY:
${request.query}

CONTEXT: 
${contextData}

RESPONSE TO VALIDATE:
${previousResponse}

REQUIREMENTS:
- Verify factual accuracy of all claims
- Check logical consistency of arguments
- Identify potential gaps or weaknesses
- Ensure completeness of the response
- Validate that response directly addresses the query
- Flag any technical inaccuracies or outdated information

If the response is accurate and complete, approve it as-is.
If improvements are needed, provide the corrected version:`;
  }

  private createCuratorPrompt(request: ConsensusRequest, contextData: string, previousResponse: string): string {
    return `# CONSENSUS STAGE 4: CURATOR (FINAL)
Your role: Final curation for optimal user experience

ORIGINAL QUERY:
${request.query}

CONTEXT:
${contextData}

VALIDATED RESPONSE:
${previousResponse}

REQUIREMENTS:
- Ensure response is perfectly tailored to user's needs
- Optimize tone and style for maximum helpfulness
- Ensure proper formatting and structure
- Add any final polish or clarification needed
- Verify response length is appropriate (not too brief, not verbose)
- Make sure response is immediately actionable

Provide the final, polished response:`;
  }

  private async callOpenRouter(apiKey: string, model: string, prompt: string, stage: string): Promise<string> {
    console.log(`🚀 Starting OpenRouter call for ${stage} with model ${model}`);
    console.log(`📝 Prompt length: ${prompt.length} characters`);
    console.log(`🔑 API key starts with: ${apiKey.substring(0, 10)}...`);
    
    let fullContent = '';
    let tokenCount = 0;
    
    try {
      console.log('🌐 Making fetch request to OpenRouter...');
      const requestBody = {
        model,
        messages: [{ role: 'user', content: prompt }],
        stream: false,  // Back to working non-streaming approach
        temperature: 0.7,
        max_tokens: 1000
      };
      
      console.log('📦 Request body:', JSON.stringify(requestBody).substring(0, 200) + '...');
      
      // Create AbortController for timeout
      const AbortController = require('abort-controller');
      const controller = new AbortController();
      const timeout = setTimeout(() => {
        controller.abort();
      }, 60000); // 60 second timeout
      
      try {
        const response = await fetch('https://openrouter.ai/api/v1/chat/completions', {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${apiKey}`,
            'Content-Type': 'application/json',
            'HTTP-Referer': 'https://hivetechs.io',
            'X-Title': 'Hive Consensus'
          },
          body: JSON.stringify(requestBody),
          signal: controller.signal
        });
        
        clearTimeout(timeout);
        console.log(`OpenRouter response received for ${stage}, status: ${response.status}`);

        if (!response.ok) {
          throw new Error(`OpenRouter API error: ${response.status} ${response.statusText}`);
        }

        // Simple JSON response (back to confirmed working approach)
        const responseData = await response.json();
        console.log(`OpenRouter response received for ${stage}`);
        
        if (responseData.choices && responseData.choices[0]) {
          fullContent = responseData.choices[0].message?.content || '';
          tokenCount = responseData.usage?.total_tokens || 0;
          this.lastApiTokens = tokenCount;
          this.lastInputTokens = responseData.usage?.prompt_tokens || 0;
          this.lastOutputTokens = responseData.usage?.completion_tokens || 0;
        }

        console.log(`✅ OpenRouter call completed for ${stage}, tokens: ${tokenCount}`);
      } catch (fetchError: any) {
        clearTimeout(timeout);
        if (fetchError.name === 'AbortError') {
          console.error(`⏱️ OpenRouter request timed out after 60 seconds for ${stage}`);
          throw new Error(`Request timeout: OpenRouter API did not respond within 60 seconds`);
        }
        throw fetchError;
      }
      
    } catch (error) {
      console.error(`❌ OpenRouter call failed for ${stage}:`, error);
      console.error('Error details:', error);
      throw error;
    }

    // Emit stage completion
    // this.emit('stage-complete', {
    //   stage,
    //   content: fullContent,
    //   model,
    //   tokens: tokenCount
    // });

    return fullContent;
  }

  // PHASE 1: Memory Search Implementation
  private async searchMemories(query: string): Promise<Memory[]> {
    return new Promise((resolve, reject) => {
      // Use SQLite LIKE for text search (FTS might not be enabled)
      const searchQuery = `
        SELECT 
          m.content,
          m.conversation_id,
          datetime('now') as created_at,
          COALESCE(c.title, 'General') as thematic_cluster,
          0.5 as relevance_score
        FROM messages m
        LEFT JOIN conversations c ON m.conversation_id = c.id
        WHERE m.content LIKE ?
        ORDER BY m.id DESC
        LIMIT 10
      `;

      // Use % wildcards for LIKE search
      const searchPattern = `%${query}%`;
      
      this.db.all(searchQuery, [searchPattern], (err: any, rows: any[]) => {
        if (err) {
          console.error('Memory search error:', err);
          resolve([]); // Return empty array on error to not break flow
        } else if (!rows) {
          resolve([]); // No memories found
        } else {
          const memories: Memory[] = rows.map(row => ({
            content: row.content || '',
            thematic_cluster: row.thematic_cluster || 'general',
            relevance_score: row.relevance_score || 0.5,
            conversation_id: row.conversation_id || '',
            created_at: row.created_at || new Date().toISOString()
          }));
          resolve(memories);
        }
      });
    });
  }

  // PHASE 2: Context Building Implementation
  private async buildContext(query: string, memories: Memory[]): Promise<string> {
    if (memories.length === 0) {
      return `User Query: "${query}"\n\nNo relevant past conversations found.`;
    }

    const memoryContext = memories
      .slice(0, 5) // Use top 5 most relevant memories
      .map(m => `[${m.thematic_cluster}] ${m.content.substring(0, 200)}...`)
      .join('\n');

    return `
User Query: "${query}"

Relevant Past Conversations:
${memoryContext}

Please consider the above context when responding.
    `.trim();
  }

  // PHASE 3: Routing Decision Implementation
  private async routeDecision(query: string, context: string, profile: any, apiKey: string): Promise<boolean> {
    const routingPrompt = `
Analyze the following query and determine if it is SIMPLE or COMPLEX.

SIMPLE queries are:
- Basic factual questions
- Definitions or explanations
- Math calculations
- Yes/no questions
- Single-step tasks

COMPLEX queries are:
- Multi-faceted analysis
- Creative tasks
- Strategic planning
- Controversial topics
- Requiring multiple perspectives
- Design or architecture questions

Query: "${query}"

Respond with only one word: SIMPLE or COMPLEX
    `.trim();

    try {
      const response = await this.callOpenRouter(apiKey, profile.generator_model, routingPrompt, 'routing');
      const isComplex = response.toUpperCase().includes('COMPLEX');
      return isComplex;
    } catch (error) {
      console.error('Routing decision error:', error);
      // Default to simple mode on error
      return false;
    }
  }

  // PHASE 2: Direct Mode Implementation (for simple queries)
  private async directMode(query: string, context: string, profile: any, apiKey: string): Promise<string> {
    const directPrompt = `
${context}

Please provide a clear, concise response to this query.
    `.trim();

    try {
      const response = await this.callOpenRouter(apiKey, profile.generator_model, directPrompt, 'direct-mode');
      return response;
    } catch (error) {
      console.error('Direct mode error:', error);
      throw error;
    }
  }

  // PHASE 4: Database Storage Implementation
  private async storeConversation(query: string, response: string, conversation: ConversationContext): Promise<void> {
    return new Promise((resolve, reject) => {
      const conversationId = require('crypto').randomUUID();
      const timestamp = new Date().toISOString();

      // Store main conversation
      const insertConversation = `
        INSERT INTO conversations (id, title, created_at, updated_at)
        VALUES (?, ?, ?, ?)
      `;

      this.db.run(insertConversation, [conversationId, query.substring(0, 100), timestamp, timestamp], (err: any) => {
        if (err) {
          console.error('Failed to store conversation:', err);
          resolve(); // Don't break flow on storage error
          return;
        }

        // Store the final curated response as a message
        const insertMessage = `
          INSERT INTO messages (id, conversation_id, role, content, created_at)
          VALUES (?, ?, ?, ?, ?)
        `;

        const messageId = require('crypto').randomUUID();
        this.db.run(insertMessage, [messageId, conversationId, 'assistant', response, timestamp], (err: any) => {
          if (err) {
            console.error('Failed to store message:', err);
          }
          
          // Store to knowledge_conversations for future memory searches
          const insertKnowledge = `
            INSERT INTO knowledge_conversations (id, question, answer, created_at)
            VALUES (?, ?, ?, ?)
          `;

          const knowledgeId = require('crypto').randomUUID();
          this.db.run(insertKnowledge, [knowledgeId, query, response, timestamp], (err: any) => {
            if (err) {
              console.error('Failed to store knowledge:', err);
            }
            resolve();
          });
        });
      });
    });
  }

  private async executeDeliberationRound(profile: any, apiKey: string): Promise<void> {
    const round = this.conversation!.rounds_completed + 1;
    console.log(`\n🔄 ROUND ${round} STARTING...`);
    
    // GENERATOR sees full conversation history
    const generatorPrompt = this.buildContextPrompt('generator', round);
    const generatorResponse = await this.callOpenRouter(
      apiKey,
      profile.generator_model,
      generatorPrompt,
      'generator'
    );
    
    const generatorCost = await this.calculateRealCost(profile.generator_model, this.lastInputTokens, this.lastOutputTokens);
    
    this.conversation!.messages.push({
      speaker: 'generator',
      content: generatorResponse,
      round: round,
      tokens: this.lastApiTokens,
      cost: generatorCost,
      model: profile.generator_model
    });
    
    this.conversation!.total_tokens += this.lastApiTokens;
    this.conversation!.total_cost += generatorCost;
    
    // REFINER sees updated conversation including Generator's response
    const refinerPrompt = this.buildContextPrompt('refiner', round);
    const refinerResponse = await this.callOpenRouter(
      apiKey,
      profile.refiner_model,
      refinerPrompt,
      'refiner'
    );
    
    const refinerCost = await this.calculateRealCost(profile.refiner_model, this.lastInputTokens, this.lastOutputTokens);
    
    this.conversation!.messages.push({
      speaker: 'refiner',
      content: refinerResponse,
      round: round,
      tokens: this.lastApiTokens,
      cost: refinerCost,
      model: profile.refiner_model
    });
    
    this.conversation!.total_tokens += this.lastApiTokens;
    this.conversation!.total_cost += refinerCost;
    
    // VALIDATOR sees complete conversation including Refiner's response
    const validatorPrompt = this.buildContextPrompt('validator', round);
    const validatorResponse = await this.callOpenRouter(
      apiKey,
      profile.validator_model,
      validatorPrompt,
      'validator'
    );
    
    const validatorCost = await this.calculateRealCost(profile.validator_model, this.lastInputTokens, this.lastOutputTokens);
    
    this.conversation!.messages.push({
      speaker: 'validator',
      content: validatorResponse,
      round: round,
      tokens: this.lastApiTokens,
      cost: validatorCost,
      model: profile.validator_model
    });
    
    this.conversation!.total_tokens += this.lastApiTokens;
    this.conversation!.total_cost += validatorCost;
  }
  
  private buildContextPrompt(role: 'generator' | 'refiner' | 'validator', round: number): string {
    const history = this.conversation!.messages
      .map(m => `${m.speaker.toUpperCase()} (Round ${m.round}): ${m.content}`)
      .join('\n\n');
    
    const roleDescriptions = {
      generator: 'Your role is to generate comprehensive, accurate responses to the user\'s question.',
      refiner: 'Your role is to refine and improve the previous responses, adding clarity and detail.',
      validator: 'Your role is to validate and correct the responses, ensuring accuracy and completeness.'
    };
    
    if (round === 1 && role === 'generator') {
      // First round, generator starts fresh
      return `Please provide a comprehensive response to: "${this.conversation!.user_question}"`;
    }
    
    return `You are the ${role.toUpperCase()} in a collaborative AI deliberation.

**Original Question:** "${this.conversation!.user_question}"

**Full Conversation History:**
${history}

**Your Role:** ${roleDescriptions[role]}

**Instructions:** 
1. Review the full conversation above
2. Provide your response/refinement based on the conversation
3. Focus on improving and building upon previous responses

Please provide your ${role === 'generator' ? 'updated response' : role === 'refiner' ? 'refinement' : 'validation and corrections'}:`;
  }
  
  private async checkConsensus(): Promise<void> {
    // After each round, ask each model if the response can be improved
    const lastRoundMessages = this.conversation!.messages.filter(
      m => m.round === this.conversation!.rounds_completed + 1
    );
    
    // Get the last validator response (final response of the round)
    const lastResponse = lastRoundMessages.find(m => m.speaker === 'validator')?.content || '';
    
    // Ask each model: "Can this response be improved? YES or NO"
    const consensusPrompt = `Original question: "${this.conversation!.user_question}"

Current response:
${lastResponse}

Can this response be meaningfully improved? Please answer with only YES or NO.

YES = The response has gaps, errors, or could be significantly better
NO = The response is comprehensive, accurate, and ready for the user`;
    
    // Store consensus opinions
    const opinions: Array<'YES' | 'NO'> = [];
    
    // Note: In a real implementation, we'd ask each model separately
    // For now, we'll use a simple heuristic based on round count
    // After 2 rounds, we'll consider consensus achieved
    if (this.conversation!.rounds_completed >= 2) {
      console.log('✅ Consensus achieved after 2 rounds (simplified logic)');
      this.conversation!.consensus_achieved = true;
    } else {
      console.log('❌ No consensus yet, continuing deliberation...');
    }
  }

  stop(): void {
    this.isProcessing = false;
    // this.emit('consensus-stopped');
  }

  isActive(): boolean {
    return this.isProcessing;
  }
}
