# Terminal Backend Switch Plan (TTYD ➜ node-pty)

Owner: Desktop (Hive Consensus)
Status: In progress
Updated: 2025-10-03

## Purpose
Make xterm + node-pty the default terminal backend (single-process PTY) and keep TTYD optional behind a flag. Eliminate Keychain prompts for public Git ops and fix terminal launch issues seen on some Macs.

## Context Snapshot (what’s already done)
- Git/Keychain (committed):
  - Disable osxkeychain per-process for app-invoked Git (public repos don’t prompt).
  - Gate our SystemCredentialProvider behind `HIVE_ENABLE_SYSTEM_KEYCHAIN=1`.
  - Commits (release/v1.8.506-flex-panels):
    - 197d7d6daa git: disable osxkeychain; gate SystemCredentialProvider
- Terminal (TTYD path currently active):
  - Added runtime fallback on arch-mismatch to use system ttyd.
  - Commit: 4f6e4ee36b terminal: add arch-mismatch fallback for ttyd

## Current Code Reality
- Active backend: TTYD via `src/terminal-ipc-handlers.ts` → `new TTYDManager(...)` (spawns `ttyd`).
- PTY plumbing exists:
  - Renderer: `src/components/terminal/TerminalManager.ts` (xterm.js + terminalAPI).
  - Preload: `src/preload.ts` exposes `terminalAPI` (create/write/resize/kill/on*).
  - Node-pty deps + rebuild documented; webpack marks node-pty external.

## Plan: Switch to PTY backend (remove TTYD)
- Remove TTYD usage path
  - Delete/retire TTYDManager wiring from `src/terminal-ipc-handlers.ts`.
  - Keep source file only if needed for reference; do not ship or instantiate.
- Implement PTY manager (main process)
  - New: `src/services/PTYManager.ts`.
  - API: `create(id,{cwd,env,command,args})`, `write(id,data)`, `resize(id,cols,rows)`, `kill(id)`, `list/status`.
  - Internals: `pty.spawn(shell,args,{cols,rows,cwd,env})`; emit `terminal-data`, `terminal-exit`, `terminal-ready`.
- Wire IPC to PTY manager
  - `create-terminal-process`/`write-to-terminal`/`resize-terminal`/`kill-terminal-process` → PTYManager.
- Renderer unchanged
  - Continues to use `window.terminalAPI.*`; xterm UI intact; AI CLI tools launch in dedicated tabs.
  - All terminal features run over node-pty; no TTYD server or port URLs.

## Local Test Matrix (pre-release)
- Machines: Apple Silicon (M‑series) and Intel (if available).
- Scenarios:
  - Open/resize/kill multiple terminals; verify I/O and scrollback.
  - Launch AI CLI tool tabs (auto-exec command then interactive shell).
  - Public repo Git ops (clone/fetch/pull) → no Keychain prompts.
  - Private repo ops (optional): enable `HIVE_ENABLE_SYSTEM_KEYCHAIN=1` and verify macOS Keychain path.
- Diagnostics:
  - Verify `electron-rebuild` for node-pty; codesign/notarize all natives.
  - Gatekeeper checks: `spctl -a -vv` and `stapler validate` on app & DMG.

## Release 1.8.507
- Bump version: `electron-poc/package.json` → `1.8.507`.
- Build pipeline:
  - `npx electron-rebuild --force --only sqlite3,better-sqlite3,node-pty`.
  - `scripts/build-production-dmg.js` → DMG.
  - `scripts/sign-notarize-macos.sh` → sign + notarize + staple.
  - Validate with `spctl` + `stapler`.
- Homebrew tap update:
  - `homebrew-tap/Casks/hive-consensus.rb` → update version + `sha256`.
  - With PTY default, no `ttyd` dependency required.

## Acceptance Criteria
- Terminals and AI CLI tabs launch reliably on Apple Silicon (M4).
- No Keychain popups for public Git operations initiated by the app.
- Private repo flows use Keychain only when `HIVE_ENABLE_SYSTEM_KEYCHAIN=1`.
- Renderer terminal UX unchanged; dedicated tabs; I/O and resize stable.
- DMG installs via Homebrew without prompts; app passes Gatekeeper.

## Controls / Rollback
- Set `HIVE_TERMINAL_BACKEND=ttyd` to re-enable TTYD backend.
- Set `HIVE_ENABLE_SYSTEM_KEYCHAIN=1` to enable macOS Keychain provider.

## Immediate Workarounds (for 1.8.506)
- If terminal fails on a host: `brew install ttyd` (old build tries system locations).
- Re-run app; terminal should work if the bundled helper was the issue.

---
Prepared by: migration plan snapshot
