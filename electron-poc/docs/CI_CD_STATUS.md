# Hive Consensus CI/CD Status – Electron Build & Memory Service Investigation

## Context
- **Branch under test:** `memory-context-cicd`
- **Reference architecture:** `electron-poc/MASTER_ARCHITECTURE.md` (dynamic ProcessManager/PortManager, zero-fallback ports, Electron-only production build)
- **Local build command:** `npm run build:complete` (17-phase script that auto-increments `package.json` version and produces a working DMG locally)
- **Goal:** make GitHub Actions produce the *same* installable DMG that succeeds locally.

## Workflow Changes Already Landed
1. **Workflow alignment** – Disabled legacy Tauri/Rust jobs. New `build-release.yml` / `build-binaries.yml` build the Electron DMG only and upload the artifact.
2. **ProcessManager updates** – Added logic so packaged apps:
   - fall back to Electron’s binary if `.env.production` `NODE_PATH` is missing;
   - release ports and auto-restart services exactly as described in the architecture doc.
3. **Node runtime handling** – Latest commit (`3af6ca028a`) downloads the official Node tarball during the build, copies `bin/node` into `app.asar.unpacked/.webpack/main/binaries/node`, and writes `NODE_PATH=./binaries/node` so ProcessManager launches the memory service with that bundled binary instead of the GitHub runner’s `/opt/homebrew` path.
4. **Version baseline** – `package.json`/`package-lock.json` bumped to `1.8.447` before the build; the 17-phase script auto-increments to `1.8.448` in CI for build #80.

## Observed Behaviour
| Build | Artifact | Result after install | Key log snippets |
|-------|----------|----------------------|------------------|
| #75–78 | `1.8.447` DMGs | Memory service crashes immediately | `spawn /Users/runner/hostedtoolcache/node/... ENOENT` |
| #79 | `1.8.448` DMG (fallback to bundled binary script) | Crash persists | Same ENOENT, then `memory-service exceeded max restart attempts` |
| **#80** | **`1.8.448` DMG (official Node bundled)** | **Crash persists, but now using Electron’s own binary** | `No saved Node path, using Electron's Node.js: /Applications/Hive Consensus.app/Contents/MacOS/Hive Consensus` followed by `Process memory-service exited with code 1` |

On run #80 each attempt produces `node --trace-uncaught` style output ending in:
```
FATAL ERROR: Error::ThrowAsJavaScriptException napi_throw
node_sqlite3::Statement::RowToJS(...)
```
Crash reports (`~/Library/Logs/DiagnosticReports/Hive Consensus-*.ips`) confirm the abort originates in the bundled `node_sqlite3` addon, not in ProcessManager or PortManager.

## Current Hypothesis
- **PortManager/ProcessManager are functioning**: they allocate dynamic ports, restart on crash, and log the retries as expected.
- **Failure point**: runtime abort inside `node_sqlite3` when the memory service issues its first database query. The addon we ship was built against the system/Homebrew node; when launched under the packaged runtime (Electron or the downloaded tarball), it throws `napi_fatal_error`.

## Next Steps (for the new session)
1. **Rebuild native modules** (`node_sqlite3`, any other `native_modules`) explicitly for Electron’s V8/Node version during the CI build (`npx electron-rebuild` or manual `node-gyp` targeting Electron). Confirm the rebuilt `.node` files live under `.webpack/main/native_modules/...` in the DMG.
2. **End-to-end test**: after rebuilding, reinstall the DMG on a clean macOS machine, verify the memory service reaches the `/health` endpoint, and run at least one consensus query.
3. **Version tracking**: with `package.json` baseline set to `1.8.447`, subsequent CI builds will auto-bump to `1.8.449`, `1.8.450`, etc. Keep the version bump committed whenever a release is shipped to avoid repeating the same number.

## References
- Logs: `~/Library/Application Support/Hive Consensus/logs/hive-2025-09-22T14-14-12-684Z.log`
- Crash report: `~/Library/Logs/DiagnosticReports/Hive Consensus-2025-09-22-090800.ips`
- Latest build report: `electron-poc/docs/CI_CD_STATUS.md` (this document)
- GitHub Actions run #80: https://github.com/hivetechs-collective/Hive/actions/runs/17916829015
