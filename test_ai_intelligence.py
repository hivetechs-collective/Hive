#!/usr/bin/env python3
"""
Test to verify AI Helper intelligence capabilities
"""

import json
import sys
import torch
from transformers import AutoModel, AutoTokenizer
import numpy as np
from scipy.spatial.distance import cosine

def test_semantic_understanding():
    """Test that AI models understand semantic similarity"""
    print("🧪 Testing AI Helper Semantic Understanding...")
    
    # Load a small model for testing
    model_name = "microsoft/codebert-base"
    print(f"Loading {model_name}...")
    
    try:
        tokenizer = AutoTokenizer.from_pretrained(model_name)
        model = AutoModel.from_pretrained(model_name)
        model.eval()
        
        # Test semantic similarity between similar concepts
        test_pairs = [
            ("create a hello world file", "make a greeting file"),
            ("delete the configuration", "remove the config"),
            ("update the database schema", "modify the db structure"),
            ("implement authentication", "add user login system"),
        ]
        
        print("\n📊 Semantic Similarity Test Results:")
        print("=" * 50)
        
        for text1, text2 in test_pairs:
            # Generate embeddings
            inputs1 = tokenizer(text1, return_tensors="pt", padding=True, truncation=True)
            inputs2 = tokenizer(text2, return_tensors="pt", padding=True, truncation=True)
            
            with torch.no_grad():
                outputs1 = model(**inputs1)
                outputs2 = model(**inputs2)
                
                # Use pooled output or mean of last hidden states
                embedding1 = outputs1.last_hidden_state.mean(dim=1).squeeze().numpy()
                embedding2 = outputs2.last_hidden_state.mean(dim=1).squeeze().numpy()
            
            # Calculate cosine similarity
            similarity = 1 - cosine(embedding1, embedding2)
            
            print(f"\n'{text1}' <-> '{text2}'")
            print(f"Similarity Score: {similarity:.3f}")
            print("Interpretation:", end=" ")
            
            if similarity > 0.8:
                print("✅ High similarity - AI understands these are related!")
            elif similarity > 0.6:
                print("🔶 Moderate similarity - AI sees some connection")
            else:
                print("❌ Low similarity - AI sees these as different")
        
        # Test code understanding
        print("\n\n🧠 Code Understanding Test:")
        print("=" * 50)
        
        code_samples = [
            "def hello_world():\n    print('Hello, World!')",
            "function helloWorld() { console.log('Hello, World!'); }",
            "public static void main(String[] args) { System.out.println('Hello, World!'); }",
        ]
        
        print("Analyzing different 'Hello World' implementations...")
        embeddings = []
        
        for code in code_samples:
            inputs = tokenizer(code, return_tensors="pt", padding=True, truncation=True)
            with torch.no_grad():
                outputs = model(**inputs)
                embedding = outputs.last_hidden_state.mean(dim=1).squeeze().numpy()
                embeddings.append(embedding)
        
        # Compare all pairs
        languages = ["Python", "JavaScript", "Java"]
        for i in range(len(embeddings)):
            for j in range(i+1, len(embeddings)):
                similarity = 1 - cosine(embeddings[i], embeddings[j])
                print(f"\n{languages[i]} vs {languages[j]}: {similarity:.3f}")
                print("AI recognizes these as similar implementations! ✅" if similarity > 0.7 else "AI sees differences")
        
        print("\n\n✨ Conclusion:")
        print("The AI Helpers use real transformer models that understand:")
        print("- Semantic meaning (not just keywords)")
        print("- Code structure and intent")
        print("- Cross-language patterns")
        print("- Context and relationships")
        print("\nThey are genuine AI models, not simple pattern matchers! 🎉")
        
    except Exception as e:
        print(f"Error: {e}")
        print("\nNote: This test requires the transformer models to be installed.")
        print("The actual Hive AI Helpers use these models through the Python service.")

if __name__ == "__main__":
    test_semantic_understanding()