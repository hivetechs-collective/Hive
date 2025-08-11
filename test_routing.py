import asyncio
import websockets
import json

async def test_routing():
    uri = "ws://localhost:8765/ws"
    
    # Test queries
    queries = [
        ("What is 2 + 2?", "Simple math - should use Direct mode"),
        ("Explain how the authentication system in this codebase works", "Complex analysis - should use Consensus"),
        ("What is the name of this repository?", "Simple factual - should use Direct"),
        ("Analyze the architecture of this project", "Complex analysis - should use Consensus"),
    ]
    
    for query, expected in queries:
        print(f"\n{'='*60}")
        print(f"Testing: {query}")
        print(f"Expected: {expected}")
        print(f"{'='*60}")
        
        async with websockets.connect(uri) as websocket:
            # Send start consensus message
            message = {
                "type": "start_consensus",
                "query": query,
                "profile": None,
                "conversation_id": None,
                "context": None
            }
            
            await websocket.send(json.dumps(message))
            
            # Listen for AI Helper decision
            while True:
                response = await websocket.recv()
                data = json.loads(response)
                
                if data.get("type") == "ai_helper_decision":
                    print(f"AI Decision: {'Direct Mode' if data['direct_mode'] else 'Consensus Mode'}")
                    print(f"Reason: {data['reason']}")
                    break
                elif data.get("type") == "error":
                    print(f"Error: {data['message']}")
                    break
                elif data.get("type") in ["stage_started", "consensus_complete"]:
                    # Skip other messages for now
                    continue
        
        await asyncio.sleep(1)

asyncio.run(test_routing())
