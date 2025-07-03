# HiveTechs Consensus - Troubleshooting Guide

> Comprehensive problem-solving guide for common issues and advanced diagnostics

## Table of Contents

1. [Quick Diagnostics](#quick-diagnostics)
2. [Installation Issues](#installation-issues)
3. [Configuration Problems](#configuration-problems)
4. [Performance Issues](#performance-issues)
5. [API & Network Problems](#api--network-problems)
6. [Memory & Storage Issues](#memory--storage-issues)
7. [IDE Integration Issues](#ide-integration-issues)
8. [Security & Permissions](#security--permissions)
9. [Advanced Diagnostics](#advanced-diagnostics)
10. [Getting Help](#getting-help)

## Quick Diagnostics

### First Steps for Any Issue

```bash
# 1. Check system health
hive health --verbose

# 2. Test basic functionality
hive test --integration

# 3. Check configuration
hive config show

# 4. View recent logs
hive logs --tail 50

# 5. Check for updates
hive version --check-updates
```

### Emergency Recovery

```bash
# Reset configuration to defaults
hive config reset --confirm

# Clear all caches
hive cache clear --all

# Rebuild symbol index
hive index rebuild --force

# Restart all services
hive restart --all-services
```

## Installation Issues

### Command Not Found

**Symptoms:** `hive: command not found` after installation

**Solutions:**

1. **Check if binary exists:**
   ```bash
   ls -la /usr/local/bin/hive
   which hive
   ```

2. **Add to PATH:**
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   export PATH="/usr/local/bin:$PATH"
   
   # Reload shell
   source ~/.bashrc  # or ~/.zshrc
   ```

3. **Install to user directory:**
   ```bash
   cargo install --root ~/.local hive-consensus
   export PATH="$HOME/.local/bin:$PATH"
   ```

4. **Fix permissions:**
   ```bash
   sudo chmod +x /usr/local/bin/hive
   ```

### Permission Denied

**Symptoms:** Permission errors during installation or execution

**Solutions:**

1. **macOS Gatekeeper issues:**
   ```bash
   # Allow unsigned binary
   sudo spctl --add /usr/local/bin/hive
   
   # Or disable Gatekeeper temporarily
   sudo spctl --master-disable
   ```

2. **Linux permission issues:**
   ```bash
   # Fix binary permissions
   sudo chmod 755 /usr/local/bin/hive
   
   # Fix config directory permissions
   chmod -R 755 ~/.hive
   ```

3. **Windows execution policy:**
   ```powershell
   # Set execution policy
   Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
   ```

### Installation Verification

```bash
# Comprehensive installation check
hive --version
hive health
hive config show --redact-secrets
hive test --quick
```

## Configuration Problems

### Invalid Configuration

**Symptoms:** Errors about invalid config values

**Diagnosis:**
```bash
# Validate configuration
hive config validate

# Show configuration with line numbers
hive config show --verbose --format toml
```

**Solutions:**

1. **Reset specific section:**
   ```bash
   hive config reset --section providers
   hive config reset --section performance
   ```

2. **Fix common config errors:**
   ```bash
   # Fix API key format
   hive config set openrouter.api_key "sk-or-your-actual-key"
   
   # Fix memory size format
   hive config set performance.cache_size "2GB"  # not "2G" or "2000MB"
   
   # Fix boolean values
   hive config set consensus.streaming true  # not "true" in quotes
   ```

3. **Restore from backup:**
   ```bash
   # Hive automatically backs up config
   cp ~/.hive/backups/config.toml.backup ~/.hive/config.toml
   ```

### Environment Variable Conflicts

**Symptoms:** Configuration seems correct but behavior is unexpected

**Diagnosis:**
```bash
# Check environment variables
env | grep HIVE_

# Show effective configuration (including env vars)
hive config show --resolved
```

**Solutions:**
```bash
# Clear conflicting environment variables
unset HIVE_CONSENSUS_PROFILE
unset HIVE_CACHE_SIZE

# Or set them correctly
export HIVE_CONSENSUS_PROFILE="balanced"
export HIVE_LOG_LEVEL="info"
```

### API Key Issues

**Symptoms:** Authentication errors, "invalid API key"

**Solutions:**

1. **Check API key format:**
   ```bash
   # OpenRouter keys start with "sk-or-"
   hive config set openrouter.api_key "sk-or-YOUR-KEY"
   
   # Anthropic keys start with "sk-ant-"
   hive config set anthropic.api_key "sk-ant-YOUR-KEY"
   ```

2. **Test API connectivity:**
   ```bash
   # Test specific provider
   hive test --provider openrouter
   hive test --provider anthropic
   ```

3. **Check account status:**
   - Verify account is active and has credits
   - Check rate limits and usage quotas
   - Ensure API key has correct permissions

## Performance Issues

### Slow Response Times

**Symptoms:** Hive responses take longer than expected

**Diagnosis:**
```bash
# Performance monitoring
hive performance monitor --duration 60s

# Check resource usage
hive analytics performance --metrics latency,throughput

# Profile specific operations
hive profile ask "test question" --verbose
```

**Solutions:**

1. **Optimize configuration:**
   ```bash
   # Increase worker count (if you have CPU cores)
   hive config set performance.max_workers 16
   
   # Increase cache size (if you have RAM)
   hive config set performance.cache_size "4GB"
   
   # Enable parallel processing
   hive config set performance.parallel_processing true
   ```

2. **Choose faster consensus profile:**
   ```bash
   # Use speed profile for quick iterations
   hive config set consensus.profile speed
   
   # Or use it per-command
   hive ask "question" --consensus speed
   ```

3. **Network optimization:**
   ```bash
   # Reduce timeout for faster failures
   hive config set providers.openrouter.timeout "30s"
   
   # Use closer endpoints if available
   hive config set providers.openrouter.base_url "https://api.openrouter.ai/api/v1"
   ```

### High Memory Usage

**Symptoms:** Hive using too much RAM, system slowdown

**Diagnosis:**
```bash
# Check memory usage
hive performance monitor --focus memory

# Show cache statistics
hive cache stats

# Memory analysis
ps aux | grep hive
top -p $(pgrep hive)
```

**Solutions:**

1. **Reduce cache size:**
   ```bash
   hive config set performance.cache_size "1GB"
   hive config set performance.memory_limit "2GB"
   ```

2. **Clear caches:**
   ```bash
   hive cache clear --all
   hive index rebuild --minimal
   ```

3. **Optimize for low memory:**
   ```bash
   hive config set performance.low_memory_mode true
   hive config set performance.max_workers 4
   ```

### Slow Startup

**Symptoms:** `hive` command takes long time to start

**Diagnosis:**
```bash
# Profile startup time
time hive --version

# Check what's loading
hive debug startup --trace
```

**Solutions:**

1. **Optimize index loading:**
   ```bash
   # Rebuild index for faster loading
   hive index rebuild --optimize

   # Reduce index size
   hive index clean --remove-stale
   ```

2. **Disable unnecessary features:**
   ```bash
   # Disable TUI auto-detection
   hive config set ui.tui.enabled false
   
   # Disable auto-start servers
   hive config set integration.auto_start_servers false
   ```

## API & Network Problems

### Connection Timeouts

**Symptoms:** "Connection timeout", "Request failed"

**Diagnosis:**
```bash
# Test network connectivity
hive test --network --verbose

# Check DNS resolution
nslookup openrouter.ai
nslookup api.anthropic.com

# Test direct connection
curl -I https://openrouter.ai/api/v1/models
```

**Solutions:**

1. **Adjust timeout settings:**
   ```bash
   hive config set providers.openrouter.timeout "60s"
   hive config set performance.request_timeout "45s"
   ```

2. **Proxy configuration:**
   ```bash
   # Set proxy environment variables
   export HTTPS_PROXY="http://proxy.company.com:8080"
   export HTTP_PROXY="http://proxy.company.com:8080"
   export NO_PROXY="localhost,127.0.0.1"
   ```

3. **Alternative endpoints:**
   ```bash
   # Try alternative endpoints
   hive config set providers.openrouter.base_url "https://api.openrouter.ai/api/v1"
   ```

### Rate Limiting

**Symptoms:** "Rate limit exceeded", "Too many requests"

**Solutions:**

1. **Check rate limits:**
   ```bash
   hive analytics usage --rate-limits
   hive cost status --remaining-quota
   ```

2. **Reduce request frequency:**
   ```bash
   hive config set consensus.retry_delay "5s"
   hive config set performance.request_rate_limit "10/minute"
   ```

3. **Use different models:**
   ```bash
   # Switch to models with higher rate limits
   hive config set models.generator "openai/gpt-4-turbo"
   ```

### SSL/TLS Certificate Issues

**Symptoms:** "SSL certificate verify failed", "Certificate error"

**Solutions:**

1. **Update certificates:**
   ```bash
   # Ubuntu/Debian
   sudo apt update && sudo apt install ca-certificates
   
   # CentOS/RHEL
   sudo yum update ca-certificates
   
   # macOS
   brew install ca-certificates
   ```

2. **Set certificate bundle:**
   ```bash
   export SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
   export SSL_CERT_DIR=/etc/ssl/certs
   ```

3. **Temporary workaround (not recommended for production):**
   ```bash
   hive config set security.verify_ssl false
   ```

## Memory & Storage Issues

### Disk Space Problems

**Symptoms:** "No space left on device", cache errors

**Diagnosis:**
```bash
# Check disk usage
df -h ~/.hive
du -sh ~/.hive/*

# Check cache size
hive cache stats --detailed
```

**Solutions:**

1. **Clean up caches:**
   ```bash
   # Remove old cache entries
   hive cache clean --older-than 7d
   
   # Clear specific cache types
   hive cache clear --type conversations
   hive cache clear --type analysis
   ```

2. **Adjust cache settings:**
   ```bash
   # Reduce cache size
   hive config set performance.cache_size "500MB"
   
   # Enable automatic cleanup
   hive config set cache.auto_cleanup true
   hive config set cache.max_age "30d"
   ```

3. **Move cache location:**
   ```bash
   # Move to different disk
   hive config set cache.location "/path/to/larger/disk/.hive-cache"
   ```

### Database Corruption

**Symptoms:** "Database is locked", "Corrupted database file"

**Solutions:**

1. **Database repair:**
   ```bash
   # Stop all hive processes
   pkill -f hive
   
   # Repair database
   hive database repair
   
   # Rebuild if repair fails
   hive database rebuild --from-backup
   ```

2. **Restore from backup:**
   ```bash
   # List available backups
   hive backup list
   
   # Restore from specific backup
   hive backup restore --date 2024-01-01
   ```

## IDE Integration Issues

### VS Code Extension Problems

**Symptoms:** Extension not working, commands not available

**Solutions:**

1. **Check extension status:**
   ```bash
   # Verify Hive server is running
   hive status --services
   
   # Check MCP connection
   hive mcp status --port 7778
   ```

2. **Restart services:**
   ```bash
   # Restart MCP server
   hive mcp restart
   
   # Reload VS Code window
   # Cmd+Shift+P → "Developer: Reload Window"
   ```

3. **Check configuration:**
   ```json
   // settings.json
   {
     "hive.server.enabled": true,
     "hive.server.port": 7778,
     "hive.api.key": "your-api-key"
   }
   ```

### Vim/Neovim Issues

**Symptoms:** LSP not working, no completions

**Solutions:**

1. **Check LSP server:**
   ```bash
   # Start LSP server manually
   hive lsp --port 7777 --verbose
   ```

2. **Verify Neovim configuration:**
   ```lua
   -- Check if Hive LSP is configured
   :LspInfo
   
   -- Restart LSP
   :LspRestart
   ```

3. **Debug LSP communication:**
   ```lua
   -- Enable LSP logging
   vim.lsp.set_log_level("debug")
   
   -- Check logs
   :echo vim.lsp.get_log_path()
   ```

## Security & Permissions

### Trust Dialog Issues

**Symptoms:** Constant permission prompts, access denied errors

**Solutions:**

1. **Adjust trust policy:**
   ```bash
   # Always trust (less secure)
   hive config set security.trust_policy always
   
   # Never prompt (most secure)
   hive config set security.trust_policy never
   
   # Prompt once per directory (balanced)
   hive config set security.trust_policy prompt
   ```

2. **Manage trusted directories:**
   ```bash
   # Add trusted directory
   hive trust add /path/to/project

   # List trusted directories
   hive trust list
   
   # Remove trusted directory
   hive trust remove /path/to/project
   ```

### File Access Issues

**Symptoms:** "Permission denied" when analyzing files

**Solutions:**

1. **Check file permissions:**
   ```bash
   ls -la /path/to/problematic/file
   chmod 644 /path/to/problematic/file
   ```

2. **Run with appropriate permissions:**
   ```bash
   # For system files (be careful)
   sudo hive analyze /etc/config
   
   # For user files
   hive analyze ~/project --trust-directory
   ```

### Sandbox Mode Issues

**Symptoms:** Commands failing in sandbox mode

**Solutions:**

1. **Disable sandbox temporarily:**
   ```bash
   hive config set security.sandbox_mode false
   hive ask "question" --no-sandbox
   ```

2. **Configure sandbox allowlist:**
   ```bash
   hive config set security.sandbox_allowed_commands '["rustfmt", "clippy", "git"]'
   ```

## Advanced Diagnostics

### Debug Mode

**Enable comprehensive debugging:**
```bash
# Set debug log level
hive config set logging.level debug

# Enable trace mode
hive --trace ask "debug question"

# Generate debug report
hive debug report --include-config --include-logs
```

### Performance Profiling

```bash
# Profile specific operations
hive profile consensus --question "test" --iterations 10

# Memory profiling
hive profile memory --duration 60s

# I/O profiling
hive profile io --operation analyze --path .
```

### Network Diagnostics

```bash
# Comprehensive network test
hive test --network --all-providers --verbose

# DNS resolution test
hive test --dns

# Firewall test
hive test --firewall --ports 80,443,7777,7778
```

### Database Diagnostics

```bash
# Database integrity check
hive database check --integrity

# Database statistics
hive database stats --detailed

# Database query performance
hive database profile --slow-queries
```

## Getting Help

### Self-Service Resources

1. **Built-in help:**
   ```bash
   hive help
   hive help ask
   hive help config
   ```

2. **Documentation:**
   - [USER_GUIDE.md](USER_GUIDE.md) - Complete usage guide
   - [API_REFERENCE.md](API_REFERENCE.md) - API documentation
   - [INSTALLATION.md](INSTALLATION.md) - Installation guide

3. **Community resources:**
   - GitHub Issues: https://github.com/hivetechs/hive-consensus/issues
   - Discord Community: https://discord.gg/hivetechs
   - Documentation: https://docs.hivetechs.com

### Collecting Information for Support

When reporting issues, include:

```bash
# Generate support bundle
hive debug support-bundle --include-logs --redact-secrets

# This creates a file with:
# - System information
# - Configuration (with secrets redacted)
# - Recent logs
# - Performance metrics
# - Network diagnostics
```

**The bundle includes:**
- Operating system and version
- Hive version and build info
- Configuration (with API keys redacted)
- Recent error logs
- Performance metrics
- Network connectivity test results

### Common Error Codes

| Code | Meaning | Solution |
|------|---------|----------|
| E001 | Invalid API key | Check API key format and permissions |
| E002 | Network timeout | Check network connectivity and timeouts |
| E003 | Rate limit exceeded | Wait or upgrade API plan |
| E004 | Insufficient permissions | Check file/directory permissions |
| E005 | Configuration error | Validate and fix configuration |
| E006 | Database error | Repair or rebuild database |
| E007 | Cache corruption | Clear and rebuild cache |
| E008 | Memory limit exceeded | Reduce memory usage or increase limits |

### Enterprise Support

For enterprise customers:

- **Priority Support**: enterprise@hivetechs.com
- **SLA**: 4-hour response for critical issues
- **Dedicated Slack Channel**: Available with Enterprise plan
- **Phone Support**: Available during business hours
- **Custom Training**: On-site or remote training available

### Contributing to Documentation

Found an issue not covered here? Help improve this guide:

1. **Report missing issues:**
   - Open GitHub issue with "documentation" label
   - Include problem description and solution if known

2. **Contribute fixes:**
   - Fork the repository
   - Add your troubleshooting solution
   - Submit a pull request

---

**Still need help?** Join our Discord community or open a GitHub issue. Our team and community are here to help you succeed with HiveTechs Consensus!