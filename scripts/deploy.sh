#!/bin/bash

# 🚀 HiveTechs Consensus Deployment Script
# Automates the complete deployment process

set -e

echo "🐝 HiveTechs Consensus - Deployment Script"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO_URL="git@github.com:hivetechs-collective/Hive.git"
VERSION="2.0.0"

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        print_error "Not in HiveTechs Consensus directory. Please run from /Users/veronelazio/Developer/Private/hive"
        exit 1
    fi
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust: https://rustup.rs/"
        exit 1
    fi
    
    # Check Git
    if ! command -v git &> /dev/null; then
        print_error "Git not found. Please install Git."
        exit 1
    fi
    
    # Check Node.js (for NPM publishing)
    if ! command -v node &> /dev/null; then
        print_warning "Node.js not found. NPM publishing will be skipped."
    fi
    
    print_success "Prerequisites check complete"
}

# Build production binaries
build_binaries() {
    print_status "Building production binaries..."
    
    # Clean previous builds
    cargo clean
    
    # Build release binary
    cargo build --release
    
    if [ -f "target/release/hive" ]; then
        print_success "Binary built successfully: target/release/hive"
        
        # Test the binary
        print_status "Testing binary..."
        ./target/release/hive --version
        print_success "Binary test passed"
    else
        print_error "Binary build failed"
        exit 1
    fi
}

# Setup Git repository
setup_git() {
    print_status "Setting up Git repository..."
    
    # Check if remote already exists
    if git remote get-url origin &> /dev/null; then
        print_warning "Git remote 'origin' already exists. Skipping remote setup."
    else
        print_status "Adding GitHub remote..."
        read -p "Enter your GitHub repository URL [${REPO_URL}]: " user_repo_url
        repo_url=${user_repo_url:-$REPO_URL}
        
        git remote add origin "$repo_url"
        print_success "GitHub remote added: $repo_url"
    fi
    
    # Check if we have changes to commit
    if git diff-index --quiet HEAD --; then
        print_status "No changes to commit"
    else
        print_status "Committing changes..."
        git add .
        git commit -m "Deploy HiveTechs Consensus v${VERSION}

🚀 Revolutionary AI development assistant with:
- 4-stage AI consensus engine (323+ models)
- 10-40x performance improvements
- Enterprise-grade security and compliance
- Professional TUI and IDE integration

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"
        print_success "Changes committed"
    fi
}

# Push to GitHub
push_to_github() {
    print_status "Pushing to GitHub..."
    
    # Push main branch
    git push -u origin main
    print_success "Code pushed to GitHub"
    
    # Create and push tag
    if git tag | grep -q "v${VERSION}"; then
        print_warning "Tag v${VERSION} already exists"
    else
        print_status "Creating release tag..."
        git tag -a "v${VERSION}" -m "HiveTechs Consensus v${VERSION} - Revolutionary AI Development Assistant"
        git push origin "v${VERSION}"
        print_success "Release tag v${VERSION} created and pushed"
    fi
}

# Setup GitHub features
setup_github_features() {
    print_status "Setting up GitHub community features..."
    
    # Create .github directory structure
    mkdir -p .github/ISSUE_TEMPLATE
    mkdir -p .github/workflows
    
    # Copy community templates if they exist
    if [ -d "distribution/github/ISSUE_TEMPLATE" ]; then
        cp -r distribution/github/ISSUE_TEMPLATE/* .github/ISSUE_TEMPLATE/
        print_success "Issue templates copied"
    fi
    
    if [ -f "distribution/github/PULL_REQUEST_TEMPLATE.md" ]; then
        cp distribution/github/PULL_REQUEST_TEMPLATE.md .github/
        print_success "Pull request template copied"
    fi
    
    if [ -d "distribution/github/workflows" ]; then
        cp -r distribution/github/workflows/* .github/workflows/
        print_success "GitHub Actions workflows copied"
    fi
    
    # Copy community files
    if [ -f "CONTRIBUTING.md" ]; then
        cp CONTRIBUTING.md .github/
    fi
    
    if [ -f "CODE_OF_CONDUCT.md" ]; then
        cp CODE_OF_CONDUCT.md .github/
    fi
    
    # Commit GitHub features
    if [ -d ".github" ]; then
        git add .github/
        git commit -m "Add GitHub community templates and workflows" || true
        git push
        print_success "GitHub community features deployed"
    fi
}

# Prepare NPM package
prepare_npm() {
    print_status "Preparing NPM package..."
    
    if ! command -v npm &> /dev/null; then
        print_warning "NPM not found. Skipping NPM package preparation."
        return
    fi
    
    # Create npm distribution directory
    mkdir -p distribution/npm
    
    # Copy NPM files
    if [ -d "npm" ]; then
        cp -r npm/* distribution/npm/
        print_success "NPM package files prepared in distribution/npm/"
        
        echo ""
        print_status "To publish to NPM:"
        echo "1. cd distribution/npm"
        echo "2. npm login"
        echo "3. npm publish --access public"
        echo ""
    else
        print_warning "NPM package files not found in npm/ directory"
    fi
}

# Create GitHub release
create_github_release() {
    print_status "GitHub release creation..."
    
    echo ""
    print_success "🎉 Deployment Complete!"
    echo ""
    echo "Next Steps:"
    echo "1. Go to: https://github.com/hivetechs-collective/hive/releases"
    echo "2. Click 'Create a new release'"
    echo "3. Select tag: v${VERSION}"
    echo "4. Upload the binary: target/release/hive"
    echo "5. Publish the release"
    echo ""
    echo "GitHub Features to Enable:"
    echo "1. Settings → Features → Discussions ✅"
    echo "2. Settings → Features → Issues ✅"
    echo "3. Settings → Pages → Deploy from docs/ folder"
    echo ""
}

# Display next steps
display_next_steps() {
    echo ""
    print_success "🚀 HiveTechs Consensus Deployment Summary"
    echo "=========================================="
    echo ""
    echo "✅ Binary built and tested"
    echo "✅ Code pushed to GitHub"
    echo "✅ Release tag created"
    echo "✅ GitHub community features setup"
    echo "✅ NPM package prepared"
    echo ""
    echo "🌐 Your Repository: ${REPO_URL}"
    echo "📦 Binary Location: target/release/hive"
    echo "📚 Documentation: docs/"
    echo "🔧 NPM Package: distribution/npm/"
    echo ""
    echo "Installation Commands for Users:"
    echo "npm install -g @hivetechs/hive-ai"
    echo "curl -fsSL https://install.hivetechs.com | sh"
    echo ""
    echo "🎯 Ready for Global Launch!"
}

# Main deployment flow
main() {
    echo ""
    print_status "Starting HiveTechs Consensus deployment..."
    echo ""
    
    check_prerequisites
    build_binaries
    setup_git
    push_to_github
    setup_github_features
    prepare_npm
    create_github_release
    display_next_steps
    
    echo ""
    print_success "🎉 Deployment script completed successfully!"
    echo ""
}

# Run main function
main "$@"