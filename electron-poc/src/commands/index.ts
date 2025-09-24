export type CommandHandler<T = any> = (payload?: T) => Promise<void> | void;

interface CommandEntry {
  id: string;
  handler: CommandHandler;
}

const registry = new Map<string, CommandHandler>();

export function registerCommand(id: string, handler: CommandHandler): void {
  if (!id) {
    throw new Error('Command id must be provided');
  }
  if (registry.has(id)) {
    console.warn(`[Commands] Overriding existing command handler for ${id}`);
  }
  registry.set(id, handler);
}

export function registerCommands(entries: CommandEntry[]): void {
  for (const entry of entries) {
    registerCommand(entry.id, entry.handler);
  }
}

export async function executeCommand<T = any>(id: string, payload?: T): Promise<void> {
  const handler = registry.get(id);
  if (!handler) {
    console.warn(`[Commands] No handler registered for ${id}`);
    return;
  }
  await handler(payload);
}

export function hasCommand(id: string): boolean {
  return registry.has(id);
}
