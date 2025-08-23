# TTYD Terminal Size Fix Test

## Test Scenarios

### 1. Initial Load Test
- Open a new TTYD terminal tab
- Check console for "Container dimensions" log
- Should NOT show 37x9
- Terminal should fill the panel properly

### 2. Window Minimize/Maximize Test
- Open TTYD terminal
- Minimize the window
- Maximize the window
- Terminal should reload and maintain full size

### 3. Center Panel Collapse Test  
- Open TTYD terminal with center panel open
- Close/collapse the center panel
- TTYD should expand to fill available space
- Terminal should reload and use full dimensions

## Expected Console Output
```
[TTYDTerminalPanel] Webview DOM ready, ensuring proper size
[TTYDTerminalPanel] Container dimensions: 400x600 (or similar large values)
[TTYDTerminalPanel] Reloading webview to fix terminal size
```

## Fixed Issues
1. Added dom-ready event handler to detect and fix tiny dimensions
2. Force reload webview when dimensions < 200px
3. Properly set webview position to absolute with full coverage
4. Reload on expand-to-fill class changes
5. Reload after window state changes

## If Issue Persists
Check:
- Container actual dimensions in DevTools
- Whether webview src is being properly reloaded
- ttyd process arguments (should have no size restrictions)