import sublime
import sublime_plugin
import threading
import json
import urllib.request
import urllib.parse
import urllib.error

class HiveMcpClient:
    def __init__(self, base_url="http://127.0.0.1:7777"):
        self.base_url = base_url
        self.request_id = 1
        self.initialized = False
    
    def send_request(self, method, params=None):
        if params is None:
            params = {}
        
        request_data = {
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params
        }
        
        self.request_id += 1
        
        try:
            data = json.dumps(request_data).encode('utf-8')
            req = urllib.request.Request(
                self.base_url,
                data=data,
                headers={'Content-Type': 'application/json'}
            )
            
            with urllib.request.urlopen(req, timeout=30) as response:
                response_data = json.loads(response.read().decode('utf-8'))
                
                if 'error' in response_data:
                    raise Exception(f"MCP Error: {response_data['error']['message']}")
                
                return response_data.get('result')
        
        except Exception as e:
            print(f"Hive AI MCP Error: {e}")
            return None
    
    def initialize(self):
        if self.initialized:
            return True
        
        result = self.send_request("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {"experimental": {}},
            "clientInfo": {
                "name": "Hive AI Sublime Text",
                "version": "2.0.0"
            }
        })
        
        if result:
            self.send_request("initialized")
            self.initialized = True
            return True
        
        return False
    
    def ask_question(self, question, context=""):
        if not self.initialize():
            return None
        
        result = self.send_request("tools/call", {
            "name": "ask_hive",
            "arguments": {
                "question": question,
                "context": context
            }
        })
        
        if result and 'content' in result and len(result['content']) > 0:
            return result['content'][0].get('text')
        
        return None
    
    def explain_code(self, code, language):
        if not self.initialize():
            return None
        
        result = self.send_request("tools/call", {
            "name": "explain_code",
            "arguments": {
                "code": code,
                "language": language
            }
        })
        
        if result and 'content' in result and len(result['content']) > 0:
            return result['content'][0].get('text')
        
        return None
    
    def improve_code(self, code, language, focus="general"):
        if not self.initialize():
            return None
        
        result = self.send_request("tools/call", {
            "name": "improve_code",
            "arguments": {
                "code": code,
                "language": language,
                "focus": focus
            }
        })
        
        if result and 'content' in result and len(result['content']) > 0:
            return result['content'][0].get('text')
        
        return None

# Global MCP client instance
mcp_client = HiveMcpClient()

class HiveAskQuestionCommand(sublime_plugin.WindowCommand):
    def run(self):
        self.window.show_input_panel(
            "Ask Hive AI:",
            "",
            self.on_done,
            None,
            None
        )
    
    def on_done(self, question):
        if not question.strip():
            return
        
        def ask_question():
            try:
                sublime.status_message("🐝 Asking Hive AI...")
                response = mcp_client.ask_question(question)
                
                if response:
                    sublime.set_timeout(lambda: self.show_response(question, response), 0)
                else:
                    sublime.set_timeout(lambda: sublime.error_message("No response from Hive AI"), 0)
            
            except Exception as e:
                sublime.set_timeout(lambda: sublime.error_message(f"Error: {e}"), 0)
        
        threading.Thread(target=ask_question).start()
    
    def show_response(self, question, response):
        view = self.window.new_file()
        view.set_name(f"Hive AI Response")
        view.settings().set('word_wrap', True)
        view.set_syntax_file("Packages/Markdown/Markdown.sublime-syntax")
        
        content = f"# 🐝 Hive AI Response\n\n## Question\n{question}\n\n## Answer\n{response}"
        view.run_command('append', {'characters': content})
        view.set_read_only(True)

class HiveExplainCodeCommand(sublime_plugin.TextCommand):
    def run(self, edit):
        if not self.view.has_non_empty_selection_region():
            sublime.error_message("Please select some code to explain")
            return
        
        selected_text = self.get_selected_text()
        language = self.get_language()
        
        def explain_code():
            try:
                sublime.status_message("🐝 Explaining code with Hive AI...")
                response = mcp_client.explain_code(selected_text, language)
                
                if response:
                    sublime.set_timeout(lambda: self.show_response("Code Explanation", response), 0)
                else:
                    sublime.set_timeout(lambda: sublime.error_message("No response from Hive AI"), 0)
            
            except Exception as e:
                sublime.set_timeout(lambda: sublime.error_message(f"Error: {e}"), 0)
        
        threading.Thread(target=explain_code).start()
    
    def get_selected_text(self):
        selection = self.view.sel()[0]
        return self.view.substr(selection)
    
    def get_language(self):
        syntax = self.view.settings().get('syntax')
        if 'Rust' in syntax:
            return 'rust'
        elif 'Python' in syntax:
            return 'python'
        elif 'JavaScript' in syntax:
            return 'javascript'
        elif 'TypeScript' in syntax:
            return 'typescript'
        elif 'Java' in syntax:
            return 'java'
        elif 'C++' in syntax:
            return 'cpp'
        elif 'Go' in syntax:
            return 'go'
        else:
            return 'unknown'
    
    def show_response(self, title, response):
        view = self.view.window().new_file()
        view.set_name(title)
        view.settings().set('word_wrap', True)
        view.set_syntax_file("Packages/Markdown/Markdown.sublime-syntax")
        
        content = f"# 🐝 {title}\n\n{response}"
        view.run_command('append', {'characters': content})
        view.set_read_only(True)
    
    def is_enabled(self):
        return self.view.has_non_empty_selection_region()

class HiveImproveCodeCommand(sublime_plugin.TextCommand):
    def run(self, edit):
        if not self.view.has_non_empty_selection_region():
            sublime.error_message("Please select some code to improve")
            return
        
        selected_text = self.get_selected_text()
        language = self.get_language()
        
        # Show quick panel for focus selection
        focus_options = [
            "general",
            "performance", 
            "readability",
            "security",
            "error-handling",
            "maintainability"
        ]
        
        def on_focus_selected(index):
            if index == -1:
                return
            
            focus = focus_options[index]
            
            def improve_code():
                try:
                    sublime.status_message(f"🐝 Improving code ({focus}) with Hive AI...")
                    response = mcp_client.improve_code(selected_text, language, focus)
                    
                    if response:
                        sublime.set_timeout(lambda: self.show_response(f"Code Improvements ({focus})", response), 0)
                    else:
                        sublime.set_timeout(lambda: sublime.error_message("No response from Hive AI"), 0)
                
                except Exception as e:
                    sublime.set_timeout(lambda: sublime.error_message(f"Error: {e}"), 0)
            
            threading.Thread(target=improve_code).start()
        
        self.view.window().show_quick_panel(focus_options, on_focus_selected)
    
    def get_selected_text(self):
        selection = self.view.sel()[0]
        return self.view.substr(selection)
    
    def get_language(self):
        syntax = self.view.settings().get('syntax')
        if 'Rust' in syntax:
            return 'rust'
        elif 'Python' in syntax:
            return 'python'
        elif 'JavaScript' in syntax:
            return 'javascript'
        elif 'TypeScript' in syntax:
            return 'typescript'
        elif 'Java' in syntax:
            return 'java'
        elif 'C++' in syntax:
            return 'cpp'
        elif 'Go' in syntax:
            return 'go'
        else:
            return 'unknown'
    
    def show_response(self, title, response):
        view = self.view.window().new_file()
        view.set_name(title)
        view.settings().set('word_wrap', True)
        view.set_syntax_file("Packages/Markdown/Markdown.sublime-syntax")
        
        content = f"# 🐝 {title}\n\n{response}"
        view.run_command('append', {'characters': content})
        view.set_read_only(True)
    
    def is_enabled(self):
        return self.view.has_non_empty_selection_region()

class HiveStatusCommand(sublime_plugin.WindowCommand):
    def run(self):
        def check_status():
            try:
                settings = sublime.load_settings("HiveAI.sublime-settings")
                server_url = settings.get("mcp_server_url", "http://127.0.0.1:7777")
                
                # Test connection
                client = HiveMcpClient(server_url)
                connected = client.initialize()
                
                status_msg = f"""# 🐝 Hive AI Status

**Connection:** {'✅ Connected' if connected else '❌ Disconnected'}
**Server URL:** {server_url}
**Version:** 2.0.0

## Commands Available:
- **Ctrl+Shift+H**: Ask Hive AI
- **Ctrl+Shift+E**: Explain selected code  
- **Ctrl+Shift+I**: Improve selected code

## Setup:
1. Start Hive AI MCP server: `hive serve --mode mcp --port 7777`
2. Configure server URL in Preferences → Package Settings → Hive AI
3. Select code and use right-click menu or keyboard shortcuts

**Need help?** Visit https://docs.hivetechs.com
"""
                
                sublime.set_timeout(lambda: self.show_status(status_msg), 0)
            
            except Exception as e:
                error_msg = f"# 🐝 Hive AI Status\n\n❌ **Error:** {e}\n\nPlease check that the Hive AI MCP server is running."
                sublime.set_timeout(lambda: self.show_status(error_msg), 0)
        
        threading.Thread(target=check_status).start()
    
    def show_status(self, content):
        view = self.window.new_file()
        view.set_name("Hive AI Status")
        view.settings().set('word_wrap', True)
        view.set_syntax_file("Packages/Markdown/Markdown.sublime-syntax")
        view.run_command('append', {'characters': content})
        view.set_read_only(True)