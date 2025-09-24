export function isValidRepoUrl(url: string | null | undefined): boolean {
  if (!url) return false;
  const s = url.trim();
  // HTTPS/HTTP URLs
  const httpsRe = /^(https?:\/\/)[\w.-]+\/[\w.-]+\/[\w.-]+(\.git)?$/i;
  // SSH URLs like git@github.com:owner/repo(.git)
  const sshRe = /^git@([\w.-]+):[\w.-]+\/[\w.-]+(\.git)?$/i;
  return httpsRe.test(s) || sshRe.test(s);
}

