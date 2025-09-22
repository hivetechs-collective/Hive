#!/usr/bin/env node
/**
 * Post-build smoke test for the Memory Service.
 * Boots the packaged memory-service entrypoint with a temporary port,
 * waits for the /health endpoint to respond, and then shuts it down.
 */

require('ts-node/register/transpile-only');

const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

const { PortManager } = require('../src/utils/PortManager');

const log = (msg) => console.log(`[smoke-memory-health] ${msg}`);
const error = (msg) => console.error(`[smoke-memory-health] ${msg}`);

const SEARCH_ROOTS = [
  path.join(__dirname, '..', '.webpack'),
  path.join(__dirname, '..', 'out')
];

function findMemoryServiceEntry() {
  const targetSuffix = path.join('memory-service', 'index.js');

  for (const root of SEARCH_ROOTS) {
    if (!fs.existsSync(root)) continue;

    const stack = [root];
    while (stack.length) {
      const current = stack.pop();
      const stat = fs.statSync(current);
      if (stat.isDirectory()) {
        const entries = fs.readdirSync(current);
        for (const entry of entries) {
          stack.push(path.join(current, entry));
        }
      } else if (stat.isFile() && current.endsWith(targetSuffix)) {
        return current;
      }
    }
  }
  return null;
}

async function waitForHealth(port, timeoutMs = 15000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(`http://127.0.0.1:${port}/health`, { signal: AbortSignal.timeout(1500) });
      if (res.ok) {
        const body = await res.json();
        if (body && body.status === 'healthy') {
          return body;
        }
      }
    } catch (err) {
      // Ignore until timeout
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  throw new Error('Timed out waiting for /health response');
}

async function main() {
  const entry = findMemoryServiceEntry();
  if (!entry) {
    error('Unable to locate memory-service entry point after build.');
    process.exit(1);
  }

  await PortManager.initialize();
  const port = await PortManager.allocatePortForService('memory-service');
  if (!port) {
    error('PortManager did not allocate a port for memory-service.');
    process.exit(1);
  }

  log(`Launching memory service from ${entry} on port ${port}`);

  const child = spawn(process.execPath, [entry], {
    env: {
      ...process.env,
      NODE_ENV: 'production',
      PORT: String(port),
      MEMORY_SERVICE_PORT: String(port)
    },
    stdio: ['ignore', 'pipe', 'pipe']
  });

  let stdout = '';
  let stderr = '';

  child.stdout.on('data', (chunk) => {
    stdout += chunk.toString();
  });

  child.stderr.on('data', (chunk) => {
    stderr += chunk.toString();
  });

  const exitPromise = new Promise((resolve, reject) => {
    child.on('exit', (code) => {
      code === 0 ? resolve() : reject(new Error(`memory-service exited with code ${code}`));
    });
    child.on('error', reject);
  });

  try {
    const payload = await waitForHealth(port);
    log(`Health check succeeded: ${JSON.stringify(payload)}`);
  } catch (err) {
    error('Health check failed. Latest stdout/stderr follow.');
    if (stdout) error(`stdout:\n${stdout}`);
    if (stderr) error(`stderr:\n${stderr}`);
    child.kill('SIGTERM');
    await exitPromise.catch(() => {});
    PortManager.releasePort('memory-service');
    process.exit(1);
  }

  child.kill('SIGTERM');
  await exitPromise.catch(() => {});
  PortManager.releasePort('memory-service');
  log('Memory service shut down cleanly.');
}

main().catch((err) => {
  error(err.message || err);
  process.exit(1);
});
