#!/usr/bin/env node

/**
 * Runs the Playwright UI smoke suite against the packaged Electron build.
 * - Verifies a packaged binary exists (build once via `npm run build:complete`).
 * - Allocates a remote debugging port so tests can attach.
 * - Allows optional attach mode (reuse a manually launched instance).
 */

const fs = require('fs');
const net = require('net');
const path = require('path');
const { spawn, spawnSync } = require('child_process');

const PROJECT_ROOT = path.resolve(__dirname, '..');
const APP_NAME = 'Hive Consensus';

function candidatesForPlatform() {
  const outDir = path.join(PROJECT_ROOT, 'out');
  const candidates = [];

  if (process.platform === 'darwin') {
    candidates.push(
      path.join(
        '/Applications',
        `${APP_NAME}.app`,
        'Contents',
        'MacOS',
        APP_NAME,
      ),
    );
    candidates.push(
      path.join(
        outDir,
        `${APP_NAME}-darwin-arm64`,
        `${APP_NAME}.app`,
        'Contents',
        'MacOS',
        APP_NAME,
      ),
    );
    candidates.push(
      path.join(
        outDir,
        `${APP_NAME}-darwin-x64`,
        `${APP_NAME}.app`,
        'Contents',
        'MacOS',
        APP_NAME,
      ),
    );
  } else if (process.platform === 'win32') {
    const exe = `${APP_NAME}.exe`;
    candidates.push(path.join(outDir, `${APP_NAME}-win32-x64`, exe));
    if (process.env.ProgramFiles) {
      candidates.push(path.join(process.env.ProgramFiles, APP_NAME, exe));
    }
  } else {
    const binary = APP_NAME.toLowerCase().replace(/\s+/g, '-');
    candidates.push(path.join(outDir, `${APP_NAME}-linux-x64`, binary));
    candidates.push(path.join('/opt', binary, binary));
  }

  return candidates;
}

function resolvePackagedExecutable() {
  const envPath = process.env.ELECTRON_APP_PATH;
  if (envPath) {
    const absolute = path.isAbsolute(envPath)
      ? envPath
      : path.resolve(PROJECT_ROOT, envPath);
    if (fs.existsSync(absolute)) {
      return absolute;
    }
  }

  for (const candidate of candidatesForPlatform()) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  return null;
}

function ensurePackagedBinary() {
  const executable = resolvePackagedExecutable();
  if (!executable) {
    console.error(
      '\n✗ Packaged app not found. Run `npm run build:complete` before `npm run test:ui`.\n',
    );
    process.exit(1);
  }

  return executable;
}

function findAvailablePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.unref();
    server.on('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const address = server.address();
      server.close((closeErr) => {
        if (closeErr) {
          reject(closeErr);
        } else {
          resolve(address.port);
        }
      });
    });
  });
}

function allocatePortWithPortManager() {
  try {
    const tsNodeRegister = require.resolve('ts-node/register/transpile-only');
    const inlineScript = [
      "const path = require('path');",
      "const { PortManager } = require(path.resolve(process.cwd(), 'src/utils/PortManager'));",
      '(async () => {',
      '  try {',
      '    await PortManager.initialize();',
      "    const port = await PortManager.allocatePortForService('playwright-remote-debug');",
      '    if (!port) throw new Error("PortManager returned invalid port");',
      '    process.stdout.write(String(port));',
      '    process.exit(0);',
      '  } catch (error) {',
      '    console.error(error && error.stack ? error.stack : String(error));',
      '    process.exit(1);',
      '  }',
      '})();',
    ].join('\n');

    const result = spawnSync(process.execPath, ['-r', tsNodeRegister, '-e', inlineScript], {
      cwd: PROJECT_ROOT,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    if (result.status === 0) {
      const candidate = parseInt(result.stdout.trim(), 10);
      if (Number.isInteger(candidate) && candidate > 0) {
        return candidate;
      }
    }
  } catch (error) {
    // ts-node may not be available or PortManager import failed – fallback below.
  }

  return null;
}

async function main() {
  const cliArgs = process.argv.slice(2);
  const attachFlagIndex = cliArgs.indexOf('--attach');
  const attachMode = attachFlagIndex !== -1 || process.env.PLAYWRIGHT_ATTACH === '1';
  const filteredArgs = attachFlagIndex !== -1
    ? cliArgs.filter((_, idx) => idx !== attachFlagIndex)
    : cliArgs;

  const executable = ensurePackagedBinary();
  const env = { ...process.env };

  if (attachMode) {
    env.PLAYWRIGHT_ATTACH = '1';
    if (!env.PLAYWRIGHT_REMOTE_DEBUG_PORT) {
      console.error(
        '\n✗ PLAYWRIGHT_REMOTE_DEBUG_PORT must be set when using --attach or PLAYWRIGHT_ATTACH=1.',
      );
      console.error(
        '  Launch the packaged app with PLAYWRIGHT_E2E=1 and the same port before running the suite.\n',
      );
      process.exit(1);
    }
  } else {
    if (!env.PLAYWRIGHT_REMOTE_DEBUG_PORT) {
      const portFromManager = allocatePortWithPortManager();
      if (portFromManager) {
        env.PLAYWRIGHT_REMOTE_DEBUG_PORT = String(portFromManager);
      } else {
        const port = await findAvailablePort();
        env.PLAYWRIGHT_REMOTE_DEBUG_PORT = String(port);
      }
    }
  }

  env.PLAYWRIGHT_E2E = '1';
  env.ELECTRON_APP_PATH = executable;

  console.log(`⚙️  Using packaged app at: ${executable}`);
  console.log(
    `🔌 Remote debugging port: ${env.PLAYWRIGHT_REMOTE_DEBUG_PORT} (${attachMode ? 'attach' : 'launch'} mode)`,
  );

  const npxExecutable = process.platform === 'win32' ? 'npx.cmd' : 'npx';
  const child = spawn(npxExecutable, ['playwright', 'test', ...filteredArgs], {
    cwd: PROJECT_ROOT,
    env,
    stdio: 'inherit',
  });

  child.on('exit', (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
    } else {
      process.exit(code ?? 1);
    }
  });
}

main().catch((err) => {
  console.error('\nUnexpected error while running UI tests:', err);
  process.exit(1);
});
