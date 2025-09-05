import * as vscode from 'vscode';
import axios from 'axios';

interface McpRequest {
  jsonrpc: string;
  id: number;
  method: string;
  params?: any;
}

interface McpResponse {
  jsonrpc: string;
  id: number;
  result?: any;
  error?: {
    code: number;
    message: string;
    data?: any;
  };
}

interface ToolResult {
  content: Array<{
    type: 'text';
    text: string;
  }>;
  isError?: boolean;
}

class HiveMcpClient {
  private baseUrl: string;
  private requestId = 1;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  private async sendRequest(method: string, params?: any): Promise<any> {
    const request: McpRequest = {
      jsonrpc: '2.0',
      id: this.requestId++,
      method,
      params: params || {}
    };

    try {
      const response = await axios.post(this.baseUrl, request, {
        headers: {
          'Content-Type': 'application/json'
        },
        timeout: 30000
      });

      const mcpResponse: McpResponse = response.data;
      
      if (mcpResponse.error) {
        throw new Error(`MCP Error: ${mcpResponse.error.message}`);
      }

      return mcpResponse.result;
    } catch (error) {
      if (axios.isAxiosError(error)) {
        throw new Error(`Network error: ${error.message}`);
      }
      throw error;
    }
  }

  async initialize(): Promise<void> {
    await this.sendRequest('initialize', {
      protocolVersion: '2024-11-05',
      capabilities: {
        experimental: {}
      },
      clientInfo: {
        name: 'Hive AI VS Code Extension',
        version: '2.0.0'
      }
    });

    await this.sendRequest('initialized');
  }

  async listTools(): Promise<any[]> {
    const result = await this.sendRequest('tools/list');
    return result.tools || [];
  }

  async callTool(name: string, arguments: any): Promise<ToolResult> {
    const result = await this.sendRequest('tools/call', {
      name,
      arguments
    });
    return result;
  }

  async listResources(): Promise<any[]> {
    const result = await this.sendRequest('resources/list');
    return result.resources || [];
  }

  async readResource(uri: string): Promise<any> {
    const result = await this.sendRequest('resources/read', { uri });
    return result;
  }
}

class HiveAiProvider {
  private mcpClient: HiveMcpClient;
  private outputChannel: vscode.OutputChannel;

  constructor(private context: vscode.ExtensionContext) {
    this.outputChannel = vscode.window.createOutputChannel('Hive AI');
    const config = vscode.workspace.getConfiguration('hive');
    const serverUrl = config.get<string>('mcpServerUrl', 'http://127.0.0.1:7777');
    this.mcpClient = new HiveMcpClient(serverUrl);
  }

  async initialize(): Promise<void> {
    try {
      await this.mcpClient.initialize();
      this.outputChannel.appendLine('✅ Connected to Hive AI MCP server');
      
      // List available tools
      const tools = await this.mcpClient.listTools();
      this.outputChannel.appendLine(`🔧 Available tools: ${tools.map(t => t.name).join(', ')}`);
      
    } catch (error) {
      this.outputChannel.appendLine(`❌ Failed to connect to Hive AI server: ${error}`);
      vscode.window.showErrorMessage('Failed to connect to Hive AI server. Please check if the server is running.');
    }
  }

  async askQuestion(): Promise<void> {
    const question = await vscode.window.showInputBox({
      prompt: 'Ask Hive AI a question',
      placeholder: 'What would you like to know?'
    });

    if (!question) {
      return;
    }

    try {
      vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: 'Asking Hive AI...',
        cancellable: false
      }, async (progress) => {
        progress.report({ increment: 0, message: 'Processing with consensus...' });
        
        const result = await this.mcpClient.callTool('ask_hive', {
          question,
          context: this.getCurrentFileContext()
        });

        progress.report({ increment: 100, message: 'Complete!' });

        if (result.content && result.content.length > 0) {
          const response = result.content[0].text;
          this.showResponse(question, response);
        }
      });
    } catch (error) {
      vscode.window.showErrorMessage(`Error: ${error}`);
    }
  }

  async analyzeCode(): Promise<void> {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showWarningMessage('No active editor found');
      return;
    }

    const document = editor.document;
    const filePath = document.fileName;

    try {
      vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: 'Analyzing code with Hive AI...',
        cancellable: false
      }, async (progress) => {
        progress.report({ increment: 0, message: 'Consensus analysis in progress...' });
        
        const result = await this.mcpClient.callTool('analyze_code', {
          path: filePath,
          focus: 'general'
        });

        progress.report({ increment: 100, message: 'Analysis complete!' });

        if (result.content && result.content.length > 0) {
          const analysis = result.content[0].text;
          this.showResponse('Code Analysis', analysis);
        }
      });
    } catch (error) {
      vscode.window.showErrorMessage(`Error analyzing code: ${error}`);
    }
  }

  async explainCode(): Promise<void> {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showWarningMessage('No active editor found');
      return;
    }

    const selection = editor.selection;
    const selectedText = editor.document.getText(selection);

    if (!selectedText) {
      vscode.window.showWarningMessage('Please select some code to explain');
      return;
    }

    try {
      vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: 'Explaining code...',
        cancellable: false
      }, async (progress) => {
        progress.report({ increment: 0, message: 'AI consensus explanation...' });
        
        const result = await this.mcpClient.callTool('explain_code', {
          code: selectedText,
          language: editor.document.languageId
        });

        progress.report({ increment: 100, message: 'Explanation ready!' });

        if (result.content && result.content.length > 0) {
          const explanation = result.content[0].text;
          this.showResponse('Code Explanation', explanation);
        }
      });
    } catch (error) {
      vscode.window.showErrorMessage(`Error explaining code: ${error}`);
    }
  }

  async improveCode(): Promise<void> {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showWarningMessage('No active editor found');
      return;
    }

    const selection = editor.selection;
    const selectedText = editor.document.getText(selection);

    if (!selectedText) {
      vscode.window.showWarningMessage('Please select some code to improve');
      return;
    }

    const focus = await vscode.window.showQuickPick([
      'general',
      'performance',
      'readability',
      'security',
      'error-handling',
      'maintainability'
    ], {
      title: 'What aspect should we focus on?'
    });

    if (!focus) {
      return;
    }

    try {
      vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: 'Improving code...',
        cancellable: false
      }, async (progress) => {
        progress.report({ increment: 0, message: 'AI consensus improvement analysis...' });
        
        const result = await this.mcpClient.callTool('improve_code', {
          code: selectedText,
          language: editor.document.languageId,
          focus
        });

        progress.report({ increment: 100, message: 'Improvements ready!' });

        if (result.content && result.content.length > 0) {
          const improvements = result.content[0].text;
          this.showResponse(`Code Improvements (${focus})`, improvements);
        }
      });
    } catch (error) {
      vscode.window.showErrorMessage(`Error improving code: ${error}`);
    }
  }

  async generateTests(): Promise<void> {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showWarningMessage('No active editor found');
      return;
    }

    const selection = editor.selection;
    const selectedText = editor.document.getText(selection);

    if (!selectedText) {
      vscode.window.showWarningMessage('Please select some code to generate tests for');
      return;
    }

    try {
      vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: 'Generating tests...',
        cancellable: false
      }, async (progress) => {
        progress.report({ increment: 0, message: 'AI consensus test generation...' });
        
        const result = await this.mcpClient.callTool('generate_tests', {
          code: selectedText,
          language: editor.document.languageId,
          test_framework: this.getTestFramework(editor.document.languageId)
        });

        progress.report({ increment: 100, message: 'Tests generated!' });

        if (result.content && result.content.length > 0) {
          const tests = result.content[0].text;
          this.showResponseInNewEditor(tests, this.getTestLanguageId(editor.document.languageId));
        }
      });
    } catch (error) {
      vscode.window.showErrorMessage(`Error generating tests: ${error}`);
    }
  }

  async showStatus(): Promise<void> {
    try {
      const tools = await this.mcpClient.listTools();
      const resources = await this.mcpClient.listResources();
      
      const statusMessage = `
🐝 **Hive AI Status**

**Connection:** ✅ Connected to MCP server
**Tools Available:** ${tools.length}
**Resources Available:** ${resources.length}

**Available Tools:**
${tools.map(tool => `• ${tool.name}: ${tool.description}`).join('\n')}

**Consensus Profile:** ${vscode.workspace.getConfiguration('hive').get('consensusProfile', 'balanced')}
**Auto Analyze:** ${vscode.workspace.getConfiguration('hive').get('autoAnalyze', true) ? '✅' : '❌'}
**Diagnostics:** ${vscode.workspace.getConfiguration('hive').get('enableDiagnostics', true) ? '✅' : '❌'}
**Completions:** ${vscode.workspace.getConfiguration('hive').get('enableCompletions', true) ? '✅' : '❌'}
      `;

      this.showResponse('Hive AI Status', statusMessage);
    } catch (error) {
      vscode.window.showErrorMessage(`Error getting status: ${error}`);
    }
  }

  private getCurrentFileContext(): string {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      return '';
    }

    const document = editor.document;
    return `File: ${document.fileName}\nLanguage: ${document.languageId}`;
  }

  private getTestFramework(languageId: string): string {
    switch (languageId) {
      case 'rust': return 'default';
      case 'javascript':
      case 'typescript': return 'jest';
      case 'python': return 'pytest';
      case 'java': return 'junit';
      case 'csharp': return 'xunit';
      case 'go': return 'testing';
      default: return 'default';
    }
  }

  private getTestLanguageId(languageId: string): string {
    return languageId; // Same language for tests
  }

  private showResponse(title: string, content: string): void {
    const panel = vscode.window.createWebviewPanel(
      'hiveAiResponse',
      title,
      vscode.ViewColumn.Beside,
      {
        enableScripts: true,
        retainContextWhenHidden: true
      }
    );

    panel.webview.html = this.getWebviewContent(title, content);
  }

  private async showResponseInNewEditor(content: string, languageId: string): Promise<void> {
    const document = await vscode.workspace.openTextDocument({
      content,
      language: languageId
    });
    
    await vscode.window.showTextDocument(document, vscode.ViewColumn.Beside);
  }

  private getWebviewContent(title: string, content: string): string {
    const markdownContent = content.replace(/\n/g, '<br>');
    
    return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>${title}</title>
    <style>
        body {
            font-family: var(--vscode-font-family);
            font-size: var(--vscode-font-size);
            color: var(--vscode-foreground);
            background-color: var(--vscode-editor-background);
            padding: 20px;
            line-height: 1.6;
        }
        h1 {
            color: var(--vscode-textLink-foreground);
            border-bottom: 1px solid var(--vscode-textSeparator-foreground);
            padding-bottom: 10px;
        }
        pre {
            background-color: var(--vscode-textCodeBlock-background);
            padding: 15px;
            border-radius: 5px;
            overflow-x: auto;
            white-space: pre-wrap;
        }
        code {
            background-color: var(--vscode-textCodeBlock-background);
            padding: 2px 4px;
            border-radius: 3px;
        }
        .hive-logo {
            color: var(--vscode-textLink-foreground);
            font-weight: bold;
        }
    </style>
</head>
<body>
    <h1><span class="hive-logo">🐝 Hive AI</span> - ${title}</h1>
    <div>${markdownContent}</div>
</body>
</html>`;
  }
}

let hiveProvider: HiveAiProvider;

export function activate(context: vscode.ExtensionContext) {
  console.log('🐝 Hive AI extension is now active!');

  // Initialize the Hive AI provider
  hiveProvider = new HiveAiProvider(context);
  hiveProvider.initialize();

  // Register commands
  const commands = [
    vscode.commands.registerCommand('hive.askQuestion', () => hiveProvider.askQuestion()),
    vscode.commands.registerCommand('hive.analyzeCode', () => hiveProvider.analyzeCode()),
    vscode.commands.registerCommand('hive.explainCode', () => hiveProvider.explainCode()),
    vscode.commands.registerCommand('hive.improveCode', () => hiveProvider.improveCode()),
    vscode.commands.registerCommand('hive.generateTests', () => hiveProvider.generateTests()),
    vscode.commands.registerCommand('hive.showStatus', () => hiveProvider.showStatus()),
    vscode.commands.registerCommand('hive.openSettings', () => {
      vscode.commands.executeCommand('workbench.action.openSettings', '@ext:hivetechs.hive-ai-vscode');
    })
  ];

  context.subscriptions.push(...commands);

  // Set context for when extension is activated
  vscode.commands.executeCommand('setContext', 'hive.activated', true);

  // Show welcome message
  vscode.window.showInformationMessage(
    '🐝 Hive AI is ready! Use Ctrl+Shift+H to ask questions or right-click on code for AI assistance.',
    'Learn More'
  ).then((selection) => {
    if (selection === 'Learn More') {
      vscode.env.openExternal(vscode.Uri.parse('https://docs.hivetechs.com'));
    }
  });
}

export function deactivate() {
  console.log('🐝 Hive AI extension is now deactivated.');
}