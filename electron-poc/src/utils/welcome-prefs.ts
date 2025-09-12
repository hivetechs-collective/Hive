// Unified logic for interpreting the welcome.showOnStartup setting
// - Missing/null/undefined => show (default on)
// - '1' => show
// - '0' => hide
export function shouldShowOnStartup(value: string | null | undefined): boolean {
  if (value == null) return true;
  return String(value) === '1';
}

