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

  // Explicit close request - return to last / welcome fallback
  if (actionView === null) {
    const fallback: CenterView | null = last && last !== current
      ? last
      : current && current !== 'welcome'
        ? 'welcome'
        : null;

    return {
      current: fallback,
      last: current || last || null,
      target: fallback,
    };
  }

  // Toggle off the current view
  if (actionView === current) {
    const updatedLast = current ?? last ?? null;
    return {
      current: null,
      last: updatedLast,
      target: null,
    };
  }

  // Switching to a different view
  return {
    current: actionView,
    last: current,
    target: actionView,
  };
}
