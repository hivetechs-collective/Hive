"use strict";
/**
 * Hive Consensus - Electron Proof of Concept
 * Day 0: Validate Electron + Monaco + Rust architecture
 */
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
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
const monaco = __importStar(require("monaco-editor"));
// Create container for Monaco editor
const container = document.createElement('div');
container.id = 'monaco-container';
container.style.width = '100vw';
container.style.height = '100vh';
container.style.display = 'flex';
container.style.flexDirection = 'column';
document.body.appendChild(container);
// Add header
const header = document.createElement('div');
header.style.padding = '10px';
header.style.backgroundColor = '#1e1e1e';
header.style.color = '#cccccc';
header.style.borderBottom = '1px solid #333';
header.innerHTML = `
  <h2 style="margin: 0; font-size: 16px; font-family: monospace;">
    🐝 Hive Consensus - Day 0 Validation
  </h2>
  <div style="margin-top: 10px;">
    <button id="test-rust" style="margin-right: 10px; padding: 5px 10px;">Test Rust Connection</button>
    <button id="test-consensus" style="padding: 5px 10px;">Test Consensus Engine</button>
    <span id="status" style="margin-left: 20px; color: #4ec9b0;"></span>
  </div>
`;
container.appendChild(header);
// Create editor container
const editorContainer = document.createElement('div');
editorContainer.style.flex = '1';
container.appendChild(editorContainer);
// Initialize Monaco Editor
const editor = monaco.editor.create(editorContainer, {
    value: `// Hive Consensus - Electron + Monaco + Rust Backend
// Day 0 Proof of Concept

console.log("✅ Monaco editor loaded successfully!");

// Test 1: Monaco works in Electron
// Test 2: Connect to Rust backend on :8765
// Test 3: Make real consensus call
// Test 4: Display consensus result

async function testRustConnection() {
  try {
    const response = await fetch('http://localhost:8765/test', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify("Hello from Electron")
    });
    const result = await response.json();
    console.log('Rust backend responded:', result);
    return result;
  } catch (error) {
    console.error('Failed to connect to Rust backend:', error);
    return null;
  }
}

async function testConsensus(query) {
  try {
    const response = await fetch('http://localhost:8765/api/consensus', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({ query })
    });
    const result = await response.json();
    console.log('Consensus result:', result);
    return result;
  } catch (error) {
    console.error('Consensus failed:', error);
    return null;
  }
}

// Ready for testing!
`,
    language: 'javascript',
    theme: 'vs-dark',
    fontSize: 14,
    minimap: { enabled: false },
    automaticLayout: true,
    wordWrap: 'on',
});
// Add button handlers
(_a = document.getElementById('test-rust')) === null || _a === void 0 ? void 0 : _a.addEventListener('click', () => __awaiter(void 0, void 0, void 0, function* () {
    const status = document.getElementById('status');
    if (status) {
        status.textContent = 'Connecting to Rust backend...';
        status.style.color = '#ffcc00';
    }
    try {
        const response = yield fetch('http://localhost:8765/test', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify("Hello from Electron")
        });
        const result = yield response.json();
        if (status) {
            status.textContent = `✅ Rust responded: ${result}`;
            status.style.color = '#4ec9b0';
        }
        // Add result to editor
        const currentValue = editor.getValue();
        editor.setValue(currentValue + `\n\n// Rust backend test successful!\n// Response: ${JSON.stringify(result)}`);
    }
    catch (error) {
        if (status) {
            status.textContent = '❌ Failed to connect to Rust backend (is it running on :8765?)';
            status.style.color = '#ff6b6b';
        }
    }
}));
(_b = document.getElementById('test-consensus')) === null || _b === void 0 ? void 0 : _b.addEventListener('click', () => __awaiter(void 0, void 0, void 0, function* () {
    const status = document.getElementById('status');
    if (status) {
        status.textContent = 'Running consensus engine...';
        status.style.color = '#ffcc00';
    }
    try {
        const response = yield fetch('http://localhost:8765/api/consensus', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ query: "What is the capital of France?" })
        });
        const result = yield response.json();
        if (status) {
            status.textContent = '✅ Consensus completed!';
            status.style.color = '#4ec9b0';
        }
        // Add result to editor
        const currentValue = editor.getValue();
        editor.setValue(currentValue + `\n\n// Consensus engine test successful!\n// Result: ${JSON.stringify(result, null, 2)}`);
    }
    catch (error) {
        if (status) {
            status.textContent = '❌ Consensus failed (is backend running?)';
            status.style.color = '#ff6b6b';
        }
    }
}));
// Log success
console.log('✅ Day 0 Step 1: Electron + Monaco setup complete!');
//# sourceMappingURL=renderer-monaco.js.map