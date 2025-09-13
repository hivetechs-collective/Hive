import { nextStateOnToggle, type CenterView, type PanelState } from './panel-state';

// Thin integration layer around the pure state machine.
// - Adds idempotentFocus behavior: clicking the same view returns the same state and targets the same view.
// - Otherwise, defers to nextStateOnToggle.
export function applyCenterView(
  state: PanelState,
  actionView: CenterView | null,
  opts: { idempotentFocus?: boolean } = { idempotentFocus: true }
): PanelState & { target: CenterView | null } {
  const { idempotentFocus = true } = opts;
  if (idempotentFocus && actionView !== null && state.current === actionView) {
    return { current: state.current, last: state.last, target: actionView };
  }
  return nextStateOnToggle(state, actionView);
}

export type { CenterView, PanelState };

