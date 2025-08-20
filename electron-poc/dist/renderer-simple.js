"use strict";
/**
 * Hive Consensus - Simple Electron Proof of Concept
 * Day 0: Validate Electron + Rust architecture WITHOUT Monaco
 */
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var _a, _b;
Object.defineProperty(exports, "__esModule", { value: true });
require("./index.css");
// Create simple UI
document.body.innerHTML = `
<div style="padding: 20px; font-family: monospace; background: #1e1e1e; color: #ccc; height: 100vh;">
  <h1 style="color: #4ec9b0;">🐝 Hive Consensus - Day 0 Validation (Simple)</h1>
  
  <div style="margin: 20px 0;">
    <button id="test-rust" style="padding: 10px 20px; margin-right: 10px;">Test Rust Connection</button>
    <button id="test-consensus" style="padding: 10px 20px;">Test Consensus Engine</button>
  </div>
  
  <div id="status" style="padding: 10px; background: #252526; border-radius: 4px; margin: 20px 0;">
    Status: Ready
  </div>
  
  <div style="background: #252526; padding: 20px; border-radius: 4px; height: 400px; overflow-y: auto;">
    <h3>Results:</h3>
    <pre id="results" style="color: #4ec9b0; font-size: 14px;">Waiting for tests...</pre>
  </div>
</div>
`;
const status = document.getElementById('status');
const results = document.getElementById('results');
// Test Rust connection
(_a = document.getElementById('test-rust')) === null || _a === void 0 ? void 0 : _a.addEventListener('click', () => __awaiter(void 0, void 0, void 0, function* () {
    status.textContent = 'Status: Connecting to Rust backend...';
    status.style.color = '#ffcc00';
    try {
        const response = yield fetch('http://localhost:8765/test', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify("Hello from Electron")
        });
        const result = yield response.json();
        status.textContent = 'Status: ✅ Connected to Rust backend!';
        status.style.color = '#4ec9b0';
        results.textContent = `Test Connection Success!\n${JSON.stringify(result, null, 2)}`;
        console.log('✅ Day 0 Step 3: Electron connected to Rust backend!');
    }
    catch (error) {
        status.textContent = 'Status: ❌ Failed to connect to Rust backend';
        status.style.color = '#ff6b6b';
        results.textContent = `Connection failed:\n${error}`;
    }
}));
// Test consensus
(_b = document.getElementById('test-consensus')) === null || _b === void 0 ? void 0 : _b.addEventListener('click', () => __awaiter(void 0, void 0, void 0, function* () {
    status.textContent = 'Status: Running consensus engine...';
    status.style.color = '#ffcc00';
    try {
        const response = yield fetch('http://localhost:8765/api/consensus', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ query: "What is the capital of France?" })
        });
        const result = yield response.json();
        status.textContent = 'Status: ✅ Consensus completed!';
        status.style.color = '#4ec9b0';
        results.textContent = `Consensus Engine Success!\n${JSON.stringify(result, null, 2)}`;
        console.log('✅ Day 0 Step 4: Real consensus call succeeded!');
    }
    catch (error) {
        status.textContent = 'Status: ❌ Consensus failed';
        status.style.color = '#ff6b6b';
        results.textContent = `Consensus failed:\n${error}`;
    }
}));
// Auto-test on load
setTimeout(() => __awaiter(void 0, void 0, void 0, function* () {
    console.log('Auto-testing Rust connection...');
    try {
        const response = yield fetch('http://localhost:8765/health');
        const health = yield response.json();
        console.log('Backend health check:', health);
        results.textContent = `Backend is healthy!\n${JSON.stringify(health, null, 2)}`;
    }
    catch (error) {
        console.error('Backend not reachable:', error);
        results.textContent = `Backend not reachable. Is it running on port 8765?\n${error}`;
    }
}), 1000);
//# sourceMappingURL=renderer-simple.js.map