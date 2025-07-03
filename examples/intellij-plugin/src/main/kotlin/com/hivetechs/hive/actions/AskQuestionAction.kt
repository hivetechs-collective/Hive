package com.hivetechs.hive.actions

import com.hivetechs.hive.services.HiveMcpService
import com.hivetechs.hive.ui.HiveResponseDialog
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.ProgressManager
import com.intellij.openapi.progress.Task
import com.intellij.openapi.ui.Messages
import kotlinx.coroutines.runBlocking

class AskQuestionAction : AnAction() {
    
    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        
        val question = Messages.showInputDialog(
            project,
            "What would you like to ask Hive AI?",
            "Ask Hive AI",
            Messages.getQuestionIcon()
        ) ?: return
        
        if (question.isBlank()) return
        
        val mcpService = HiveMcpService.getInstance()
        
        ProgressManager.getInstance().run(object : Task.Backgroundable(project, "Asking Hive AI...", false) {
            override fun run(indicator: ProgressIndicator) {
                indicator.text = "Processing question with AI consensus..."
                indicator.isIndeterminate = true
                
                try {
                    val response = runBlocking {
                        // Ensure MCP connection is initialized
                        if (!mcpService.initialize()) {
                            throw Exception("Failed to connect to Hive AI MCP server")
                        }
                        
                        mcpService.askQuestion(question)
                    }
                    
                    if (response != null) {
                        ApplicationManager.getApplication().invokeLater {
                            HiveResponseDialog.show(
                                project,
                                "Question: $question",
                                response,
                                "Hive AI Response"
                            )
                        }
                    } else {
                        ApplicationManager.getApplication().invokeLater {
                            Messages.showErrorDialog(
                                project,
                                "No response received from Hive AI. Please check your connection and try again.",
                                "Error"
                            )
                        }
                    }
                } catch (e: Exception) {
                    ApplicationManager.getApplication().invokeLater {
                        Messages.showErrorDialog(
                            project,
                            "Error: ${e.message}",
                            "Hive AI Error"
                        )
                    }
                }
            }
        })
    }
}