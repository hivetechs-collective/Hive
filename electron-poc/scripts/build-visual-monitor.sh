#!/bin/bash

# Build Visual Monitor - Shows real-time phase progress
echo "🏗️  Production DMG Build Monitor"
echo "================================"
echo ""
echo "Monitoring build progress..."
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
RESET='\033[0m'
BOLD='\033[1m'

# Monitor the build log for phase progress
tail -f "$1" 2>/dev/null | while IFS= read -r line; do
    # Check for phase markers
    if [[ "$line" == *"PHASE"*"/"* ]]; then
        echo -e "${BLUE}${BOLD}$line${RESET}"
    elif [[ "$line" == *"✓ Success"* ]]; then
        echo -e "${GREEN}$line${RESET}"
    elif [[ "$line" == *"✗"* ]] || [[ "$line" == *"ERROR"* ]]; then
        echo -e "${RED}$line${RESET}"
    elif [[ "$line" == *"WARNING"* ]] || [[ "$line" == *"➤"* ]]; then
        echo -e "${YELLOW}$line${RESET}"
    elif [[ "$line" == *"BUILD SUCCESSFUL"* ]]; then
        echo -e "${GREEN}${BOLD}$line${RESET}"
    elif [[ "$line" == *"Description:"* ]]; then
        echo -e "${CYAN}$line${RESET}"
    else
        echo "$line"
    fi
done