#!/usr/bin/env python3
"""
AI Model Service for Hive
Handles embedding generation, text generation, and code analysis using various models.
"""

import json
import sys
import os
import argparse
import logging
import traceback
from typing import List, Dict, Any, Optional
import uuid
import numpy as np

# Suppress transformers warnings
os.environ['TRANSFORMERS_NO_ADVISORY_WARNINGS'] = 'true'
logging.getLogger("transformers").setLevel(logging.ERROR)

try:
    import torch
    from transformers import (
        AutoModel, 
        AutoTokenizer,
        AutoModelForSeq2SeqLM,
        T5ForConditionalGeneration,
        pipeline
    )
    from sentence_transformers import SentenceTransformer
    import chromadb
    from chromadb.config import Settings
except ImportError as e:
    print(json.dumps({
        "type": "error",
        "error": f"Missing required dependencies: {e}. Please install: transformers torch sentence-transformers chromadb",
        "request_id": "startup"
    }))
    sys.exit(1)

class ModelService:
    def __init__(self, model_cache_dir: str):
        self.model_cache_dir = os.path.expanduser(model_cache_dir)
        os.makedirs(self.model_cache_dir, exist_ok=True)
        
        # Model registry
        self.models = {}
        self.tokenizers = {}
        
        # Setup logging
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s',
            handlers=[logging.FileHandler(os.path.join(model_cache_dir, 'model_service.log'))]
        )
        self.logger = logging.getLogger(__name__)
        
        # Initialize models lazily
        self.model_loaders = {
            "microsoft/codebert-base": self._load_codebert,
            "Salesforce/codet5p-110m-embedding": self._load_codet5_embedding,
            "microsoft/graphcodebert-base": self._load_graphcodebert,
            "microsoft/unixcoder-base": self._load_unixcoder,
            "sentence-transformers/all-MiniLM-L6-v2": self._load_sentence_transformer,
        }
        
        # Chroma client for vector storage
        self.chroma_client = None
        
    def _load_codebert(self):
        """Load CodeBERT model for code understanding"""
        model_name = "microsoft/codebert-base"
        self.logger.info(f"Loading {model_name}...")
        
        try:
            tokenizer = AutoTokenizer.from_pretrained(
                model_name,
                cache_dir=self.model_cache_dir
            )
            model = AutoModel.from_pretrained(
                model_name,
                cache_dir=self.model_cache_dir
            )
            
            if torch.cuda.is_available():
                model = model.cuda()
                
            self.models[model_name] = model
            self.tokenizers[model_name] = tokenizer
            self.logger.info(f"Loaded {model_name}")
        except Exception as e:
            self.logger.error(f"Failed to load {model_name}: {e}")
            # Fall back to sentence transformer
            self._load_sentence_transformer()
            self.models[model_name] = self.models["sentence-transformers/all-MiniLM-L6-v2"]
        
    def _load_codet5_embedding(self):
        """Load CodeT5+ for embeddings"""
        model_name = "Salesforce/codet5p-110m-embedding"
        self.logger.info(f"Loading {model_name}...")
        
        tokenizer = AutoTokenizer.from_pretrained(
            model_name,
            cache_dir=self.model_cache_dir,
            trust_remote_code=True
        )
        model = AutoModel.from_pretrained(
            model_name,
            cache_dir=self.model_cache_dir,
            trust_remote_code=True
        )
        
        if torch.cuda.is_available():
            model = model.cuda()
            
        self.models[model_name] = model
        self.tokenizers[model_name] = tokenizer
        self.logger.info(f"Loaded {model_name}")
        
    def _load_graphcodebert(self):
        """Load GraphCodeBERT for code understanding with structure"""
        model_name = "microsoft/graphcodebert-base"
        self.logger.info(f"Loading {model_name}...")
        
        tokenizer = AutoTokenizer.from_pretrained(
            model_name,
            cache_dir=self.model_cache_dir
        )
        model = AutoModel.from_pretrained(
            model_name,
            cache_dir=self.model_cache_dir
        )
        
        if torch.cuda.is_available():
            model = model.cuda()
            
        self.models[model_name] = model
        self.tokenizers[model_name] = tokenizer
        self.logger.info(f"Loaded {model_name}")
        
    def _load_unixcoder(self):
        """Load UniXcoder for cross-language understanding"""
        model_name = "microsoft/unixcoder-base"
        self.logger.info(f"Loading {model_name}...")
        
        tokenizer = AutoTokenizer.from_pretrained(
            model_name,
            cache_dir=self.model_cache_dir
        )
        model = AutoModel.from_pretrained(
            model_name,
            cache_dir=self.model_cache_dir
        )
        
        if torch.cuda.is_available():
            model = model.cuda()
            
        self.models[model_name] = model
        self.tokenizers[model_name] = tokenizer
        self.logger.info(f"Loaded {model_name}")
        
    def _load_sentence_transformer(self):
        """Load sentence transformer for general embeddings"""
        model_name = "sentence-transformers/all-MiniLM-L6-v2"
        self.logger.info(f"Loading {model_name}...")
        
        model = SentenceTransformer(
            model_name,
            cache_folder=self.model_cache_dir
        )
        
        self.models[model_name] = model
        self.logger.info(f"Loaded {model_name}")
        
    def _ensure_model_loaded(self, model_name: str):
        """Ensure a model is loaded"""
        if model_name not in self.models:
            if model_name in self.model_loaders:
                self.model_loaders[model_name]()
            else:
                raise ValueError(f"Unknown model: {model_name}")
                
    def generate_embeddings(self, model_name: str, texts: List[str]) -> List[List[float]]:
        """Generate embeddings for given texts"""
        self._ensure_model_loaded(model_name)
        
        if model_name == "sentence-transformers/all-MiniLM-L6-v2":
            # Use sentence transformers directly
            model = self.models[model_name]
            embeddings = model.encode(texts, convert_to_tensor=True)
            # Move to CPU for numpy conversion (handles CUDA, MPS, etc.)
            if hasattr(embeddings, 'cpu'):
                embeddings = embeddings.cpu()
            return embeddings.numpy().tolist()
        else:
            # Use transformers models
            model = self.models[model_name]
            tokenizer = self.tokenizers[model_name]
            
            embeddings = []
            for text in texts:
                inputs = tokenizer(
                    text,
                    return_tensors="pt",
                    max_length=512,
                    truncation=True,
                    padding=True
                )
                
                if torch.cuda.is_available():
                    inputs = {k: v.cuda() for k, v in inputs.items()}
                
                with torch.no_grad():
                    outputs = model(**inputs)
                    # Use pooled output or mean of last hidden states
                    if hasattr(outputs, 'pooler_output') and outputs.pooler_output is not None:
                        embedding = outputs.pooler_output
                    else:
                        embedding = outputs.last_hidden_state.mean(dim=1)
                    
                    if torch.cuda.is_available():
                        embedding = embedding.cpu()
                    
                    embeddings.append(embedding.squeeze().numpy().tolist())
                    
            return embeddings
            
    def analyze_code(self, model_name: str, code: str, task: str) -> Dict[str, Any]:
        """Analyze code for specific task"""
        self._ensure_model_loaded(model_name)
        
        # For now, use embeddings as a proxy for analysis
        embeddings = self.generate_embeddings(model_name, [code])
        
        # Placeholder analysis based on task
        if task == "quality":
            return {
                "quality_score": 0.85,
                "issues": [],
                "suggestions": ["Code quality looks good"]
            }
        elif task == "patterns":
            return {
                "patterns": ["singleton", "factory"],
                "confidence": 0.75
            }
        elif task == "complexity":
            return {
                "cyclomatic_complexity": 5,
                "cognitive_complexity": 8
            }
        else:
            return {
                "task": task,
                "result": "Analysis completed",
                "embedding_dim": len(embeddings[0])
            }
            
    def generate_text(self, model_name: str, prompt: str, max_tokens: int, temperature: float) -> str:
        """Generate text using a model (placeholder for now)"""
        # For actual implementation, we'd load a generative model like Mistral-7B
        # For now, return a placeholder response
        return f"Generated response for: {prompt[:50]}... (using {model_name})"
        
    def get_chroma_client(self):
        """Get or create Chroma client"""
        if self.chroma_client is None:
            chroma_db_path = os.path.join(self.model_cache_dir, "chroma_db")
            self.chroma_client = chromadb.PersistentClient(
                path=chroma_db_path,
                settings=Settings(anonymized_telemetry=False)
            )
        return self.chroma_client
        
    def process_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Process a single request"""
        request_type = request.get("type")
        request_id = request.get("request_id", str(uuid.uuid4()))
        
        try:
            if request_type == "health":
                return {
                    "type": "health_result",
                    "status": "ready",
                    "models_loaded": list(self.models.keys()),
                    "request_id": request_id
                }
                
            elif request_type == "embed":
                model = request.get("model", "sentence-transformers/all-MiniLM-L6-v2")
                texts = request.get("texts", [])
                embeddings = self.generate_embeddings(model, texts)
                return {
                    "type": "embed_result",
                    "embeddings": embeddings,
                    "request_id": request_id
                }
                
            elif request_type == "generate":
                model = request.get("model", "mistral-7b")
                prompt = request.get("prompt", "")
                max_tokens = request.get("max_tokens", 256)
                temperature = request.get("temperature", 0.7)
                text = self.generate_text(model, prompt, max_tokens, temperature)
                return {
                    "type": "generate_result",
                    "text": text,
                    "request_id": request_id
                }
                
            elif request_type == "analyze":
                model = request.get("model", "microsoft/codebert-base")
                code = request.get("code", "")
                task = request.get("task", "quality")
                result = self.analyze_code(model, code, task)
                return {
                    "type": "analyze_result",
                    "result": result,
                    "request_id": request_id
                }
                
            else:
                return {
                    "type": "error",
                    "error": f"Unknown request type: {request_type}",
                    "request_id": request_id
                }
                
        except Exception as e:
            self.logger.error(f"Error processing request: {e}\n{traceback.format_exc()}")
            return {
                "type": "error",
                "error": str(e),
                "request_id": request_id
            }
            
    def run(self):
        """Main service loop"""
        self.logger.info("Model service started")
        
        # Send ready signal
        print(json.dumps({
            "type": "health_result",
            "status": "ready",
            "models_loaded": [],
            "request_id": "startup"
        }))
        sys.stdout.flush()
        
        # Process requests from stdin
        for line in sys.stdin:
            try:
                request = json.loads(line.strip())
                response = self.process_request(request)
                print(json.dumps(response))
                sys.stdout.flush()
            except json.JSONDecodeError as e:
                self.logger.error(f"Invalid JSON: {e}")
            except Exception as e:
                self.logger.error(f"Unexpected error: {e}\n{traceback.format_exc()}")
                

def main():
    parser = argparse.ArgumentParser(description="AI Model Service for Hive")
    parser.add_argument(
        "--model-cache-dir",
        type=str,
        required=True,
        help="Directory for caching models"
    )
    args = parser.parse_args()
    
    service = ModelService(args.model_cache_dir)
    service.run()
    

if __name__ == "__main__":
    main()