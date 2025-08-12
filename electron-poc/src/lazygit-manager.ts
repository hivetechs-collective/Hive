// LazyGit Binary Manager and Auto-Updater
// Handles downloading, updating, and managing LazyGit binaries

import { app } from 'electron';
import * as path from 'path';
import * as fs from 'fs';
import * as https from 'https';
import * as crypto from 'crypto';
import { spawn, ChildProcess } from 'child_process';

interface LazyGitRelease {
  version: string;
  assets: {
    platform: string;
    arch: string;
    url: string;
    checksum: string;
  }[];
}

interface LazyGitConfig {
  currentVersion: string;
  autoUpdate: boolean;
  updateChannel: 'stable' | 'preview';
  lastCheckTime: number;
  checkInterval: number; // milliseconds
}

export class LazyGitManager {
  private static instance: LazyGitManager;
  private config: LazyGitConfig;
  private configPath: string;
  private resourcesPath: string;
  private lazyGitProcess: ChildProcess | null = null;

  private constructor() {
    this.resourcesPath = app.isPackaged 
      ? process.resourcesPath 
      : path.join(__dirname, '../../resources');
    
    this.configPath = path.join(app.getPath('userData'), 'lazygit-config.json');
    this.config = this.loadConfig();
  }

  public static getInstance(): LazyGitManager {
    if (!this.instance) {
      this.instance = new LazyGitManager();
    }
    return this.instance;
  }

  private loadConfig(): LazyGitConfig {
    try {
      if (fs.existsSync(this.configPath)) {
        return JSON.parse(fs.readFileSync(this.configPath, 'utf8'));
      }
    } catch (error) {
      console.error('Failed to load LazyGit config:', error);
    }
    
    // Default config
    return {
      currentVersion: '0.40.2',
      autoUpdate: true,
      updateChannel: 'stable',
      lastCheckTime: 0,
      checkInterval: 86400000 // 24 hours
    };
  }

  private saveConfig(): void {
    try {
      fs.writeFileSync(this.configPath, JSON.stringify(this.config, null, 2));
    } catch (error) {
      console.error('Failed to save LazyGit config:', error);
    }
  }

  public getBinaryPath(): string {
    const platform = process.platform;
    const arch = process.arch;
    const binaryName = platform === 'win32' ? 'lazygit.exe' : 'lazygit';
    
    return path.join(this.resourcesPath, 'lazygit', `${platform}-${arch}`, binaryName);
  }

  public async ensureBinary(): Promise<boolean> {
    const binaryPath = this.getBinaryPath();
    
    // Check if binary exists
    if (fs.existsSync(binaryPath)) {
      // Make sure it's executable on Unix systems
      if (process.platform !== 'win32') {
        try {
          fs.chmodSync(binaryPath, 0o755);
        } catch (error) {
          console.error('Failed to set executable permission:', error);
        }
      }
      
      // Check for updates if auto-update is enabled
      if (this.config.autoUpdate && this.shouldCheckForUpdates()) {
        this.checkForUpdates(); // Run async, don't wait
      }
      
      return true;
    }
    
    // Binary doesn't exist, try to download it
    console.log('LazyGit binary not found, downloading...');
    return await this.downloadBinary();
  }

  private shouldCheckForUpdates(): boolean {
    const now = Date.now();
    return (now - this.config.lastCheckTime) > this.config.checkInterval;
  }

  private async checkForUpdates(): Promise<void> {
    try {
      const latestRelease = await this.fetchLatestRelease();
      
      if (this.isNewerVersion(latestRelease.version, this.config.currentVersion)) {
        console.log(`LazyGit update available: ${this.config.currentVersion} -> ${latestRelease.version}`);
        
        // Download update in background
        const success = await this.downloadBinary(latestRelease.version);
        if (success) {
          this.config.currentVersion = latestRelease.version;
          this.saveConfig();
        }
      }
      
      this.config.lastCheckTime = Date.now();
      this.saveConfig();
    } catch (error) {
      console.error('Failed to check for LazyGit updates:', error);
    }
  }

  private isNewerVersion(latest: string, current: string): boolean {
    const latestParts = latest.replace('v', '').split('.').map(Number);
    const currentParts = current.replace('v', '').split('.').map(Number);
    
    for (let i = 0; i < 3; i++) {
      if (latestParts[i] > currentParts[i]) return true;
      if (latestParts[i] < currentParts[i]) return false;
    }
    
    return false;
  }

  private async fetchLatestRelease(): Promise<LazyGitRelease> {
    return new Promise((resolve, reject) => {
      https.get({
        hostname: 'api.github.com',
        path: '/repos/jesseduffield/lazygit/releases/latest',
        headers: {
          'User-Agent': 'HiveConsensus/1.0'
        }
      }, (res) => {
        let data = '';
        res.on('data', chunk => data += chunk);
        res.on('end', () => {
          try {
            const release = JSON.parse(data);
            const assets = this.parseReleaseAssets(release);
            resolve({
              version: release.tag_name,
              assets
            });
          } catch (error) {
            reject(error);
          }
        });
      }).on('error', reject);
    });
  }

  private parseReleaseAssets(release: any): LazyGitRelease['assets'] {
    const assets: LazyGitRelease['assets'] = [];
    
    for (const asset of release.assets) {
      const name = asset.name.toLowerCase();
      
      // Map release assets to our platform/arch format
      if (name.includes('darwin') && name.includes('x86_64')) {
        assets.push({
          platform: 'darwin',
          arch: 'x64',
          url: asset.browser_download_url,
          checksum: ''
        });
      } else if (name.includes('darwin') && (name.includes('arm64') || name.includes('aarch64'))) {
        assets.push({
          platform: 'darwin',
          arch: 'arm64',
          url: asset.browser_download_url,
          checksum: ''
        });
      } else if (name.includes('windows') && name.includes('x86_64')) {
        assets.push({
          platform: 'win32',
          arch: 'x64',
          url: asset.browser_download_url,
          checksum: ''
        });
      } else if (name.includes('linux') && name.includes('x86_64')) {
        assets.push({
          platform: 'linux',
          arch: 'x64',
          url: asset.browser_download_url,
          checksum: ''
        });
      }
    }
    
    return assets;
  }

  private async downloadBinary(version?: string): Promise<boolean> {
    try {
      const release = version 
        ? await this.fetchRelease(version)
        : await this.fetchLatestRelease();
      
      const platform = process.platform;
      const arch = process.arch;
      
      const asset = release.assets.find(a => 
        a.platform === platform && a.arch === arch
      );
      
      if (!asset) {
        console.error(`No LazyGit binary found for ${platform}-${arch}`);
        return false;
      }
      
      const binaryPath = this.getBinaryPath();
      const dir = path.dirname(binaryPath);
      
      // Ensure directory exists
      if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }
      
      // Download binary
      console.log(`Downloading LazyGit from ${asset.url}`);
      await this.downloadFile(asset.url, binaryPath);
      
      // Make executable on Unix
      if (process.platform !== 'win32') {
        fs.chmodSync(binaryPath, 0o755);
      }
      
      console.log(`LazyGit ${release.version} downloaded successfully`);
      return true;
    } catch (error) {
      console.error('Failed to download LazyGit:', error);
      return false;
    }
  }

  private async fetchRelease(version: string): Promise<LazyGitRelease> {
    return new Promise((resolve, reject) => {
      https.get({
        hostname: 'api.github.com',
        path: `/repos/jesseduffield/lazygit/releases/tags/${version}`,
        headers: {
          'User-Agent': 'HiveConsensus/1.0'
        }
      }, (res) => {
        let data = '';
        res.on('data', chunk => data += chunk);
        res.on('end', () => {
          try {
            const release = JSON.parse(data);
            const assets = this.parseReleaseAssets(release);
            resolve({
              version: release.tag_name,
              assets
            });
          } catch (error) {
            reject(error);
          }
        });
      }).on('error', reject);
    });
  }

  private downloadFile(url: string, dest: string): Promise<void> {
    return new Promise((resolve, reject) => {
      // Handle GitHub redirects
      const download = (url: string) => {
        https.get(url, (response) => {
          if (response.statusCode === 302 || response.statusCode === 301) {
            // Follow redirect
            download(response.headers.location!);
            return;
          }
          
          if (response.statusCode !== 200) {
            reject(new Error(`Failed to download: ${response.statusCode}`));
            return;
          }
          
          // Check if it's a tar.gz file that needs extraction
          const contentType = response.headers['content-type'];
          const isCompressed = url.includes('.tar.gz') || url.includes('.zip');
          
          if (isCompressed) {
            // For compressed files, we need to extract
            // For now, we'll handle this separately
            console.log('Compressed file detected, extraction needed');
            // TODO: Implement extraction logic
          }
          
          const file = fs.createWriteStream(dest);
          response.pipe(file);
          
          file.on('finish', () => {
            file.close();
            resolve();
          });
          
          file.on('error', (err) => {
            fs.unlink(dest, () => {}); // Delete incomplete file
            reject(err);
          });
        }).on('error', reject);
      };
      
      download(url);
    });
  }

  public async spawnLazyGit(cwd?: string): Promise<ChildProcess> {
    // Ensure binary exists
    const binaryExists = await this.ensureBinary();
    if (!binaryExists) {
      throw new Error('LazyGit binary not available');
    }
    
    const binaryPath = this.getBinaryPath();
    
    // Kill existing process if any
    if (this.lazyGitProcess) {
      this.lazyGitProcess.kill();
      this.lazyGitProcess = null;
    }
    
    // Spawn LazyGit process
    this.lazyGitProcess = spawn(binaryPath, [], {
      cwd: cwd || process.cwd(),
      env: {
        ...process.env,
        TERM: 'xterm-256color',
        COLORTERM: 'truecolor'
      },
      shell: false
    });
    
    return this.lazyGitProcess;
  }

  public killLazyGit(): void {
    if (this.lazyGitProcess) {
      this.lazyGitProcess.kill();
      this.lazyGitProcess = null;
    }
  }
}