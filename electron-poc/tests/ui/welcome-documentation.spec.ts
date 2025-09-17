import fs from 'fs';
import path from 'path';
import { execSync, spawn, ChildProcess } from 'child_process';
import fetch from 'node-fetch';
import { Browser, BrowserContext, Page, chromium, expect, test } from '@playwright/test';

const APP_ROOT = path.resolve(__dirname, '..', '..');
const ATTACH_MODE = process.env.PLAYWRIGHT_ATTACH === '1';
const REMOTE_DEBUG_PORT = process.env.PLAYWRIGHT_REMOTE_DEBUG_PORT;
const INSTALLED_EXECUTABLE = path.resolve(
  '/Applications',
  'Hive Consensus.app',
  'Contents',
  'MacOS',
  'Hive Consensus',
);
const PACKAGED_EXECUTABLE = path.resolve(
  APP_ROOT,
  'out',
  'Hive Consensus-darwin-arm64',
  'Hive Consensus.app',
  'Contents',
  'MacOS',
  'Hive Consensus',
);
const FALLBACK_EXECUTABLES = [
  path.resolve(
    APP_ROOT,
    'out',
    'Hive Consensus-darwin-x64',
    'Hive Consensus.app',
    'Contents',
    'MacOS',
    'Hive Consensus',
  ),
];

function resolveExecutablePath(): string {
  const envPath = process.env.ELECTRON_APP_PATH;
  if (envPath) {
    const resolved = path.isAbsolute(envPath)
      ? envPath
      : path.resolve(APP_ROOT, envPath);
    if (fs.existsSync(resolved)) return resolved;
  }
  if (fs.existsSync(INSTALLED_EXECUTABLE)) return INSTALLED_EXECUTABLE;
  if (fs.existsSync(PACKAGED_EXECUTABLE)) return PACKAGED_EXECUTABLE;
  for (const candidate of FALLBACK_EXECUTABLES) {
    if (fs.existsSync(candidate)) return candidate;
  }
  throw new Error(
    'Packaged app not found. Run `npm run build:complete` first or set ELECTRON_APP_PATH.',
  );
}

function ensureAppNotQuarantined(executablePath: string) {
  if (process.platform !== 'darwin') return;

  const appBundle = path.resolve(executablePath, '../../..');
  try {
    execSync(`xattr -dr com.apple.quarantine "${appBundle}"`, {
      stdio: 'ignore',
    });
  } catch (err) {
    console.warn('[test] failed to clear quarantine (may be harmless)', err);
  }
}

async function resolveDebuggerEndpoint(port: number, timeoutMs = 45_000): Promise<string> {
  const endpointOverride = process.env.PLAYWRIGHT_WS_ENDPOINT;
  if (endpointOverride) return endpointOverride;

  const url = `http://127.0.0.1:${port}/json/version`;
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    try {
      const response = await fetch(url, { timeout: 2_000 });
      if (response.ok) {
        const payload = await response.json();
        if (payload?.webSocketDebuggerUrl) {
          return payload.webSocketDebuggerUrl as string;
        }
      }
    } catch (err) {
      // Expected until the Electron runtime exposes the endpoint.
    }
    await new Promise((resolve) => setTimeout(resolve, 1_000));
  }

  throw new Error(
    `Timed out waiting for remote debugging endpoint on port ${port}. ` +
      'Did you launch the packaged app with PLAYWRIGHT_E2E=1 and the same port?',
  );
}

test.describe('Welcome documentation links', () => {
  test.describe.configure({ timeout: 180_000 });

  let launchedProcess: ChildProcess | null = null;
  let browser: Browser | null = null;
  let context: BrowserContext | null = null;
  let page: Page | null = null;

  test.beforeAll(async () => {
    if (!REMOTE_DEBUG_PORT) {
      throw new Error('PLAYWRIGHT_REMOTE_DEBUG_PORT must be set before running the UI suite.');
    }

    const port = Number(REMOTE_DEBUG_PORT);
    if (!Number.isInteger(port)) {
      throw new Error(`Invalid PLAYWRIGHT_REMOTE_DEBUG_PORT value: ${REMOTE_DEBUG_PORT}`);
    }

    const executablePath = resolveExecutablePath();
    ensureAppNotQuarantined(executablePath);

    if (!ATTACH_MODE) {
      const env = {
        ...process.env,
        PLAYWRIGHT_E2E: '1',
        PLAYWRIGHT_REMOTE_DEBUG_PORT: String(port),
        ELECTRON_ENABLE_LOGGING: '1',
      } as NodeJS.ProcessEnv;

      launchedProcess = spawn(executablePath, [], {
        env,
        stdio: ['ignore', 'pipe', 'pipe'],
      });

      launchedProcess.stdout?.on('data', (data) => {
        process.stdout.write(`[app stdout] ${data}`);
      });
      launchedProcess.stderr?.on('data', (data) => {
        process.stderr.write(`[app stderr] ${data}`);
      });
      launchedProcess.once('exit', (code, signal) => {
        console.log(`[test] launched app exited (code=${code}, signal=${signal})`);
      });

      // Give the process a moment to boot before polling the debugger endpoint.
      await new Promise((resolve) => setTimeout(resolve, 1_000));
    }

    const wsEndpoint = await resolveDebuggerEndpoint(port);
    browser = await chromium.connectOverCDP(wsEndpoint);
    const contexts = browser.contexts();
    context = contexts.length > 0 ? contexts[0] : await browser.newContext();

    const acquirePage = async (): Promise<Page> => {
      const start = Date.now();
      while (Date.now() - start < 120_000) {
        const candidates = context!.pages().filter((candidate) => !candidate.isClosed());
        for (const candidate of candidates) {
          try {
            await candidate.waitForLoadState('domcontentloaded', { timeout: 5_000 });
            await candidate.waitForFunction(
              () => Boolean((window as any).commandAPI?.executeCommand),
              undefined,
              { timeout: 2_000 },
            );
            return candidate;
          } catch (err) {
            if (candidate.isClosed()) {
              continue;
            }
          }
        }

        await Promise.race([
          context!.waitForEvent('page').catch(() => null),
          new Promise((resolve) => setTimeout(resolve, 1_000)),
        ]);
      }

      throw new Error('Timed out waiting for renderer window with commandAPI');
    };

    page = await acquirePage();
  });

  test.afterAll(async () => {
    if (browser) {
      await browser.close();
      browser = null;
      context = null;
      page = null;
    }

    if (launchedProcess && !ATTACH_MODE) {
      try {
        launchedProcess.kill();
      } catch (err) {
        console.warn('[test] failed to kill launched process', err);
      }
      launchedProcess = null;
    }
  });

  async function showWelcomePanel() {
    await page!.evaluate(() =>
      (window as any).commandAPI?.executeCommand('view.welcome.open'),
    );
    await page!.waitForSelector('#welcome-panel', {
      state: 'visible',
      timeout: 30_000,
    });
  }

  async function showHelpSection(section: string) {
    await page!.evaluate((targetSection) =>
      (window as any).commandAPI?.executeCommand('view.help.open', {
        section: targetSection,
        forceFocus: true,
      }),
    section,
    );
    await page!.waitForSelector('#help-panel', {
      state: 'visible',
      timeout: 30_000,
    });
  }

  test('AI Workflows button opens Help section', async () => {
    await showWelcomePanel();

    await page!.click('#workflows-btn');
    await showHelpSection('ai-workflows');

    const activeNav = page!.locator(
      '#help-panel .help-nav-item[data-section="ai-workflows"]',
    );
    await expect(activeNav).toHaveClass(/active/);
    await expect(
      page!.locator('#help-panel .help-content-inner'),
    ).toContainText(/AI Workflows/);
  });

  test("What's New button opens Help section", async () => {
    await showWelcomePanel();

    await page!.click('#whats-new-btn');
    await showHelpSection('whats-new');

    const activeNav = page!.locator(
      '#help-panel .help-nav-item[data-section="whats-new"]',
    );
    await expect(activeNav).toHaveClass(/active/);
    await expect(
      page!.locator('#help-panel .help-content-inner'),
    ).toContainText(/Hive v1\.8/);
  });
});
