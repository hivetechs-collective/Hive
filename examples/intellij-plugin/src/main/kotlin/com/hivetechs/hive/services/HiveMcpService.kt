package com.hivetechs.hive.services

import com.google.gson.Gson
import com.google.gson.annotations.SerializedName
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.Service
import com.intellij.openapi.diagnostic.thisLogger
import kotlinx.coroutines.*
import okhttp3.*
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.RequestBody.Companion.toRequestBody
import java.io.IOException
import java.util.concurrent.atomic.AtomicInteger

data class McpRequest(
    val jsonrpc: String = "2.0",
    val id: Int,
    val method: String,
    val params: Map<String, Any> = emptyMap()
)

data class McpResponse(
    val jsonrpc: String,
    val id: Int,
    val result: Map<String, Any>? = null,
    val error: McpError? = null
)

data class McpError(
    val code: Int,
    val message: String,
    val data: Any? = null
)

data class ToolResult(
    val content: List<ToolContent>,
    @SerializedName("isError")
    val isError: Boolean? = null
)

data class ToolContent(
    val type: String,
    val text: String
)

data class Tool(
    val name: String,
    val description: String,
    @SerializedName("inputSchema")
    val inputSchema: Map<String, Any>
)

data class ListToolsResult(
    val tools: List<Tool>
)

@Service(Service.Level.APP)
class HiveMcpService : CoroutineScope {
    private val job = SupervisorJob()
    override val coroutineContext = Dispatchers.IO + job
    
    private val logger = thisLogger()
    private val gson = Gson()
    private val httpClient = OkHttpClient.Builder()
        .connectTimeout(30, java.util.concurrent.TimeUnit.SECONDS)
        .readTimeout(60, java.util.concurrent.TimeUnit.SECONDS)
        .writeTimeout(30, java.util.concurrent.TimeUnit.SECONDS)
        .build()
    
    private val requestIdCounter = AtomicInteger(1)
    private var serverUrl = "http://127.0.0.1:7777"
    private var isInitialized = false
    
    fun configure(url: String) {
        serverUrl = url
        isInitialized = false
    }
    
    suspend fun initialize(): Boolean {
        if (isInitialized) return true
        
        return try {
            // Initialize MCP connection
            val initResult = sendRequest("initialize", mapOf(
                "protocolVersion" to "2024-11-05",
                "capabilities" to mapOf("experimental" to emptyMap<String, Any>()),
                "clientInfo" to mapOf(
                    "name" to "Hive AI IntelliJ Plugin",
                    "version" to "2.0.0"
                )
            ))
            
            if (initResult != null) {
                // Send initialized notification
                sendRequest("initialized", emptyMap())
                isInitialized = true
                logger.info("Successfully initialized MCP connection to $serverUrl")
                true
            } else {
                logger.warn("Failed to initialize MCP connection")
                false
            }
        } catch (e: Exception) {
            logger.error("Error initializing MCP connection", e)
            false
        }
    }
    
    suspend fun askQuestion(question: String, context: String = ""): String? {
        return try {
            val result = callTool("ask_hive", mapOf(
                "question" to question,
                "context" to context
            ))
            
            result?.content?.firstOrNull()?.text
        } catch (e: Exception) {
            logger.error("Error asking question", e)
            null
        }
    }
    
    suspend fun analyzeCode(path: String, focus: String = "general"): String? {
        return try {
            val result = callTool("analyze_code", mapOf(
                "path" to path,
                "focus" to focus
            ))
            
            result?.content?.firstOrNull()?.text
        } catch (e: Exception) {
            logger.error("Error analyzing code", e)
            null
        }
    }
    
    suspend fun explainCode(code: String, language: String): String? {
        return try {
            val result = callTool("explain_code", mapOf(
                "code" to code,
                "language" to language
            ))
            
            result?.content?.firstOrNull()?.text
        } catch (e: Exception) {
            logger.error("Error explaining code", e)
            null
        }
    }
    
    suspend fun improveCode(code: String, language: String, focus: String = "general"): String? {
        return try {
            val result = callTool("improve_code", mapOf(
                "code" to code,
                "language" to language,
                "focus" to focus
            ))
            
            result?.content?.firstOrNull()?.text
        } catch (e: Exception) {
            logger.error("Error improving code", e)
            null
        }
    }
    
    suspend fun generateTests(code: String, language: String, testFramework: String = "default"): String? {
        return try {
            val result = callTool("generate_tests", mapOf(
                "code" to code,
                "language" to language,
                "test_framework" to testFramework
            ))
            
            result?.content?.firstOrNull()?.text
        } catch (e: Exception) {
            logger.error("Error generating tests", e)
            null
        }
    }
    
    suspend fun listTools(): List<Tool>? {
        return try {
            val result = sendRequest("tools/list", emptyMap())
            val toolsResult = gson.fromJson(gson.toJson(result), ListToolsResult::class.java)
            toolsResult.tools
        } catch (e: Exception) {
            logger.error("Error listing tools", e)
            null
        }
    }
    
    suspend fun getConnectionStatus(): Boolean {
        return try {
            val tools = listTools()
            tools != null
        } catch (e: Exception) {
            false
        }
    }
    
    private suspend fun callTool(name: String, arguments: Map<String, Any>): ToolResult? {
        val result = sendRequest("tools/call", mapOf(
            "name" to name,
            "arguments" to arguments
        )) ?: return null
        
        return gson.fromJson(gson.toJson(result), ToolResult::class.java)
    }
    
    private suspend fun sendRequest(method: String, params: Map<String, Any>): Map<String, Any>? {
        return withContext(Dispatchers.IO) {
            try {
                val requestId = requestIdCounter.getAndIncrement()
                val request = McpRequest(id = requestId, method = method, params = params)
                val requestJson = gson.toJson(request)
                
                logger.debug("Sending MCP request: $method")
                
                val requestBody = requestJson.toRequestBody("application/json".toMediaType())
                val httpRequest = Request.Builder()
                    .url(serverUrl)
                    .post(requestBody)
                    .build()
                
                val response = httpClient.newCall(httpRequest).execute()
                
                if (!response.isSuccessful) {
                    logger.warn("HTTP error: ${response.code}")
                    return@withContext null
                }
                
                val responseBody = response.body?.string()
                if (responseBody == null) {
                    logger.warn("Empty response body")
                    return@withContext null
                }
                
                val mcpResponse = gson.fromJson(responseBody, McpResponse::class.java)
                
                if (mcpResponse.error != null) {
                    logger.warn("MCP error: ${mcpResponse.error.message}")
                    return@withContext null
                }
                
                mcpResponse.result
            } catch (e: IOException) {
                logger.error("Network error during MCP request", e)
                null
            } catch (e: Exception) {
                logger.error("Unexpected error during MCP request", e)
                null
            }
        }
    }
    
    fun dispose() {
        job.cancel()
        httpClient.dispatcher.executorService.shutdown()
        httpClient.connectionPool.evictAll()
    }
    
    companion object {
        fun getInstance(): HiveMcpService {
            return ApplicationManager.getApplication().getService(HiveMcpService::class.java)
        }
    }
}