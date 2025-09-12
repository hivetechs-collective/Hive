export type CenterView = 'welcome' | 'help' | 'settings' | 'memory' | 'cli-tools' | 'analytics';

export interface PanelState {
  current: CenterView | null;
  last: CenterView | null;
}

// Pure state machine for center panel toggling semantics.
// Given the current state and an action (toggle a view or null to close),
// returns the next state and which target view should be visible.
export function nextStateOnToggle(
  state: PanelState,
  actionView: CenterView | null
): PanelState & { target: CenterView | null } {
  const { current, last } = state;

  // Toggle off current or explicit close
  if (actionView === null || actionView === current) {
    // Prefer returning to last if it's different; otherwise fallback to Welcome
    const fallback: CenterView | null = last && last !== current
      ? last
      : (current && current !== 'welcome')
        ? 'welcome'
        : null; // Already at Welcome; nothing else to show

    return {
      current: fallback,
      last: current || null,
      target: fallback,
    };
  }

  // Switching to a different view
  return {
    current: actionView,
    last: current,
    target: actionView,
  };
}

