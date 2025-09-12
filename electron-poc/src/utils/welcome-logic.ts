export function shouldShowWhatsNewBadge(currentVersion: string | null | undefined, lastSeenVersion: string | null | undefined): boolean {
  if (!currentVersion) return false;
  if (!lastSeenVersion) return true;
  // Simple comparison: show if different
  return String(currentVersion).trim() !== String(lastSeenVersion).trim();
}

