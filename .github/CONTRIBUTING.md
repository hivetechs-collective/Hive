# Contributing to HiveTechs Consensus

Thank you for your interest in contributing to HiveTechs Consensus! This guide will help you get started with contributing code, documentation, or other improvements.

## 🚀 Quick Start

1. **Fork** the repository
2. **Clone** your fork: `git clone https://github.com/your-username/hive-consensus.git`
3. **Create** a feature branch: `git checkout -b feature/your-feature-name`
4. **Make** your changes
5. **Test** your changes thoroughly
6. **Submit** a pull request

## 📋 Ways to Contribute

### Code Contributions
- **Bug fixes**: Fix reported issues
- **Performance improvements**: Optimize existing functionality
- **New features**: Implement planned features from our roadmap
- **Test coverage**: Add unit tests and integration tests
- **Documentation**: Improve code documentation and comments

### Non-Code Contributions
- **Documentation**: Improve user guides, API docs, or tutorials
- **Bug reports**: Report issues with detailed reproduction steps
- **Feature requests**: Suggest new functionality
- **Community support**: Help other users in Discord and GitHub
- **Translation**: Help translate documentation (future)

## 🛠️ Development Setup

### Prerequisites

- **Rust 1.70+**: Install via [rustup](https://rustup.rs/)
- **Git**: For version control
- **Make**: For build automation (optional)

### Local Development

```bash
# Clone the repository
git clone https://github.com/hivetechs/hive-consensus.git
cd hive-consensus

# Install development dependencies
cargo install cargo-watch cargo-criterion cargo-tarpaulin

# Build the project
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- ask "test question"

# Watch mode for development
cargo watch -x "run -- --version"
```

### Environment Setup

```bash
# Set up configuration for development
cp templates/default_config.toml ~/.hive/config.toml

# Set development API keys (create .env file)
echo "OPENROUTER_API_KEY=your-dev-key" > .env
echo "RUST_LOG=debug" >> .env

# Source environment
source .env
```

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test consensus::

# Run integration tests
cargo test --test integration

# Run with coverage
cargo tarpaulin --out html

# Run benchmarks
cargo bench
```

### Writing Tests

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consensus_pipeline() {
        // Test implementation
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_async_functionality() {
        // Async test implementation
    }
}
```

#### Integration Tests
```rust
// tests/integration_test.rs
use hive_consensus::*;

#[test]
fn test_cli_integration() {
    // Integration test implementation
}
```

### Test Guidelines

- **Unit tests**: Test individual functions and modules
- **Integration tests**: Test component interactions
- **End-to-end tests**: Test complete workflows
- **Performance tests**: Benchmark critical paths
- **Property-based tests**: Use quickcheck for edge cases

## 📝 Code Style

### Rust Guidelines

We follow standard Rust conventions with some additions:

```rust
// Use descriptive names
fn analyze_code_quality(file_path: &Path) -> Result<QualityScore> {
    // Implementation
}

// Document public APIs
/// Analyzes code quality for the given file.
/// 
/// # Arguments
/// * `file_path` - Path to the file to analyze
/// 
/// # Returns
/// Quality score from 0-100, or error if analysis fails
pub fn analyze_file(file_path: &Path) -> Result<u8> {
    // Implementation
}

// Use type aliases for clarity
type ConsensusResult = Result<String, ConsensusError>;

// Prefer explicit error handling
match perform_analysis() {
    Ok(result) => handle_success(result),
    Err(e) => handle_error(e),
}
```

### Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Lint code
cargo clippy

# Strict linting
cargo clippy -- -D warnings
```

### Performance Considerations

- **Avoid allocations** in hot paths
- **Use `&str` over `String`** when possible
- **Prefer iterators** over collecting to vectors
- **Profile before optimizing** with `cargo bench`
- **Consider async/await** for I/O operations

## 🏗️ Architecture

### Project Structure

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library entry point
├── cli/                 # CLI interface and commands
├── consensus/           # 4-stage consensus pipeline
├── core/                # Core functionality
├── providers/           # AI provider integrations
├── analysis/            # Code analysis engine
├── transformation/      # Code transformation
├── memory/              # Memory and persistence
├── tui/                 # Terminal UI
├── integration/         # IDE integrations
└── security/            # Security and permissions
```

### Design Principles

1. **Performance First**: Optimize for speed and memory efficiency
2. **Safety**: Use Rust's type system to prevent errors
3. **Modularity**: Keep components loosely coupled
4. **Testability**: Design for easy testing
5. **User Experience**: Prioritize developer experience
6. **Backward Compatibility**: Maintain API stability

## 📋 Pull Request Process

### Before Submitting

1. **Create an issue** for discussion (for large changes)
2. **Write tests** for your changes
3. **Update documentation** if needed
4. **Run the full test suite**
5. **Check code formatting** with `cargo fmt`
6. **Fix all clippy warnings**

### Pull Request Template

```markdown
## Description
Brief description of the changes and motivation.

## Type of Change
- [ ] Bug fix (non-breaking change fixing an issue)
- [ ] New feature (non-breaking change adding functionality)
- [ ] Breaking change (fix/feature causing existing functionality to change)
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Refactoring (no functional changes)

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing completed
- [ ] Performance benchmarks run (if applicable)

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No new warnings introduced
- [ ] All tests pass
```

### Review Process

1. **Automated checks**: CI runs tests and linting
2. **Code review**: Maintainers review code quality and design
3. **Testing**: Changes are tested on multiple platforms
4. **Documentation review**: Documentation changes are reviewed
5. **Approval**: At least one maintainer approves
6. **Merge**: Changes are merged to main branch

## 🐛 Bug Reports

### Before Reporting

1. **Search existing issues** to avoid duplicates
2. **Try the latest version** to see if it's already fixed
3. **Check the troubleshooting guide**
4. **Gather reproduction steps**

### Good Bug Report

```markdown
**Bug Description**
Clear description of what's wrong.

**To Reproduce**
1. Step one
2. Step two
3. See error

**Expected Behavior**
What should have happened.

**Environment**
- OS: macOS 13.0
- Hive version: 2.0.0
- Rust version: 1.70.0

**Additional Context**
Any other relevant information.
```

## 💡 Feature Requests

### Before Requesting

1. **Check existing issues** and roadmap
2. **Discuss in Discord** or GitHub Discussions
3. **Consider implementation complexity**
4. **Think about backward compatibility**

### Good Feature Request

```markdown
**Problem Statement**
What problem does this solve?

**Proposed Solution**
How should this work?

**Alternatives Considered**
What other approaches did you consider?

**Additional Context**
Use cases, mockups, etc.
```

## 📚 Documentation

### Writing Guidelines

- **Clear and concise**: Write for your audience
- **Include examples**: Show don't just tell
- **Keep it current**: Update docs with code changes
- **Test instructions**: Verify all steps work
- **Use consistent formatting**: Follow existing patterns

### Documentation Types

- **User guides**: How to use features
- **API documentation**: Function and method docs
- **Tutorials**: Step-by-step learning
- **Reference**: Complete option lists
- **Architecture docs**: Design decisions

## 🌟 Recognition

Contributors are recognized in several ways:

- **Contributors list**: Listed in README.md
- **Release notes**: Mentioned in changelogs
- **Discord recognition**: Special roles and shoutouts
- **Contributor statistics**: GitHub insights
- **Annual recognition**: Special acknowledgments

## 📞 Getting Help

### Community Support

- **Discord**: https://discord.gg/hivetechs
- **GitHub Discussions**: For design discussions
- **Stack Overflow**: Tag questions with `hive-consensus`

### Maintainer Contact

- **General questions**: Open a discussion
- **Security issues**: security@hivetechs.com
- **Urgent maintainer contact**: @maintainers in Discord

## 📜 Code of Conduct

We are committed to providing a welcoming and inclusive experience for everyone. Please read our [Code of Conduct](CODE_OF_CONDUCT.md).

### Quick Summary

- **Be respectful**: Treat everyone with respect
- **Be inclusive**: Welcome newcomers and diverse perspectives  
- **Be constructive**: Focus on helping and improving
- **Be professional**: Maintain professional communication
- **Be patient**: Help others learn and grow

## 🎯 Contribution Focus Areas

### High Priority

- **Performance optimizations**: Critical path improvements
- **Test coverage**: Increase to >90%
- **Documentation**: User guides and API docs
- **Bug fixes**: Issues marked as `good first issue`

### Medium Priority

- **New features**: From approved roadmap
- **IDE integrations**: VS Code, Vim, etc.
- **Platform support**: Windows, Linux improvements
- **Accessibility**: TUI and CLI accessibility

### Future

- **Internationalization**: Multi-language support
- **Plugin system**: Extensibility framework
- **Advanced analytics**: ML-powered insights

## 🚀 Next Steps

1. **Join our Discord**: https://discord.gg/hivetechs
2. **Read the codebase**: Start with `src/main.rs`
3. **Pick an issue**: Look for `good first issue` labels
4. **Ask questions**: We're here to help!

Thank you for contributing to HiveTechs Consensus! 🐝✨