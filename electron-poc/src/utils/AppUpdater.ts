import { app, dialog, shell } from 'electron';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';
import * as https from 'https';
import { createHash } from 'crypto';
import { logger } from '../utils/SafeLogger';

type VersionInfo = {
  version: string;
  channel: string;
  platform: string;
  arch: string;
  url: string;          // Versioned DMG URL
  size?: number;        // Bytes
  size_mb?: string;     // MiB (string)
  sha256?: string;      // Optional checksum (hex)
  date?: string;        // ISO timestamp
};

function compareSemver(a: string, b: string): number {
  const pa = a.split('.').map(n => parseInt(n, 10) || 0);
  const pb = b.split('.').map(n => parseInt(n, 10) || 0);
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const da = pa[i] || 0, db = pb[i] || 0;
    if (da > db) return 1;
    if (da < db) return -1;
  }
  return 0;
}

export class AppUpdater {
  private static instance: AppUpdater | null = null;
  private checking = false;
  private sessionIgnoredVersion: string | null = null;
  private lastCheckedAt = 0;
  private intervalMs = 24 * 60 * 60 * 1000; // 24 hours
  private timer: NodeJS.Timeout | null = null;

  static getInstance(): AppUpdater {
    if (!this.instance) this.instance = new AppUpdater();
    return this.instance;
  }

  startAutoChecks(channel: 'stable' | 'beta' = 'stable') {
    // Check once shortly after launch
    setTimeout(() => this.checkForUpdates(channel).catch(() => {}), 5_000);
    // Then every 24h while running
    if (this.timer) clearInterval(this.timer);
    this.timer = setInterval(() => {
      this.checkForUpdates(channel).catch(() => {});
    }, this.intervalMs);
  }

  async checkForUpdates(channel: 'stable' | 'beta' = 'stable', opts: { force?: boolean } = {}): Promise<void> {
    if (this.checking) return;
    this.checking = true;
    try {
      // Use Homebrew tap cask as the authoritative version source
      // Cask: https://github.com/hivetechs-collective/homebrew-tap/blob/main/Casks/hive-consensus.rb
      const info = await this.fetchVersionFromTap();
      if (!info?.version || !info?.url) {
        logger.warn('[Updater] Invalid version info from tap');
        return;
      }

      const current = app.getVersion();
      const cmp = compareSemver(info.version, current);
      logger.info(`[Updater] Current ${current}, Latest ${info.version}, cmp=${cmp}`);

      if (cmp <= 0) {
        if (opts.force) {
          await dialog.showMessageBox({
            type: 'info',
            message: 'You are up to date',
            detail: `Hive Consensus ${current} is the latest version.`,
            buttons: ['OK'],
            defaultId: 0,
          });
        }
        return;
      }

      if (!opts.force && this.sessionIgnoredVersion === info.version) {
        logger.info('[Updater] Latest version already ignored this session');
        return;
      }

      // Prompt user
      const buttons = ['Download and Install', 'Later', 'Release Notes'];
      const res = await dialog.showMessageBox({
        type: 'info',
        buttons,
        defaultId: 0,
        cancelId: 1,
        title: 'Update Available',
        message: `A new version of Hive Consensus is available (${info.version}).`,
        detail: 'You can install the update now. The app will quit and relaunch after installation.',
        noLink: true,
      });

      if (res.response === 2) {
        // Open GitHub Release page for this version
        const tagUrl = `https://github.com/hivetechs-collective/Hive/releases/tag/v${info.version}`;
        try { await shell.openExternal(tagUrl); } catch {}
        return; // Keep prompt minimal; user can invoke manual check again
      }

      if (res.response === 1) {
        // Later — ignore just for this session
        this.sessionIgnoredVersion = info.version;
        this.lastCheckedAt = Date.now();
        return;
      }

      // Download and install
      await this.downloadAndInstall(info);
    } catch (e: any) {
      logger.error('[Updater] Check failed', e?.message || e);
      if (opts.force) {
        await dialog.showMessageBox({
          type: 'error',
          message: 'Update check failed',
          detail: String(e?.message || e),
          buttons: ['OK'],
        });
      }
    } finally {
      this.checking = false;
    }
  }

  private fetchJSON<T>(url: string): Promise<T> {
    return new Promise((resolve, reject) => {
      https
        .get(url, (res) => {
          if (res.statusCode && res.statusCode >= 400) {
            return reject(new Error(`HTTP ${res.statusCode}`));
          }
          let data = '';
          res.setEncoding('utf8');
          res.on('data', (chunk) => (data += chunk));
          res.on('end', () => {
            try { resolve(JSON.parse(data)); } catch (e) { reject(e); }
          });
        })
        .on('error', reject);
    });
  }

  private fetchText(url: string): Promise<string> {
    return new Promise((resolve, reject) => {
      https
        .get(url, (res) => {
          if (res.statusCode && res.statusCode >= 400) {
            return reject(new Error(`HTTP ${res.statusCode}`));
          }
          let data = '';
          res.setEncoding('utf8');
          res.on('data', (chunk) => (data += chunk));
          res.on('end', () => resolve(data));
        })
        .on('error', reject);
    });
  }

  private async fetchVersionFromTap(): Promise<VersionInfo> {
    // Read the cask file from the tap repository and parse version/url/sha256
    // Default branch is main
    const caskUrl = 'https://raw.githubusercontent.com/hivetechs-collective/homebrew-tap/main/Casks/hive-consensus.rb';
    const content = await this.fetchText(caskUrl);

    const versionMatch = content.match(/version\s+"([^"]+)"/);
    const shaMatch = content.match(/sha256\s+"([a-fA-F0-9]{64})"/);
    const urlMatch = content.match(/url\s+"([^"]+)"/);

    if (!versionMatch || !(urlMatch || versionMatch)) {
      throw new Error('Failed to parse cask for version/url');
    }

    const version = versionMatch[1];
    let url = urlMatch ? urlMatch[1] : '';
    if (url.includes('#{version}')) {
      url = url.replace(/#\{version\}/g, version);
    }
    // Fallback if url not found: construct from GitHub tag
    if (!url || !/^https?:\/\//.test(url)) {
      url = `https://github.com/hivetechs-collective/Hive/releases/download/v${version}/Hive.Consensus.dmg`;
    }

    const info: VersionInfo = {
      version,
      channel: 'tap',
      platform: process.platform,
      arch: process.arch,
      url,
      sha256: shaMatch ? shaMatch[1] : undefined,
    };
    return info;
  }

  private async downloadAndInstall(info: VersionInfo): Promise<void> {
    const dmgPath = path.join(os.tmpdir(), `Hive-Consensus-${info.version}.dmg`);
    await this.downloadFile(info.url, dmgPath, info.sha256);

    const choice = await dialog.showMessageBox({
      type: 'question',
      buttons: ['Install and Relaunch', 'Cancel'],
      defaultId: 0,
      cancelId: 1,
      message: 'Ready to install update',
      detail: 'Hive will quit now and install the update into Applications, then relaunch.',
      noLink: true,
    });
    if (choice.response !== 0) return;

    // Spawn detached installer and quit
    const script = `
set -e
DMG="$1"
MOUNT_POINT=""
attach_out=$(hdiutil attach "$DMG" -nobrowse | tail -n1 || true)
mp=$(echo "$attach_out" | awk '{print $3}')
if [ -d "$mp" ]; then MOUNT_POINT="$mp"; else MOUNT_POINT="/Volumes/Hive Consensus"; fi

# Stop old app and replace
pkill -f "Hive Consensus" || true
sleep 1
rm -rf "/Applications/Hive Consensus.app" || true
cp -Rf "$MOUNT_POINT/Hive Consensus.app" /Applications/
hdiutil detach "$MOUNT_POINT" || true

# Relaunch
open -a "Hive Consensus"
`;

    const { spawn } = require('child_process');
    const child = spawn('bash', ['-lc', script, '--', dmgPath], {
      detached: true,
      stdio: 'ignore',
    });
    child.unref();

    app.quit();
  }

  private downloadFile(url: string, dest: string, sha256?: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const file = fs.createWriteStream(dest);
      const hash = createHash('sha256');
      https.get(url, (res) => {
        if (res.statusCode && res.statusCode >= 400) {
          return reject(new Error(`HTTP ${res.statusCode}`));
        }
        res.on('data', (chunk) => hash.update(chunk));
        res.pipe(file);
        file.on('finish', () => {
          file.close(() => {
            try {
              const digest = hash.digest('hex');
              if (sha256 && digest.toLowerCase() !== sha256.toLowerCase()) {
                return reject(new Error('SHA256 mismatch for downloaded DMG'));
              }
              resolve();
            } catch (e) { reject(e); }
          });
        });
      }).on('error', (err) => {
        try { file.close(); } catch {}
        reject(err);
      });
    });
  }
}
