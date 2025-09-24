# Hive IDE – Conversation Context Summary (Welcome v1.9 + Backup/DB)

Last updated: v1.8.345 (auto-installed to /Applications)

## Overview
- Focus: Improving the Welcome page (developer-first), DB-backed preferences/recents, and production-grade backup/restore.
- Completed Phases for Welcome v1.9: 1–4 (core actions, high-value flows, continuity/onboarding, layout + analytics).
- Added unified DB best practices and robust Backup/Restore with auto-backup, encryption, compression, and a backups manager.

## IPC Endpoints (Renderer ⇄ Main)
- Database: `db-get-setting`, `db-set-setting`, `db-save-session`, `db-load-session`, `db-clear-session`
- Recents: `db-add-recent-folder`, `db-get-recent-folders`, `db-remove-recent-folder`, `db-clear-recent-folders`
- Welcome analytics: `db-welcome-analytics-log`
- DB maintenance: `db-compact`, `db-integrity-check`
- Backup/Restore: `db-backup`, `db-restore`, `list-backups`, `delete-backup`, `reveal-in-folder`, `open-path`
- OS dialogs/links: `show-open-dialog`, `show-save-dialog`, `show-message-box`, `open-external`

Why IPC: keeps the UI sandboxed and routes all privileged work (DB, FS, backups) through validated handlers in the main process for consistency, safety, and auditability.

## Welcome Page (v1.9)
Implementation: `src/components/welcome-page.ts`

- Start actions:
  - New Project dialog (template selection: Node/Python/Rust/Empty, name/location picker, optional Git init). Uses `src/utils/template-scaffold.ts`.
  - Open Folder (native dialog).
  - Clone Repository dialog: URL | GitHub | GitLab providers; URL validation in `src/utils/clone-validate.ts`.
  - Open Terminal… (pick a folder; opens terminal via terminal API).

- Recents:
  - Inline list (up to 20). DB-backed from `recent_folders` or legacy fallback key.
  - Open Recent ▾ (top 10 with remove). Clear all.
  - Show All modal with search; per-row actions: Open, Restore (via open), Terminal, Remove.
  - Drag-and-drop open removed (mac UX simplification).

- Learn:
  - Keyboard Shortcuts (inline modal).
  - AI Workflows and Documentation links.
  - What’s New badge shows only when app version changes; click clears (`welcome.lastSeenVersion`).

- Continuity & onboarding:
  - Restore Session button shows when most recent folder has saved tabs.
  - Basics Tour prompt (one-time) → Getting Started; persists `welcome.tourSeen`.

- Layout modes:
  - Footer toggle cycles minimal/balanced/full; persisted as `welcome.layoutMode`.
  - Logic in `src/utils/welcome-layout.ts`. Learn column hidden in minimal.

- Analytics events: stored in `welcome_analytics` table via IPC/logging from Welcome UI (e.g., `click_recent`, `open_recents_modal`, `clear_recents`, `restore_session`, `open_shortcuts`, `layout_toggle_*`, `clone_success`, `clone_fail`, `create_template_*`).

Renderer integration highlights: `src/renderer.ts`
- Ensures when showing Welcome/Help/Analytics, other center panels (Settings, Memory, CLI Tools) are hidden first.
- Fix applied to hide Settings when showing other panels to avoid stale visible state.

## Unified Database Strategy (Production)
- Single ACID SQLite DB at `~/.hive/hive-ai.db`.
- PRAGMAs on init: `foreign_keys=ON`, `journal_mode=WAL`, `synchronous=NORMAL`, `busy_timeout=5000`.
- HIVE_DB_PATH override for tests/tools. DB parent dir creation included.
- Tables used by Welcome: `settings`, `recent_folders`, `sessions`, `welcome_analytics`.

Environment variables:
- `HIVE_DB_PATH`: override DB path (testing, tooling)
- `HIVE_BACKUP_DIR`: override default backup directory

## Backup & Restore
Implementation (IPC/UI): `src/index.ts`, `src/preload.ts`, `src/settings-modal.ts`

- Manual backup: WAL checkpoint + `VACUUM INTO` to snapshot file; UI allows encryption (AES‑256‑GCM) and compression (gzip).
  - Formats: `.sqlite` (plain), `.sqlite.gz` (compressed). Encrypted backups embed magic headers `HIVEENC1` (encrypted) or `HIVEENC2` (encrypted+compressed) in the file content; the extension does not change automatically.
- Restore: Detects format → decrypt/decompress when necessary → integrity_check → replace DB → re-init.
- Auto-backup:
  - Manual/On Exit/Daily/Weekly; user-selected backup folder (`backup.dir`), retention count, “always encrypt/compress.”
  - Reminder when disabled with snooze: Enable Auto / Backup Now / Remind Me Later / Dismiss.
- Backups Manager (Settings > Advanced): list/search backups; reveal, restore, delete; open backup folder.

Encryption details:
- AES-256-GCM using `scrypt` KDF with 16-byte salt, 12-byte IV, 16-byte auth tag.
- File layout: `[header:8][salt:16][iv:12][tag:16][ciphertext]`. For `HIVEENC2`, plaintext is gzip-compressed before encryption.

Notes and caveats:
- Auto-backup retention keeps the N most recent `.sqlite` and `.sqlite.gz` backups (default N=7).
- Backups Manager lists `.sqlite` and `.sqlite.gz`; encrypted backups are listed based on filename extension the user chose.

DB keys (backup):
- `backup.autoEnabled` ('1'|'0'), `backup.frequency` ('manual'|'on-exit'|'daily'|'weekly'),
- `backup.retentionCount`, `backup.dir`, `backup.alwaysEncrypt`, `backup.alwaysCompress`,
- `backup.lastBackupAt`, `backup.reminderDays`, `backup.snoozeUntil`.

## Tests
Runner: `npm run test:welcome`
- Type checks.
- DB round-trip (settings/recents/sessions): `scripts/test-welcome-db.js`.
- Welcome logic: `tests/welcome-logic.test.ts` (version badge).
- Template scaffolds: `tests/template-scaffold.test.ts`.
- Clone URL validation: `tests/clone-validate.test.ts`.
- Layout computation: `tests/welcome-layout.test.ts`.
- Backup policy: `tests/backup-policy.test.ts`.

## Build
- Full production pipeline (`npm run build:complete`) passes and auto-installs the app (v1.8.345).
- Optional ws native modules warn but are non-blocking (bufferutil, utf-8-validate).

## Primary files touched
- Welcome UI: `src/components/welcome-page.ts`, `src/renderer.ts`.
- Utils: `src/utils/template-scaffold.ts`, `src/utils/clone-validate.ts`, `src/utils/welcome-layout.ts`, `src/utils/backup-policy.ts`.
- DB/IPC: `src/index.ts`, `src/preload.ts`.
- Settings UI: `src/settings-modal.ts`.
- Docs: `MASTER_ARCHITECTURE.md`.

## Outstanding polish (optional)
- Pin recents; Open in new window (multi-window infra).
- Additional clone providers (Azure DevOps) and token-aware shortcuts.
- Centralize “hide all center panels” into a utility for extra safety.

Known gaps now addressed:
- Docs clarified for IPC endpoints, env vars, and encryption layout.
- Backups Manager now includes `.sqlite.gz` files in listing.

## How to run
- Dev: `cd electron-poc && npm start`
- Tests: `npm run test:welcome`
- Build: `npm run build:complete` (produces DMG and auto-installs)
