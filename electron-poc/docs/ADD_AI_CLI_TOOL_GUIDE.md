# Complete Guide: Adding a New AI CLI Tool to Hive Consensus

## Overview
This guide provides step-by-step instructions for adding a new AI CLI tool to the Hive Consensus IDE. Each tool appears in the activity bar and has a panel in the AI CLI Tools window.

## Prerequisites
- Tool name and description
- NPM package name (or installation command)
- Logo/icon for the tool
- Documentation URL

## Step-by-Step Process

### Step 1: Create SVG Icon
**Location:** `resources/ai-cli-icons/[tool-name].svg`

Create a 24x24 SVG icon file. Example structure:
```xml
<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="none">
  <!-- Tool icon paths here -->
</svg>
```

**Naming convention:** Use lowercase with hyphens (e.g., `deepseek.svg`, `aider.svg`, `v0.svg`)

### Step 2: Add Tool to Registry
**File:** `src/shared/types/cli-tools.ts`

Add the tool configuration to `CLI_TOOLS_REGISTRY` object (before the closing brace):

```typescript
'tool-id': {
  id: 'tool-id',
  name: 'Tool Display Name',
  description: 'Brief description of the tool',
  command: 'tool-command',  // Command to run in terminal
  installCommand: 'npm install -g package-name',
  updateCommand: 'npm update -g package-name',
  versionCommand: 'tool-command --version',
  versionRegex: /(\d+\.\d+\.\d+)/,
  docsUrl: 'https://tool-documentation.com',
  icon: '🔧',  // Emoji icon (optional)
  requiresNode: true  // If it's an npm package
},
```

### Step 3: Import Icon in Renderer
**File:** `src/renderer.ts`

Add import statement with other icon imports (around line 62-67):
```typescript
import toolIcon from '../resources/ai-cli-icons/tool-name.svg';
```

### Step 4: Add Activity Bar Button
**File:** `src/renderer.ts`

Find the activity bar buttons section (around line 440-455) and add button in desired position:
```typescript
<button class="activity-btn cli-quick-launch" data-tool="tool-id" aria-label="Tool Name">
  <img src="${toolIcon}" width="24" height="24" alt="Tool Name" style="object-fit: contain;" />
  <span class="activity-tooltip">Tool Name</span>
</button>
```

**Note:** Place between existing buttons to control order on activity bar

### Step 5: Add Panel Card
**File:** `src/renderer.ts`

In the `renderCliToolsPanel` function (around line 3900-3980), add the tool card:
```typescript
// Tool Name - Description
const toolStatus = await electronAPI.detectCliTool('tool-id');
gridContainer.appendChild(createCliToolCard({
  id: 'tool-id',
  name: 'Tool Display Name',
  description: 'Tool description for panel',
  status: toolStatus,
  docsUrl: 'https://tool-documentation.com',
  badgeText: 'BADGE TEXT',  // Optional badge
  badgeColor: '#COLOR'      // Optional badge color
}));
```

### Step 6: Update Install/Update/Uninstall All Arrays
**File:** `src/renderer.ts`

Update three arrays to include the new tool:

1. **Install All** (around line 4441):
```typescript
const toolsToInstall = [
  'claude-code',
  'gemini-cli',
  'qwen-code',
  'openai-codex',
  'deepseek',
  'tool-id',  // Add new tool
  'grok'
];
```

2. **Also update the refresh array** (around line 4526):
```typescript
const toolsToInstall = ['claude-code', 'gemini-cli', 'qwen-code', 'openai-codex', 'deepseek', 'tool-id', 'grok'];
```

3. **Update All array** (around line 4547 and 4641):
```typescript
const toolsToUpdate = [
  'claude-code',
  'gemini-cli',
  'qwen-code',
  'openai-codex',
  'deepseek',
  'tool-id',  // Add new tool
  'grok'
];
```

4. **Uninstall All array** (around line 4754 and 4833):
```typescript
const toolsToUninstall = [
  'claude-code',
  'gemini-cli',
  'qwen-code',
  'openai-codex',
  'deepseek',
  'tool-id',  // Add new tool
  'grok'
];
```

### Step 7: Update Main Process
**File:** `src/index.ts`

#### 7.1: Update Supported Tools Arrays (2 locations)
Around line 2620:
```typescript
const supportedTools = ['claude-code', 'gemini-cli', 'qwen-code', 'openai-codex', 'deepseek', 'tool-id', 'grok'];
```

Around line 2657:
```typescript
const supportedTools = ['claude-code', 'gemini-cli', 'qwen-code', 'openai-codex', 'deepseek', 'tool-id', 'grok'];
```

#### 7.2: Add NPM Package Mapping (if different from registry)
Around line 3210 and 3494:
```typescript
const npmPackages: Record<string, string> = {
  'claude-code': '@anthropic-ai/claude-code',
  'gemini-cli': '@google/gemini-cli',
  'qwen-code': '@qwen-code/qwen-code',
  'openai-codex': '@openai/codex',
  'deepseek': 'deepseek-cli',
  'tool-id': 'npm-package-name',  // Add if package name differs
  'grok': '@vibe-kit/grok-cli'
};
```

#### 7.3: Add Version Detection
Around line 3005-3012, add version extraction logic if needed:
```typescript
} else if (toolId === 'tool-id') {
  // Tool-specific version extraction
  const match = versionResult.stdout.match(/(\d+\.\d+\.\d+)/);
  version = match ? match[1] : 'Unknown';
```

### Step 8: Optional - Special Configuration
If the tool needs special handling (API keys, wrappers, etc.):

1. **For API key configuration:** Add logic in `cli-tool-configure` handler
2. **For special launch commands:** Add conditions in `cli-tool-launch` handler around line 3840
3. **For wrapper scripts:** Create wrapper file and reference it

### Step 9: Testing Checklist

#### 9.1: TypeScript Compilation Check
```bash
npx tsc --noEmit
```

#### 9.2: Quick Visual Test (Optional)
```bash
npm run dev
```

#### 9.3: Full Production Build

Run the comprehensive production build:

```bash
cd electron-poc
npm run build:complete
```

**What the build does:**
- Executes 17 phases including TypeScript compilation, webpack bundling, and DMG creation
- Shows progress in the current terminal
- Phase 11 (Application Build) is the longest, typically taking 2-5 minutes
- Total build time: 5-10 minutes
- Automatically opens the DMG when complete for testing

**Note:** Always use `npm run build:complete` for production builds. This ensures all build phases are executed in the correct order with proper verification.

#### 9.4: Verify Functionality
After build completes and DMG opens:
- [ ] Icon appears in activity bar in correct position
- [ ] Tooltip shows on hover
- [ ] Click opens AI CLI Tools panel
- [ ] Tool card appears in panel
- [ ] Detect button works
- [ ] Install button works (if not installed)
- [ ] Update button works (if installed)
- [ ] Uninstall button works (if installed)
- [ ] Launch button opens terminal with tool
- [ ] Tool included in Install All
- [ ] Tool included in Update All
- [ ] Tool included in Uninstall All

### Step 10: Commit Changes
```bash
git add -A
git commit -m "feat(cli-tools): add [Tool Name] integration

- Add [Tool Name] to CLI_TOOLS_REGISTRY
- Create [Tool Name] SVG icon
- Add [Tool Name] button to activity bar
- Add [Tool Name] panel card in AI CLI Tools window
- Include [Tool Name] in Install/Update/Uninstall All operations
- Add version detection for [Tool Name]
- Update all supported tools arrays

[Tool Name] provides [brief description of functionality]"
```

## Common Issues & Solutions

### Issue: Icon not showing
- Check SVG file path and name matches import
- Ensure SVG is valid (viewBox="0 0 24 24")
- Verify import statement in renderer.ts

### Issue: Tool not detected
- Check tool ID consistency across all files
- Verify command name in registry matches actual CLI command
- Check npm package name in index.ts mappings

### Issue: Version not showing
- Test version command manually: `tool-command --version`
- Adjust regex pattern for version extraction
- Add specific version detection logic in index.ts

### Issue: Install/Update fails
- Verify npm package name is correct
- Check if package requires special installation flags
- Ensure package is published to npm

## File Reference Summary

| File | Purpose | Key Sections |
|------|---------|--------------|
| `src/shared/types/cli-tools.ts` | Tool registry | CLI_TOOLS_REGISTRY object |
| `src/renderer.ts` | UI components | Imports, activity bar, panel cards, arrays |
| `src/index.ts` | Backend logic | Supported tools, npm packages, version detection |
| `resources/ai-cli-icons/` | Icon storage | SVG files |

## Current Tool Order (as of latest update)
1. Claude Code
2. Gemini CLI
3. Qwen Code
4. OpenAI Codex
5. DeepSeek
6. [Next position for new tool]
7. Consensus (special - not a CLI tool)
8. Grok CLI

## Notes
- Always test TypeScript compilation before full build
- Keep consistent naming across all files (tool ID)
- Place new tools logically in UI (group similar tools)
- Document any special requirements or configurations
- The full build takes 5-10 minutes to complete
- After DMG opens, drag to Applications if you want to install

## Build Script Details
The production build uses a comprehensive 17-phase build system:
- **Command**: `npm run build:complete`
- Executes `scripts/build-production-dmg.js` with all verification steps
- Shows progress in the current terminal
- Phase 11 (Application Build with webpack) is typically the longest phase
- Always use this command for consistent, reliable builds

## Next Tools to Add
- [ ] Aider - AI pair programming tool
- [ ] v0 - Vercel's AI development tool
- [ ] Additional tools as needed