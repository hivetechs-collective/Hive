" Hive AI Vim/Neovim Plugin
" AI-powered code analysis and assistance

if exists('g:loaded_hive_ai')
    finish
endif
let g:loaded_hive_ai = 1

" Configuration
let g:hive_mcp_server_url = get(g:, 'hive_mcp_server_url', 'http://127.0.0.1:7777')
let g:hive_timeout = get(g:, 'hive_timeout', 30)

" Python interface for MCP communication
python3 << EOF
import vim
import json
import urllib.request
import urllib.parse
import urllib.error
import threading

class HiveMcpClient:
    def __init__(self, base_url):
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
            
            timeout = int(vim.eval('g:hive_timeout'))
            with urllib.request.urlopen(req, timeout=timeout) as response:
                response_data = json.loads(response.read().decode('utf-8'))
                
                if 'error' in response_data:
                    raise Exception(f"MCP Error: {response_data['error']['message']}")
                
                return response_data.get('result')
        
        except Exception as e:
            vim.command(f'echohl ErrorMsg | echo "Hive AI Error: {str(e)}" | echohl None')
            return None
    
    def initialize(self):
        if self.initialized:
            return True
        
        result = self.send_request("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {"experimental": {}},
            "clientInfo": {
                "name": "Hive AI Vim Plugin",
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

# Global client instance
hive_client = None

def get_client():
    global hive_client
    if hive_client is None:
        server_url = vim.eval('g:hive_mcp_server_url')
        hive_client = HiveMcpClient(server_url)
    return hive_client

def get_file_language():
    """Detect language from file type"""
    filetype = vim.eval('&filetype')
    
    language_map = {
        'rust': 'rust',
        'python': 'python',
        'javascript': 'javascript',
        'typescript': 'typescript',
        'java': 'java',
        'cpp': 'cpp',
        'c': 'c',
        'go': 'go',
        'php': 'php',
        'ruby': 'ruby',
    }
    
    return language_map.get(filetype, 'unknown')

def create_response_buffer(title, content):
    """Create a new buffer with the response content"""
    vim.command('vnew')
    vim.command('setlocal buftype=nofile')
    vim.command('setlocal bufhidden=wipe')
    vim.command('setlocal noswapfile')
    vim.command('setlocal wrap')
    vim.command('setlocal filetype=markdown')
    vim.command(f'file {title}')
    
    lines = [f"# 🐝 {title}", ""] + content.split('\n')
    vim.current.buffer[:] = lines
    vim.command('setlocal readonly')
    vim.command('setlocal nomodifiable')

EOF

" Ask Hive AI a question
function! HiveAskQuestion()
    let question = input('Ask Hive AI: ')
    if empty(question)
        return
    endif
    
    echo '🐝 Asking Hive AI...'
    
    python3 << EOF
question = vim.eval('question')
client = get_client()

def ask_async():
    try:
        response = client.ask_question(question)
        if response:
            vim.command('redraw')
            create_response_buffer("Hive AI Response", f"## Question\n{question}\n\n## Answer\n{response}")
        else:
            vim.command('redraw | echohl ErrorMsg | echo "No response from Hive AI" | echohl None')
    except Exception as e:
        vim.command(f'redraw | echohl ErrorMsg | echo "Error: {str(e)}" | echohl None')

# Run in background
threading.Thread(target=ask_async).start()
EOF

endfunction

" Explain selected code
function! HiveExplainCode() range
    if !HiveHasSelection()
        echo 'Please select some code to explain'
        return
    endif
    
    let selected_text = HiveGetSelectedText()
    
    echo '🐝 Explaining code with Hive AI...'
    
    python3 << EOF
selected_text = vim.eval('selected_text')
language = get_file_language()
client = get_client()

def explain_async():
    try:
        response = client.explain_code(selected_text, language)
        if response:
            vim.command('redraw')
            create_response_buffer("Code Explanation", response)
        else:
            vim.command('redraw | echohl ErrorMsg | echo "No response from Hive AI" | echohl None')
    except Exception as e:
        vim.command(f'redraw | echohl ErrorMsg | echo "Error: {str(e)}" | echohl None')

threading.Thread(target=explain_async).start()
EOF

endfunction

" Improve selected code
function! HiveImproveCode() range
    if !HiveHasSelection()
        echo 'Please select some code to improve'
        return
    endif
    
    let selected_text = HiveGetSelectedText()
    
    " Ask for focus area
    let focus_options = ['general', 'performance', 'readability', 'security', 'error-handling', 'maintainability']
    let focus_choice = inputlist(['Select improvement focus:'] + 
                                \ map(copy(focus_options), 'v:key + 1 . ". " . v:val'))
    
    if focus_choice < 1 || focus_choice > len(focus_options)
        return
    endif
    
    let focus = focus_options[focus_choice - 1]
    
    echo '🐝 Improving code (' . focus . ') with Hive AI...'
    
    python3 << EOF
selected_text = vim.eval('selected_text')
focus = vim.eval('focus')
language = get_file_language()
client = get_client()

def improve_async():
    try:
        response = client.improve_code(selected_text, language, focus)
        if response:
            vim.command('redraw')
            create_response_buffer(f"Code Improvements ({focus})", response)
        else:
            vim.command('redraw | echohl ErrorMsg | echo "No response from Hive AI" | echohl None')
    except Exception as e:
        vim.command(f'redraw | echohl ErrorMsg | echo "Error: {str(e)}" | echohl None')

threading.Thread(target=improve_async).start()
EOF

endfunction

" Show Hive AI status
function! HiveStatus()
    python3 << EOF
import urllib.error

server_url = vim.eval('g:hive_mcp_server_url')
client = get_client()

try:
    connected = client.initialize()
    
    status_content = f"""# 🐝 Hive AI Status

**Connection:** {'✅ Connected' if connected else '❌ Disconnected'}
**Server URL:** {server_url}
**Version:** 2.0.0

## Commands Available:
- `:HiveAsk` - Ask Hive AI a question
- `:HiveExplain` - Explain selected code
- `:HiveImprove` - Improve selected code
- `:HiveStatus` - Show this status

## Key Mappings:
- `<leader>ha` - Ask question
- `<leader>he` - Explain code
- `<leader>hi` - Improve code

## Setup:
1. Start Hive AI MCP server: `hive serve --mode mcp --port 7777`
2. Configure server URL: `let g:hive_mcp_server_url = 'http://127.0.0.1:7777'`
3. Select code and use commands or key mappings

**Need help?** Visit https://docs.hivetechs.com
"""
    
    create_response_buffer("Hive AI Status", status_content)

except Exception as e:
    error_content = f"""# 🐝 Hive AI Status

❌ **Error:** {str(e)}

Please check that the Hive AI MCP server is running:

```bash
hive serve --mode mcp --port 7777
```

And verify your configuration:

```vim
let g:hive_mcp_server_url = 'http://127.0.0.1:7777'
```
"""
    create_response_buffer("Hive AI Status", error_content)
EOF

endfunction

" Helper functions
function! HiveHasSelection()
    return line("'<") != line("'>") || col("'<") != col("'>")
endfunction

function! HiveGetSelectedText()
    let old_reg = getreg('"')
    let old_regtype = getregtype('"')
    
    normal! gvy
    let selected = getreg('"')
    
    call setreg('"', old_reg, old_regtype)
    return selected
endfunction

" Commands
command! HiveAsk call HiveAskQuestion()
command! -range HiveExplain <line1>,<line2>call HiveExplainCode()
command! -range HiveImprove <line1>,<line2>call HiveImproveCode()
command! HiveStatus call HiveStatus()

" Key mappings
nnoremap <silent> <leader>ha :HiveAsk<CR>
vnoremap <silent> <leader>he :HiveExplain<CR>
vnoremap <silent> <leader>hi :HiveImprove<CR>
nnoremap <silent> <leader>hs :HiveStatus<CR>

" Menu items (for GUI)
if has('menu')
    menu &Tools.Hive\ AI.Ask\ Question :HiveAsk<CR>
    menu &Tools.Hive\ AI.Explain\ Code :HiveExplain<CR>
    menu &Tools.Hive\ AI.Improve\ Code :HiveImprove<CR>
    menu &Tools.Hive\ AI.-Sep- :
    menu &Tools.Hive\ AI.Status :HiveStatus<CR>
endif

" Auto-commands
augroup HiveAI
    autocmd!
    " Show status message when plugin loads
    autocmd VimEnter * echo '🐝 Hive AI plugin loaded! Use :HiveStatus for help'
augroup END