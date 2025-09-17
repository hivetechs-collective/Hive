# UI Smoke Tests

These Playwright tests automate the packaged Hive Consensus build through the `window.commandAPI` command layer.

## Prerequisites

1. Run the full 17-phase production build once:
   ```bash
   npm run build:complete
   ```
2. Ensure the packaged binary remains available under `out/` (or install it to `/Applications`).

## Running the Suite

```bash
npm run test:ui
```

The runner checks that a packaged binary exists, allocates a remote-debugging port (preferring `PortManager` allocations when available), and then executes the Playwright suite.

### Attach Mode (Reuse a Manually Launched App)

If you already have the packaged app running, start it with remote debugging enabled:

```bash
PLAYWRIGHT_E2E=1 PLAYWRIGHT_REMOTE_DEBUG_PORT=9450 \
  /Applications/Hive\ Consensus.app/Contents/MacOS/Hive\ Consensus
```

Then run the suite in attach mode so Playwright connects to that instance without launching another process:

```bash
PLAYWRIGHT_REMOTE_DEBUG_PORT=9450 npm run test:ui -- --attach
```

### Default Mode (Launches Once per Suite)

Without `--attach` the suite launches the packaged app itself, waits for `window.commandAPI` to be ready, and closes it when the tests finish.

## Current Coverage

- Welcome → Documentation shortcuts (AI Workflows, What's New) driven via commandAPI.
- Help menu commands (`help.showGettingStarted`, `help.showMemoryGuide`, `help.showAbout`) with modal/dialog assertions.
- Activity bar toggles for Explorer, Git, Settings, Memory, CLI Tools, and Analytics panels.
- TTYD terminal collapse/expand behaviour (toggle button and `expandTTYDTerminal` helper).

## Extending Coverage

- Add new specs under `tests/ui/` and import the shared helpers in `welcome-documentation.spec.ts`.
- Drive behaviour exclusively through `window.commandAPI.executeCommand` so the smoke tests continue to exercise the single command layer.
- Future targets include:
  - Help menu printing/export flows once implemented.
  - CLI tool launch telemetry and analytics panels once deterministic hooks are exposed.
  - Additional regression checks around consensus logging and memory dashboard widgets.

Keep the suite fast: reuse the packaged launch, avoid rebuilding, and prefer DOM/state assertions over screenshots.
