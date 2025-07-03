#!/usr/bin/env python3
"""
Integration test for Hive AI MCP and LSP servers
Tests the IDE integration functionality
"""

import json
import requests
import time
import subprocess
import signal
import os
import sys
from typing import Optional

class HiveIntegrationTest:
    def __init__(self):
        self.mcp_url = "http://127.0.0.1:7777"
        self.lsp_url = "http://127.0.0.1:7778"
        self.server_process: Optional[subprocess.Popen] = None
        self.request_id = 1

    def start_servers(self) -> bool:
        """Start the Hive AI servers for testing"""
        try:
            print("🚀 Starting Hive AI servers...")
            
            # Start both MCP and LSP servers
            self.server_process = subprocess.Popen([
                "cargo", "run", "--", "serve", "--mode", "both", "--port", "7777"
            ], cwd="/Users/veronelazio/Developer/Private/hive")
            
            # Wait for servers to start
            time.sleep(3)
            
            print("✅ Servers started")
            return True
            
        except Exception as e:
            print(f"❌ Failed to start servers: {e}")
            return False

    def stop_servers(self):
        """Stop the test servers"""
        if self.server_process:
            print("🛑 Stopping servers...")
            self.server_process.terminate()
            self.server_process.wait(timeout=10)
            print("✅ Servers stopped")

    def send_mcp_request(self, method: str, params: dict = None) -> dict:
        """Send MCP JSON-RPC request"""
        if params is None:
            params = {}
        
        request_data = {
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params
        }
        
        self.request_id += 1
        
        try:
            response = requests.post(
                self.mcp_url,
                json=request_data,
                headers={"Content-Type": "application/json"},
                timeout=30
            )
            
            if response.status_code == 200:
                return response.json()
            else:
                print(f"HTTP Error: {response.status_code}")
                return {}
                
        except Exception as e:
            print(f"Request error: {e}")
            return {}

    def test_mcp_initialize(self) -> bool:
        """Test MCP server initialization"""
        print("🧪 Testing MCP initialization...")
        
        result = self.send_mcp_request("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {"experimental": {}},
            "clientInfo": {
                "name": "Hive Integration Test",
                "version": "1.0.0"
            }
        })
        
        if result.get("result"):
            print("✅ MCP initialization successful")
            
            # Send initialized notification
            self.send_mcp_request("initialized")
            return True
        else:
            print("❌ MCP initialization failed")
            print(f"Response: {result}")
            return False

    def test_mcp_list_tools(self) -> bool:
        """Test listing available tools"""
        print("🧪 Testing MCP tool listing...")
        
        result = self.send_mcp_request("tools/list")
        
        if result.get("result") and "tools" in result["result"]:
            tools = result["result"]["tools"]
            print(f"✅ Found {len(tools)} tools:")
            for tool in tools:
                print(f"   • {tool['name']}: {tool['description']}")
            return True
        else:
            print("❌ Failed to list tools")
            print(f"Response: {result}")
            return False

    def test_mcp_ask_hive(self) -> bool:
        """Test the ask_hive tool"""
        print("🧪 Testing ask_hive tool...")
        
        result = self.send_mcp_request("tools/call", {
            "name": "ask_hive",
            "arguments": {
                "question": "What is the purpose of this integration test?",
                "context": "This is a test of the Hive AI MCP server"
            }
        })
        
        if (result.get("result") and 
            "content" in result["result"] and 
            len(result["result"]["content"]) > 0):
            
            response_text = result["result"]["content"][0].get("text", "")
            print("✅ ask_hive tool works!")
            print(f"   Response: {response_text[:100]}...")
            return True
        else:
            print("❌ ask_hive tool failed")
            print(f"Response: {result}")
            return False

    def test_mcp_explain_code(self) -> bool:
        """Test the explain_code tool"""
        print("🧪 Testing explain_code tool...")
        
        test_code = """
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
"""
        
        result = self.send_mcp_request("tools/call", {
            "name": "explain_code",
            "arguments": {
                "code": test_code,
                "language": "rust"
            }
        })
        
        if (result.get("result") and 
            "content" in result["result"] and 
            len(result["result"]["content"]) > 0):
            
            explanation = result["result"]["content"][0].get("text", "")
            print("✅ explain_code tool works!")
            print(f"   Explanation: {explanation[:100]}...")
            return True
        else:
            print("❌ explain_code tool failed")
            print(f"Response: {result}")
            return False

    def test_mcp_list_resources(self) -> bool:
        """Test listing available resources"""
        print("🧪 Testing MCP resource listing...")
        
        result = self.send_mcp_request("resources/list")
        
        if result.get("result") and "resources" in result["result"]:
            resources = result["result"]["resources"]
            print(f"✅ Found {len(resources)} resources:")
            for resource in resources[:5]:  # Show first 5
                print(f"   • {resource['name']}: {resource['uri']}")
            return True
        else:
            print("❌ Failed to list resources")
            print(f"Response: {result}")
            return False

    def test_lsp_initialize(self) -> bool:
        """Test LSP server initialization"""
        print("🧪 Testing LSP initialization...")
        
        try:
            result = self.send_lsp_request("initialize", {
                "processId": os.getpid(),
                "clientInfo": {
                    "name": "Hive Integration Test",
                    "version": "1.0.0"
                },
                "capabilities": {
                    "textDocument": {
                        "completion": {},
                        "hover": {}
                    }
                }
            })
            
            if result.get("result"):
                print("✅ LSP initialization successful")
                
                # Send initialized notification
                self.send_lsp_request("initialized")
                return True
            else:
                print("❌ LSP initialization failed")
                print(f"Response: {result}")
                return False
                
        except Exception as e:
            print(f"❌ LSP test failed: {e}")
            return False

    def send_lsp_request(self, method: str, params: dict = None) -> dict:
        """Send LSP JSON-RPC request"""
        if params is None:
            params = {}
        
        request_data = {
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params
        }
        
        self.request_id += 1
        
        try:
            response = requests.post(
                self.lsp_url,
                json=request_data,
                headers={"Content-Type": "application/json"},
                timeout=30
            )
            
            if response.status_code == 200:
                return response.json()
            else:
                print(f"HTTP Error: {response.status_code}")
                return {}
                
        except Exception as e:
            print(f"Request error: {e}")
            return {}

    def run_all_tests(self) -> bool:
        """Run all integration tests"""
        print("🐝 Hive AI IDE Integration Test Suite")
        print("=" * 50)
        
        if not self.start_servers():
            return False
        
        try:
            tests = [
                self.test_mcp_initialize,
                self.test_mcp_list_tools,
                self.test_mcp_ask_hive,
                self.test_mcp_explain_code,
                self.test_mcp_list_resources,
                self.test_lsp_initialize,
            ]
            
            passed = 0
            total = len(tests)
            
            for test in tests:
                if test():
                    passed += 1
                print()
            
            print("=" * 50)
            print(f"📊 Test Results: {passed}/{total} passed")
            
            if passed == total:
                print("🎉 All tests passed! IDE integration is working correctly.")
                return True
            else:
                print("❌ Some tests failed. Please check the server implementation.")
                return False
                
        finally:
            self.stop_servers()

def main():
    """Main test runner"""
    tester = HiveIntegrationTest()
    
    try:
        success = tester.run_all_tests()
        sys.exit(0 if success else 1)
    except KeyboardInterrupt:
        print("\n🛑 Test interrupted by user")
        tester.stop_servers()
        sys.exit(1)
    except Exception as e:
        print(f"❌ Test suite failed: {e}")
        tester.stop_servers()
        sys.exit(1)

if __name__ == "__main__":
    main()