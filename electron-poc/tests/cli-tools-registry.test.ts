import { CLI_TOOLS_REGISTRY } from '../src/shared/types/cli-tools';

function assert(cond: any, msg: string) { if (!cond) throw new Error(msg); }

(async () => {
  const ids = ['claude-code','gemini-cli','qwen-code','openai-codex','grok','specify'];
  for (const id of ids) {
    const cfg = (CLI_TOOLS_REGISTRY as any)[id];
    assert(cfg, `Missing tool config for ${id}`);
    assert(typeof cfg.name === 'string' && cfg.name.length > 0, `${id}: missing name`);
    assert(typeof cfg.command === 'string' && cfg.command.length > 0, `${id}: missing command`);
  }
  console.log('[cli-tools-registry.test] OK');
})();

