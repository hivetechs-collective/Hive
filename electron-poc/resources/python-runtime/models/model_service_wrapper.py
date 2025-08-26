#!/usr/bin/env python3
"""
AI Helper Service Wrapper
Handles package installation on-demand for production
"""
import sys
import os
import subprocess
import json

def ensure_package(package_name, import_name=None):
    """Ensure a package is installed, installing it if necessary"""
    if import_name is None:
        import_name = package_name
    
    try:
        __import__(import_name)
        return True
    except ImportError:
        print(f"Installing {package_name}...", file=sys.stderr)
        try:
            subprocess.check_call([
                sys.executable, '-m', 'pip', 'install', 
                '--quiet', '--disable-pip-version-check',
                package_name
            ])
            return True
        except subprocess.CalledProcessError:
            return False

# For AI Helpers, we'll skip heavy ML packages and use simpler alternatives
# The consensus routing can work without them in production
def init_minimal_mode():
    """Initialize minimal mode without ML packages"""
    return {
        "type": "initialized",
        "mode": "minimal",
        "message": "AI Helpers running in minimal mode (no ML models)"
    }

# Check if we're being called as model_service
if __name__ == "__main__":
    # Ensure basic packages
    ensure_package('requests')
    
    # Check if heavy packages are available
    try:
        import torch
        import transformers
        mode = "full"
    except ImportError:
        mode = "minimal"
    
    if mode == "minimal":
        # Run in minimal mode - handle all expected request types
        # Don't print init message - wait for requests instead
        sys.stderr.write("Python minimal mode started, waiting for requests...\n")
        sys.stderr.flush()
        
        # Main request handling loop
        while True:
            try:
                line = input()
                if not line:
                    continue
                    
                request = json.loads(line)
                request_type = request.get("type")
                request_id = request.get("request_id", "unknown")
                
                # Handle health check requests
                if request_type == "health":
                    response = {
                        "type": "health_result",
                        "request_id": request_id,
                        "status": "ready",
                        "models_loaded": []  # No ML models in minimal mode
                    }
                    print(json.dumps(response))
                    sys.stdout.flush()
                    sys.stderr.write(f"Responded to health check: {request_id}\n")
                    sys.stderr.flush()
                    
                # Handle route decision requests
                elif request_type == "route_decision":
                    query = request.get("query", "")
                    # Simple queries are short and don't have complex keywords
                    is_simple = (
                        len(query) < 100 and 
                        not any(word in query.lower() for word in [
                            'analyze', 'debug', 'implement', 'architecture', 
                            'design', 'optimize', 'refactor'
                        ])
                    )
                    
                    response = {
                        "type": "route_decision",
                        "request_id": request_id,
                        "mode": "simple" if is_simple else "complex",
                        "confidence": 0.8
                    }
                    print(json.dumps(response))
                    sys.stdout.flush()
                    sys.stderr.write(f"Routed query as: {'simple' if is_simple else 'complex'}\n")
                    sys.stderr.flush()
                    
                # Handle unknown request types
                else:
                    response = {
                        "type": "error",
                        "request_id": request_id,
                        "error": f"Unknown request type: {request_type}"
                    }
                    print(json.dumps(response))
                    sys.stdout.flush()
                    sys.stderr.write(f"Unknown request type: {request_type}\n")
                    sys.stderr.flush()
                    
            except EOFError:
                sys.stderr.write("EOF received, exiting...\n")
                sys.stderr.flush()
                break
            except json.JSONDecodeError as e:
                error_response = {
                    "type": "error",
                    "request_id": "decode_error",
                    "error": f"JSON decode error: {str(e)}"
                }
                print(json.dumps(error_response))
                sys.stdout.flush()
                sys.stderr.write(f"JSON decode error: {str(e)}\n")
                sys.stderr.flush()
            except Exception as e:
                error_response = {
                    "type": "error",
                    "request_id": request_id if 'request_id' in locals() else "unknown",
                    "error": str(e)
                }
                print(json.dumps(error_response))
                sys.stdout.flush()
                sys.stderr.write(f"Exception: {str(e)}\n")
                sys.stderr.flush()
    else:
        # Full mode with ML packages - import the real service
        from model_service import main
        main()
