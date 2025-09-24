export type BackupFrequency = 'manual' | 'on-exit' | 'daily' | 'weekly';

export function isBackupDue(lastIso: string | null | undefined, freq: BackupFrequency, now: Date = new Date()): boolean {
  if (freq === 'manual') return false;
  if (!lastIso) return true; // never backed up -> due
  const last = new Date(lastIso);
  if (isNaN(last.getTime())) return true;
  if (freq === 'on-exit') return true; // decided at exit
  const msInDay = 24 * 60 * 60 * 1000;
  const diff = now.getTime() - last.getTime();
  if (freq === 'daily') return diff >= msInDay;
  if (freq === 'weekly') return diff >= 7 * msInDay;
  return false;
}

export function sortByMtimeDesc(files: Array<{ name: string; mtimeMs: number }>) {
  return files.sort((a, b) => b.mtimeMs - a.mtimeMs);
}

export function computeRetentionDeletes(files: Array<{ name: string; mtimeMs: number }>, retention: number): string[] {
  if (retention <= 0) return files.map(f => f.name);
  const sorted = sortByMtimeDesc([...files]);
  return sorted.slice(retention).map(f => f.name);
}

