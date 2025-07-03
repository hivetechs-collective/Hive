# HiveTechs Consensus Security Documentation

> **This document will be hosted at**: `https://docs.hivetechs.com/security/trust`

## Understanding HiveTechs Consensus Security

HiveTechs Consensus is designed with security at its core. When you open a new repository or folder, Consensus asks for your permission before accessing files. This protects you from potentially malicious code while ensuring Consensus can help you work effectively.

## Why Does This Matter?

### The Risks

When HiveTechs Consensus analyzes code, it:
- **Reads source files** to understand your codebase
- **Executes analysis tools** to gather insights
- **Applies code transformations** based on AI consensus
- **Accesses repository metadata** like Git history

While these capabilities make Consensus incredibly powerful, they also mean that malicious code could potentially:
- ❌ Access sensitive files
- ❌ Execute harmful commands
- ❌ Modify critical system files
- ❌ Steal credentials or API keys

### Our Protection

HiveTechs Consensus uses a **trust-based security model** to protect you:

✅ **Explicit Permission**: You must explicitly trust each new directory  
✅ **Persistent Trust**: Trusted directories stay trusted (until you revoke)  
✅ **Sandboxed Execution**: Untrusted code runs in a restricted environment  
✅ **Network Filtering**: Only approved domains can be accessed  
✅ **Audit Logging**: All security events are logged for review  

## The Trust Dialog

When you navigate to a new repository, you'll see this security prompt:

```
┌──────────────────────────────────────────────────────┐
│     Do you trust the files in this folder?          │
│                                                      │
│ /Users/yourname/new-project                         │
│                                                      │
│ HiveTechs Consensus may read files in this folder.  │
│ Reading untrusted files may lead HiveTechs          │
│ Consensus to behave in unexpected ways.             │
│                                                      │
│ With your permission HiveTechs Consensus may        │
│ execute code transformations and analyze files      │
│ in this folder. Processing untrusted code is        │
│ potentially unsafe.                                  │
│                                                      │
│ https://docs.hivetechs.com/security/trust           │
│                                                      │
│ ❯ 1. Yes, proceed                                    │
│   2. No, exit                                        │
└──────────────────────────────────────────────────────┘
```

### When to Choose "Yes, proceed"

✅ **Your own repositories** - Code you've written or maintain  
✅ **Trusted team repositories** - Projects from colleagues you trust  
✅ **Well-known open source projects** - Popular libraries with good reputations  
✅ **Repositories you've reviewed** - Code you've examined for safety  

### When to Choose "No, exit"

❌ **Unknown repositories** - Code from untrusted sources  
❌ **Suspicious downloads** - Files from questionable websites  
❌ **Unreviewed code** - Projects you haven't examined  
❌ **Temporary downloads** - One-time code samples  

## Managing Trusted Directories

### Viewing Trusted Directories

```bash
hive trust list
```

This shows all directories you've previously trusted:

```
🔒 Trusted Directories:
  ✓ /Users/yourname/my-project
  ✓ /Users/yourname/work/api-service
  ✓ /Users/yourname/open-source/rust-project
```

### Adding Trust Manually

```bash
hive trust add /path/to/directory
```

### Removing Trust

```bash
hive trust remove /path/to/directory
```

### Clearing All Trust

```bash
hive trust clear --confirm
```

⚠️ **Warning**: This removes trust from ALL directories. You'll need to re-approve each one.

## Security Features in Detail

### 1. File Access Protection

HiveTechs Consensus only reads files in trusted directories. Additionally:

- **Size Limits**: Files larger than 10MB require explicit permission
- **Path Validation**: Prevents access outside trusted directories via symlinks
- **Permission Checks**: Respects file system permissions
- **Binary File Detection**: Warns before reading binary files

### 2. Command Execution Control

When Consensus needs to run commands (like `cargo build` or `git status`):

- **Whitelist-based**: Only pre-approved commands are allowed
- **Argument Validation**: Command arguments are sanitized
- **Environment Isolation**: Commands run with minimal environment variables
- **Working Directory Restriction**: Commands execute only in trusted directories

### 3. Network Request Filtering

Consensus connects to external services for AI processing:

- **Approved Domains**: Only OpenRouter API and Cloudflare Workers are allowed by default
- **HTTPS Only**: All network traffic uses encrypted connections
- **Request Validation**: API requests are validated before sending
- **Rate Limiting**: Prevents excessive API usage

### 4. Sandbox Mode

For maximum security when working with untrusted code:

```bash
hive --sandbox analyze suspicious-project/
```

In sandbox mode:
- File writes are blocked
- Network access is restricted
- Command execution is disabled
- Analysis results are read-only

## Best Practices

### For Individual Users

1. **🔍 Review Before Trusting**
   - Examine the code structure before granting trust
   - Look for suspicious files or patterns
   - Check the repository's reputation and contributors

2. **🔄 Regular Audits**
   - Periodically review your trusted directories
   - Remove trust from projects you no longer use
   - Update HiveTechs Consensus regularly

3. **🛡️ Use Sandbox Mode**
   - Always use sandbox mode for unfamiliar code
   - Test suspicious repositories in isolated environments
   - Never trust repositories from unknown sources

4. **📋 Monitor Audit Logs**
   ```bash
   hive security audit-log
   ```
   - Review security events regularly
   - Investigate any unexpected file access
   - Report suspicious activity to HiveTechs

### For Teams and Organizations

1. **📜 Establish Trust Policies**
   - Define criteria for trusting repositories
   - Require code review before adding team repositories
   - Maintain a list of approved external libraries

2. **👥 Shared Trust Management**
   ```bash
   hive trust sync --team team-config.json
   ```
   - Share trusted repository lists across team members
   - Centrally manage organizational trust policies
   - Automate trust for company repositories

3. **🔐 Enhanced Security Controls**
   ```toml
   # ~/.hive/config.toml
   [security]
   require_team_approval = true
   max_file_size = 5242880  # 5MB limit
   sandbox_mode_default = true
   ```

4. **📊 Security Monitoring**
   - Enable comprehensive audit logging
   - Monitor for unusual file access patterns
   - Set up alerts for security violations

## Configuration Options

### Security Settings

Edit `~/.hive/config.toml`:

```toml
[security]
# Trust and permission settings
trust_prompts_enabled = true      # Never disable in production
sandbox_mode_default = false      # Use sandbox by default
require_explicit_trust = true     # Always prompt for new directories

# File access controls
max_file_size = 10485760          # 10MB file size limit
allow_binary_files = false        # Block binary file analysis
symlink_protection = true         # Prevent symlink escape attacks

# Network security
network_requests_enabled = true   # Allow API calls
allowed_domains = [               # Additional trusted domains
  "github.com",
  "api.example.com"
]
blocked_domains = [               # Explicitly blocked domains
  "malicious-site.com"
]

# Command execution
command_execution_enabled = true  # Allow running commands
allowed_commands = [              # Whitelist of safe commands
  "cargo", "git", "npm", "yarn", 
  "python", "node", "go", "rustc"
]

# Audit and logging
security_audit_enabled = true     # Enable security logging
audit_log_retention_days = 90     # Keep logs for 90 days
log_file_access = true            # Log all file reads
log_network_requests = true       # Log API calls
```

### Environment Variables

For CI/CD and automated environments:

```bash
# Auto-trust the current directory (use with caution)
export HIVE_AUTO_TRUST_CWD=1

# Enable sandbox mode by default
export HIVE_SANDBOX_MODE=1

# Disable interactive prompts (for scripts)
export HIVE_NON_INTERACTIVE=1
```

## Incident Response

### If You Suspect a Security Issue

1. **🛑 Stop Immediately**
   - Exit HiveTechs Consensus
   - Don't run any more commands
   - Disconnect from the network if necessary

2. **🔍 Investigate**
   ```bash
   hive security audit-log --recent
   hive trust list
   ```
   - Check what files were accessed
   - Review recent trust decisions
   - Look for unexpected network activity

3. **🧹 Clean Up**
   ```bash
   hive trust clear --confirm
   hive security reset --confirm
   ```
   - Remove all trusted directories
   - Reset security settings to defaults
   - Change any exposed credentials

4. **📞 Report**
   - Contact HiveTechs Security: security@hivetechs.com
   - Provide audit logs and incident details
   - Follow up with your security team

### Security Vulnerabilities

If you discover a security vulnerability in HiveTechs Consensus:

- **🔒 Email**: security@hivetechs.com (encrypted communication preferred)
- **📋 Include**: Detailed reproduction steps, affected versions, impact assessment
- **⏰ Response**: We aim to respond within 24 hours
- **🛡️ Disclosure**: We follow responsible disclosure practices

## Frequently Asked Questions

### Q: Why do I need to trust directories?

A: HiveTechs Consensus is powerful - it can read, analyze, and modify code. The trust system ensures you're always in control of what Consensus can access, protecting you from malicious or unknown code.

### Q: Can I trust a directory permanently?

A: Yes! Once you trust a directory, it stays trusted until you manually remove it. This means you won't see the prompt again for that directory.

### Q: What happens if I don't trust a directory?

A: HiveTechs Consensus will exit and won't access any files in that directory. You can still use Consensus in other trusted directories.

### Q: Is it safe to trust open source projects?

A: Generally yes, especially well-known projects. However, always review the code first, check the project's reputation, and be cautious with recently created or little-known repositories.

### Q: Can I use Consensus without trusting directories?

A: You can use sandbox mode (`hive --sandbox`) for limited analysis without full trust. This provides read-only analysis with restricted capabilities.

### Q: What data does HiveTechs collect?

A: We only process your code through our AI consensus pipeline. No code is stored permanently, and all processing happens through secure, encrypted connections. See our [Privacy Policy](https://hivetechs.com/privacy) for complete details.

### Q: How often should I review trusted directories?

A: We recommend monthly reviews to remove directories you no longer use and ensure all trusted paths are still valid.

---

## Need Help?

- **📖 Documentation**: [docs.hivetechs.com](https://docs.hivetechs.com)
- **💬 Support**: support@hivetechs.com
- **🔒 Security**: security@hivetechs.com
- **🐛 Bug Reports**: [github.com/hivetechs/hive/issues](https://github.com/hivetechs/hive/issues)

Your security is our priority. HiveTechs Consensus is designed to be both powerful and safe - the trust system ensures you maintain complete control over your code and data.