#!/bin/bash

# Clone AI Helper Repositories Script
# This script clones all required open-source AI models and frameworks
# for the Hive AI Helper Ecosystem

echo "🚀 Starting AI Helper Repository Cloning..."

# Create directory for external dependencies
mkdir -p external/ai_helpers
cd external/ai_helpers

# Clone Microsoft CodeBERT Family
echo "📚 Cloning Microsoft CodeBERT..."
git clone https://github.com/microsoft/CodeBERT.git
echo "✅ CodeBERT cloned"

# Clone Salesforce CodeT5
echo "📚 Cloning Salesforce CodeT5..."
git clone https://github.com/salesforce/CodeT5.git
echo "✅ CodeT5 cloned"

# Clone LangChain
echo "📚 Cloning LangChain..."
git clone https://github.com/langchain-ai/langchain.git
echo "✅ LangChain cloned"

# Clone Chroma
echo "📚 Cloning Chroma..."
git clone https://github.com/chroma-core/chroma.git
echo "✅ Chroma cloned"

# Clone LlamaIndex (for additional RAG capabilities)
echo "📚 Cloning LlamaIndex..."
git clone https://github.com/run-llama/llama_index.git
echo "✅ LlamaIndex cloned"

# Clone Continue.dev (for codebase indexing patterns)
echo "📚 Cloning Continue.dev..."
git clone https://github.com/continuedev/continue.git
echo "✅ Continue.dev cloned"

# Clone Aider (for repository mapping patterns)
echo "📚 Cloning Aider..."
git clone https://github.com/Aider-AI/aider.git
echo "✅ Aider cloned"

# Return to main directory
cd ../..

echo "🎉 All AI Helper repositories cloned successfully!"
echo "📁 Repositories are in: external/ai_helpers/"
echo ""
echo "Next steps:"
echo "1. Review each repository's core functionality"
echo "2. Extract relevant modules for integration"
echo "3. Create Rust bindings where needed"
echo "4. Implement the AI Helper Ecosystem"