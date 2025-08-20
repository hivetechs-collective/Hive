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
                    # FIXED: Check available attributes to prevent tensor attribute errors
                    if hasattr(outputs, 'pooler_output') and outputs.pooler_output is not None:
                        embedding = outputs.pooler_output
                    elif hasattr(outputs, 'last_hidden_state') and outputs.last_hidden_state is not None:
                        embedding = outputs.last_hidden_state.mean(dim=1)
                    else:
                        # Fallback: use first output if it's a tensor
                        if hasattr(outputs, '__getitem__') and len(outputs) > 0:
                            embedding = outputs[0]
                            if len(embedding.shape) > 2:  # If sequence output, take mean
                                embedding = embedding.mean(dim=1)
                        else:
                            # Last resort: create zero embedding
                            embedding = torch.zeros(1, 768)  # Standard BERT dimension
                    
                    # FIXED: Always move to CPU regardless of device (CUDA, MPS, etc.)
                    if hasattr(embedding, 'cpu'):
                        embedding = embedding.cpu()
                    elif hasattr(embedding, 'detach'):
                        embedding = embedding.detach()
                    
                    # Ensure it's a tensor before converting to numpy
                    if not isinstance(embedding, torch.Tensor):
                        embedding = torch.tensor(embedding)
                    
                    embeddings.append(embedding.squeeze().numpy().tolist())
                    
            return embeddings
            
    def analyze_code(self, model_name: str, code: str, task: str) -> Dict[str, Any]:
        """Analyze code for specific task with proper tensor handling"""
        self._ensure_model_loaded(model_name)
        
        try:
            # Generate embeddings for analysis
            embeddings = self.generate_embeddings(model_name, [code])
            
            if task == "quality":
                # FIXED: Use embeddings directly instead of non-existent attributes
                embedding = embeddings[0] if embeddings else []
                quality_score = self._calculate_quality_score(embedding)
                
                return {
                    "quality_score": quality_score,
                    "consistency": 0.85,  # Placeholder - implement actual analysis
                    "completeness": 0.90,
                    "confidence": 0.88,
                    "issues": []
                }
            elif task == "patterns":
                return self._analyze_patterns(embeddings[0] if embeddings else [])
            elif task == "complexity":
                return self._analyze_complexity(embeddings[0] if embeddings else [])
            else:
                return {
                    "task": task,
                    "result": "Analysis completed",
                    "embedding_dim": len(embeddings[0]) if embeddings else 0,
                    "confidence": 0.75
                }
                
        except Exception as e:
            self.logger.error(f"Analysis error: {e}")
            return {"error": str(e)}
    
    def _calculate_quality_score(self, embedding: List[float]) -> float:
        """Calculate quality score from embedding"""
        if not embedding:
            return 0.5
        # Simple heuristic: higher dimensional variance = more complex = higher quality
        variance = float(np.var(embedding)) if len(embedding) > 0 else 0.0
        return min(0.9, max(0.1, variance * 100))  # Scale to 0.1-0.9 range
    
    def _analyze_patterns(self, embedding: List[float]) -> Dict[str, Any]:
        """Analyze code patterns from embedding"""
        if not embedding:
            return {"patterns": [], "confidence": 0.0}
        
        # Simple pattern detection based on embedding characteristics
        patterns = []
        if np.mean(embedding) > 0.1:
            patterns.append("functional")
        if np.std(embedding) > 0.2:
            patterns.append("complex")
        if len(embedding) > 500:
            patterns.append("detailed")
            
        return {
            "patterns": patterns,
            "confidence": 0.75
        }
    
    def _analyze_complexity(self, embedding: List[float]) -> Dict[str, Any]:
        """Analyze code complexity from embedding"""
        if not embedding:
            return {"cyclomatic_complexity": 1, "cognitive_complexity": 1}
        
        # Estimate complexity from embedding characteristics
        std_dev = float(np.std(embedding)) if len(embedding) > 0 else 0.0
        cyclomatic = max(1, min(20, int(std_dev * 50)))
        cognitive = max(1, min(30, int(std_dev * 75)))
        
        return {
            "cyclomatic_complexity": cyclomatic,
            "cognitive_complexity": cognitive
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