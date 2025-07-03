package com.hivetechs.hive.ui

import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.DialogWrapper
import com.intellij.ui.components.JBScrollPane
import com.intellij.util.ui.JBUI
import java.awt.BorderLayout
import java.awt.Dimension
import javax.swing.*

class HiveResponseDialog(
    project: Project?,
    private val title: String,
    private val content: String,
    private val responseTitle: String
) : DialogWrapper(project) {
    
    init {
        init()
        setTitle(responseTitle)
        setResizable(true)
    }
    
    override fun createCenterPanel(): JComponent {
        val panel = JPanel(BorderLayout())
        
        // Title
        val titleLabel = JLabel("<html><h2>🐝 $title</h2></html>")
        titleLabel.border = JBUI.Borders.empty(10, 10, 10, 10)
        panel.add(titleLabel, BorderLayout.NORTH)
        
        // Content
        val textArea = JTextArea(content)
        textArea.isEditable = false
        textArea.lineWrap = true
        textArea.wrapStyleWord = true
        textArea.font = UIManager.getFont("TextArea.font")
        textArea.background = UIManager.getColor("Panel.background")
        
        val scrollPane = JBScrollPane(textArea)
        scrollPane.preferredSize = Dimension(600, 400)
        scrollPane.border = JBUI.Borders.empty(0, 10, 10, 10)
        
        panel.add(scrollPane, BorderLayout.CENTER)
        
        return panel
    }
    
    override fun createActions(): Array<Action> {
        return arrayOf(okAction)
    }
    
    companion object {
        fun show(project: Project?, title: String, content: String, responseTitle: String = "Hive AI") {
            val dialog = HiveResponseDialog(project, title, content, responseTitle)
            dialog.show()
        }
    }
}