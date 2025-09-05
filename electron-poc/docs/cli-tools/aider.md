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

## Advanced Configuration

### YAML Configuration File
Aider supports extensive configuration through `.aider.conf.yml` files located in:
- Home directory (`~/.aider.conf.yml`)
- Git repository root
- Current directory
- Custom location via `--config <filename>`

#### Complete Configuration Example
```yaml
# Model Configuration
model: claude-3-7-sonnet
editor-model: gpt-4o
weak-model: gpt-4o-mini

# API Keys
api-key:
  anthropic: sk-ant-...
  openai: sk-...
  deepseek: sk-...

# Git Integration
auto-commits: true
attribute-author: true
attribute-committer: false
attribute-commit-message-author: true
attribute-commit-message-committer: false
commit-prompt: |
  Write a commit message for these changes.
  Follow conventional commits format.

# Linting and Testing
auto-lint: true
auto-test: true
lint-cmd:
  python: "black . && flake8"
  javascript: "eslint --fix"
  typescript: "tsc --noEmit && eslint --fix"
test-cmd: "pytest -xvs"

# File Watching
watch-files: true
aiderignore: |
  *.log
  *.tmp
  node_modules/
  __pycache__/

# Editor Settings
edit-format: diff
show-diffs: true
yes: false  # Don't auto-approve changes
dry-run: false

# Voice Input
voice: true
voice-format: webm
voice-language: en

# Repository Mapping
map-tokens: 2048
repo-map: true
map-refresh: auto

# Performance
cache-prompts: true
streaming: true
show-repo-map: false

# UI Settings
color: auto
pretty: true
show-model-warnings: true
```

### Custom Workflows and Hooks

#### Pre/Post Edit Hooks
```yaml
# Run before making edits
pre-edit-cmd: "git stash"

# Run after making edits
post-edit-cmd: "npm run format && npm test"

# Custom validation
validate-cmd: "python scripts/validate_changes.py"
```

#### Automated Testing Workflow
```yaml
# Automatically run tests after each change
auto-test: true
test-cmd: "pytest --cov=src"

# Only commit if tests pass
test-before-commit: true

# Custom test patterns
test-files:
  - "test_*.py"
  - "*_test.py"
  - "tests/*.py"
```

#### Code Review Workflow
```yaml
# Automated code review
review-mode: true
review-cmd: "python scripts/ai_review.py"

# Require review before committing
require-review: true

# Review prompts
review-prompts:
  security: "Check for security vulnerabilities"
  performance: "Identify performance bottlenecks"
  style: "Ensure code follows project style guide"
```

### Environment Variables
All configuration options can be set via environment variables:

```bash
# Model settings
export AIDER_MODEL="claude-3-7-sonnet"
export AIDER_EDITOR_MODEL="gpt-4o"
export AIDER_WEAK_MODEL="gpt-4o-mini"

# API keys
export AIDER_ANTHROPIC_API_KEY="sk-ant-..."
export AIDER_OPENAI_API_KEY="sk-..."

# Behavior settings
export AIDER_AUTO_COMMITS="true"
export AIDER_AUTO_LINT="true"
export AIDER_AUTO_TEST="true"

# Custom commands
export AIDER_LINT_CMD="black . && flake8"
export AIDER_TEST_CMD="pytest -xvs"
export AIDER_COMMIT_PROMPT="Write a conventional commit message"

# Advanced settings
export AIDER_CACHE_PROMPTS="true"
export AIDER_MAP_TOKENS="2048"
export AIDER_VOICE="true"
```

### Custom Instructions and Conventions

#### Project Conventions File
Create a `.aider.conventions.md` file in your project:

```markdown
# Project Conventions

## Code Style
- Use 4 spaces for indentation in Python
- Use 2 spaces for indentation in JavaScript
- Follow PEP 8 for Python code
- Use ESLint rules for JavaScript

## Architecture
- Follow MVC pattern
- Keep controllers thin
- Use dependency injection
- Write unit tests for all public methods

## Documentation
- Document all public APIs
- Include examples in docstrings
- Update README for new features
```

#### Custom System Prompts
```yaml
# Add custom instructions to every interaction
custom-instructions: |
  You are working on a Django project.
  Follow Django best practices.
  Use type hints for all Python functions.
  Write comprehensive docstrings.
  
# Model-specific instructions
model-instructions:
  claude-3-7-sonnet: "Be concise but thorough"
  gpt-4o: "Focus on performance optimization"
```

### Advanced Model Settings

#### Custom Model Configuration
```yaml
# Configure unknown or custom models
model-settings:
  - name: "custom-model"
    edit-format: "whole"
    weak-model-temperature: 0.3
    editor-edit-format: "diff"
    editor-temperature: 0.1
    extra-params:
      top_p: 0.95
      frequency_penalty: 0.5
      presence_penalty: 0.5

# Model aliases
model-aliases:
  fast: "gpt-4o-mini"
  smart: "claude-3-7-sonnet"
  cheap: "deepseek-chat"
```

### Integration Capabilities

#### Memory Service Integration
```yaml
# Connect to Hive Memory Service
memory-service:
  enabled: true
  endpoint: "http://localhost:3457"
  auth-token: "your-token"
  auto-sync: true
  sync-interval: 300  # seconds
```

#### External Tool Integration
```yaml
# Integration with external tools
integrations:
  github:
    enabled: true
    auto-pr: true
    pr-template: ".github/pull_request_template.md"
  
  jira:
    enabled: true
    ticket-pattern: "PROJ-\\d+"
    auto-link: true
  
  slack:
    webhook: "https://hooks.slack.com/..."
    notify-on-commit: true
```

#### Custom Commands
```yaml
# Define custom slash commands
custom-commands:
  "/review":
    cmd: "python scripts/code_review.py"
    description: "Run AI code review"
  
  "/deploy":
    cmd: "npm run deploy"
    description: "Deploy to staging"
  
  "/benchmark":
    cmd: "python scripts/benchmark.py"
    description: "Run performance benchmarks"
```

### Performance Optimization

#### Caching Configuration
```yaml
# Optimize for large codebases
cache:
  prompts: true
  repo-map: true
  ttl: 3600  # seconds
  max-size: 1000  # MB

# Context management
context:
  max-tokens: 100000
  compression: true
  smart-selection: true
```

#### Batch Operations
```yaml
# Process multiple files efficiently
batch:
  enabled: true
  parallel: true
  max-concurrent: 4
  chunk-size: 10
```

### Security Configuration

#### API Key Management
```yaml
# Secure API key storage
api-keys:
  storage: "keychain"  # or "env", "file"
  encryption: true
  rotation-days: 90
```

#### File Protection
```yaml
# Prevent editing sensitive files
protected-files:
  - "*.env"
  - "*.key"
  - "*.pem"
  - "secrets/*"

# Require confirmation for certain operations
require-confirmation:
  - delete-files
  - modify-config
  - run-commands
```

## Troubleshooting Advanced Features

### Debug Configuration
```yaml
# Enable debug output
debug: true
verbose: true
show-prompts: true
show-model-warnings: true

# Logging
log-file: "~/.aider/aider.log"
log-level: "DEBUG"
```

### Common Configuration Issues
1. **Conflicting settings**: Command line > .aider.conf.yml > environment variables
2. **Model not found**: Check model name and API key
3. **Hooks not running**: Ensure scripts have execute permissions
4. **Cache issues**: Clear with `rm -rf ~/.aider/cache`

## Links
- [Official Website](https://aider.chat/)
- [GitHub Repository](https://github.com/Aider-AI/aider)
- [PyPI Package](https://pypi.org/project/aider-chat/)
- [Documentation](https://aider.chat/docs/)
- [Configuration Guide](https://aider.chat/docs/config.html)
- [YAML Config Reference](https://aider.chat/docs/config/aider_conf.html)
- [Model Settings](https://aider.chat/docs/config/adv-model-settings.html)