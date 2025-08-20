# Aider Documentation

## Overview
Aider is an AI pair programming tool that works in your terminal, supporting multiple LLMs including Claude 3.7 Sonnet, DeepSeek, OpenAI GPT-4o, and local models. It enables collaborative coding with git integration and multi-file editing capabilities.

**Latest Version**: Check with `pip show aider-chat`  
**PyPI Package**: `aider-chat`  
**GitHub**: https://github.com/Aider-AI/aider  
**Documentation**: https://aider.chat/docs/  
**Stars**: 35.2k+ on GitHub

## Installation

### Prerequisites
- Python 3.9-3.12
- Git (for version control features)
- API keys for chosen LLM provider

### Installation Methods

#### Method 1: Using aider-install (Recommended)
```bash
# Install the installer
python -m pip install aider-install

# Run the installer
aider-install
```

#### Method 2: Direct pip Installation
```bash
# Install with pip
python -m pip install -U --upgrade-strategy only-if-needed aider-chat

# With virtual environment (recommended)
python -m venv aider-env
source aider-env/bin/activate  # On Windows: aider-env\Scripts\activate
pip install aider-chat
```

#### Method 3: Using UV
```bash
# Install UV first
python -m pip install uv

# Install aider with UV (auto-installs Python 3.12 if needed)
uv tool install --force --python python3.12 --with pip aider-chat@latest
```

#### Method 4: Using pipx
```bash
# Install pipx first
python -m pip install pipx

# Install aider
pipx install aider-chat
```

## Configuration

### API Key Setup

#### DeepSeek
```bash
aider --model deepseek --api-key deepseek=<key>
```

#### Claude 3.7 Sonnet
```bash
aider --model sonnet --api-key anthropic=<key>
```

#### OpenAI Models
```bash
aider --model o3-mini --api-key openai=<key>
```

#### Local Models (Ollama)
```bash
# Start Ollama first
ollama serve

# Use with aider
aider --model ollama/codellama
```

### Configuration File
Location: `~/.aider.conf.yml`

```yaml
# Model configuration
model: claude-3-7-sonnet
api-key: 
  anthropic: sk-...
  openai: sk-...
  deepseek: sk-...

# Git settings
auto-commits: true
commit-prompt: true

# Editor settings
edit-format: diff
show-diffs: true

# Context settings
map-tokens: 1024
max-chat-history: 10
```

## Core Features

### Multi-File Editing
Unlike single-file assistants, Aider can:
- Understand relationships between files
- Make coordinated changes across multiple files
- Maintain consistency throughout codebase
- Handle complex refactoring

### Git Integration
```bash
# Aider automatically commits changes
aider --auto-commits

# Disable auto-commits
aider --no-auto-commits

# Custom commit messages
aider --commit-prompt
```

### Voice Coding
```bash
# Enable voice input
aider --voice

# Voice with specific language
aider --voice --voice-language en
```

### Visual Context
```bash
# Add images to chat
aider --image screenshot.png

# Add web pages
aider --web https://docs.example.com
```

## Usage

### Basic Commands
```bash
# Start in current directory
cd /path/to/project
aider

# Start with specific files
aider file1.py file2.py

# Start with all Python files
aider *.py

# Start with specific model
aider --model gpt-4o
```

### In-Chat Commands
```bash
/add <file>     # Add files to chat
/drop <file>    # Remove files from chat
/undo           # Undo last git commit (if by aider)
/diff           # Show git diff
/run <command>  # Run shell command
/help           # Show all commands
/quit           # Exit aider
```

### Working with Files
```bash
# Add files during chat
> /add src/main.py src/utils.py

# Drop files
> /drop test_*.py

# Add entire directory
> /add src/

# Use glob patterns
> /add **/*.js
```

## Memory Service Integration

### Configuration for Hive Memory Service
```yaml
# In .aider.conf.yml
plugins:
  memory_service:
    endpoint: http://localhost:3457
    token: auth-token
    enabled: true
```

### Benefits
- Persist context across sessions
- Share learnings between projects
- Track code patterns and solutions

## Advanced Features

### Codebase Mapping
```bash
# Generate repository map
aider --map

# Use specific map tokens
aider --map-tokens 2048

# Disable mapping
aider --no-map
```

### Edit Formats
```bash
# Diff format (default, efficient)
aider --edit-format diff

# Whole file format (simple, clear)
aider --edit-format whole

# SEARCH/REPLACE format
aider --edit-format search-replace
```

### Context Management
```bash
# Clear chat history
aider --clear

# Limit chat history
aider --max-chat-history 5

# Show token usage
aider --show-tokens
```

## Model Support

### Supported Models
- **Claude**: 3.7 Sonnet, 3.5 Sonnet, Opus
- **OpenAI**: GPT-4o, GPT-4, o3-mini, o1
- **DeepSeek**: R1, Chat V3
- **Google**: Gemini Pro, Gemini Flash
- **Local**: Ollama models, llama.cpp

### Model Selection
```bash
# List available models
aider --list-models

# Use specific model
aider --model claude-3-7-sonnet

# Use local model
aider --model ollama/codellama:34b
```

## Language Support

Aider works with most programming languages:
- Python, JavaScript, TypeScript
- Rust, Go, C++, C
- Ruby, PHP, Java
- HTML, CSS, SQL
- And dozens more...

## Cost Management

### Token Usage
```bash
# Show token usage
aider --show-tokens

# Limit tokens per request
aider --max-tokens 4000
```

### Cost Optimization
- Use smaller models for simple tasks
- Enable caching for repeated operations
- Use local models when possible
- Clear unnecessary context regularly

## Troubleshooting

### Common Issues

1. **API Key Errors**
   ```bash
   # Check API key
   echo $ANTHROPIC_API_KEY
   
   # Set in config file instead
   aider --config
   ```

2. **Git Issues**
   ```bash
   # Initialize git repo
   git init
   
   # Configure git user
   git config user.name "Your Name"
   git config user.email "your@email.com"
   ```

3. **Python Version**
   ```bash
   # Check Python version
   python --version
   
   # Use specific Python
   python3.11 -m pip install aider-chat
   ```

4. **Virtual Environment**
   ```bash
   # Always use venv for isolation
   python -m venv aider-env
   source aider-env/bin/activate
   pip install aider-chat
   ```

## Integration with IDEs

### VS Code
```json
// tasks.json
{
  "version": "2.0.0",
  "tasks": [{
    "label": "Aider",
    "type": "shell",
    "command": "aider",
    "args": ["${file}"],
    "problemMatcher": []
  }]
}
```

### Terminal Integration
```bash
# Add to .bashrc or .zshrc
alias ai="aider"
alias aid="aider --model deepseek"
alias aic="aider --model claude-3-7-sonnet"
```

## Best Practices

1. **Start Small**: Begin with a few files, add more as needed
2. **Use Git**: Enable auto-commits for safety
3. **Clear Context**: Remove unnecessary files to save tokens
4. **Be Specific**: Clear, detailed prompts work best
5. **Review Changes**: Always review before committing

## Comparison with Other Tools

| Feature | Aider | Claude Code | Codex CLI |
|---------|-------|-------------|-----------|
| Multi-file | ✅ Excellent | ✅ Good | ✅ Good |
| Git Integration | ✅ Native | Limited | Limited |
| Local Models | ✅ | ❌ | ❌ |
| Voice Coding | ✅ | ❌ | ❌ |
| Cost | Pay per use | Subscription | Subscription |
| Open Source | ✅ | ❌ | ❌ |

## Community & Support

### Resources
- [GitHub Discussions](https://github.com/Aider-AI/aider/discussions)
- [Discord Server](https://discord.gg/aider)
- [Documentation](https://aider.chat/docs/)
- [Blog](https://aider.chat/blog/)

### Contributing
```bash
# Clone repository
git clone https://github.com/Aider-AI/aider
cd aider

# Install in development mode
pip install -e .

# Run tests
python -m pytest
```

## Links
- [Official Website](https://aider.chat/)
- [GitHub Repository](https://github.com/Aider-AI/aider)
- [PyPI Package](https://pypi.org/project/aider-chat/)
- [Documentation](https://aider.chat/docs/)
- [Installation Guide](https://aider.chat/docs/install.html)