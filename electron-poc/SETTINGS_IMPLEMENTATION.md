# Settings Modal - API Key Masking Implementation

## Overview
The settings modal now properly loads existing API keys from the SQLite database and displays them in a masked format, matching the behavior of the original Dioxus GUI.

## Key Features Implemented

### 1. Masked API Key Display
- When settings modal opens, existing API keys are loaded from the database
- Keys are displayed masked showing first 6 and last 4 characters
- Example: `sk-or-••••••••••••wxyz` for OpenRouter key
- Example: `hive-x••••••••••1234` for Hive license key

### 2. Toggle Visibility
- Eye button toggles between masked and full key display
- When shown: displays the complete API key
- When hidden: returns to masked display
- Icon changes between 👁 (hidden) and 👁‍🗨 (visible)

### 3. Smart Input Handling
- Stored keys are kept in `data-actual-key` attribute
- When user types new value, the masked state is cleared
- Distinguishes between existing masked keys and new user input

### 4. Database Integration
- Loads keys from SQLite database at: `~/Library/Application Support/electron-poc/hive_unified.db`
- Configuration table stores:
  - `openrouter_api_key`: OpenRouter API key
  - `hive_license_key`: Hive license key
  - Profile selection and other settings

### 5. Save Functionality
- Preserves existing keys when saving if not modified
- Only saves actual key values, not masked versions
- Updates display to show masked version after successful save

## Technical Implementation

### Data Attributes
- `data-actual-key`: Stores the real API key value
- `data-masked`: Indicates if the displayed value is masked ("true"/"false")

### Masking Function
```typescript
private maskApiKey(key: string): string {
  if (!key || key.length < 8) return key;
  
  // Show first 6 characters and last 4 characters
  const firstPart = key.substring(0, 6);
  const lastPart = key.substring(key.length - 4);
  const maskedMiddle = '•'.repeat(Math.min(key.length - 10, 20));
  
  return `${firstPart}${maskedMiddle}${lastPart}`;
}
```

## Testing
Use the test script to add sample API keys to the database:
```bash
node test-settings.js
```

This will add:
- OpenRouter key: `sk-or-v1-1234567890abcdefghijklmnopqrstuvwxyz`
- Hive key: `hive-xxxx-yyyy-zzzz-1234`

## User Experience
1. User opens settings → sees masked API keys if they exist
2. Clicks eye button → reveals full key
3. Can edit key directly → automatically detects new input
4. Saves settings → key is stored and displayed masked
5. Test button → uses actual key value (not masked) for validation

This implementation provides security by not displaying full API keys by default while still allowing users to view and modify them when needed.