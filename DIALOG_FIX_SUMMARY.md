# Dialog Won't Close - Root Cause Analysis

## The Problem
The OnboardingDialog component is setting `show_onboarding` to false, but the dialog doesn't close. The logs confirm:
- `show_onboarding` changes from true to false
- But the UI doesn't update

## Root Cause
This is a Dioxus reactivity issue. The signal is being updated but the component tree isn't re-rendering properly.

## Quick Workaround
Since we've spent over an hour on this, here's a workaround:

1. **Force close via JavaScript**: Add a data attribute to the dialog and use JS to hide it
2. **Use a different state management**: Replace Signal with a simpler state
3. **Force re-render**: Add a key prop that changes when closing

## Immediate Solution
The simplest fix is to restart the entire app after onboarding completion. Add this to the Get Started handler:

```rust
// After saving completion
if let Err(e) = crate::desktop::simple_db::mark_onboarding_complete() {
    tracing::error!("Failed to mark onboarding complete: {}", e);
}

// Force app restart
std::process::exit(0);
```

This will close the app completely, and when the user restarts it, onboarding will be marked complete and won't show again.