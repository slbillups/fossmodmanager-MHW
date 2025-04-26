#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# Test script to simulate a new user experience with FOSSModManager
# This script will:
# 1. Delete existing configuration files
# 2. Launch the app and perform setup
# 3. Verify that configuration files are created

# Text colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== FOSSModManager New User Test ===${NC}"

# Define configuration paths (based on paths in the codebase)
# These will vary by OS - this script is for Linux

# Get the XDG config and data dirs, or use defaults
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/fossmodmanager"
DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/fossmodmanager"

USER_CONFIG="$CONFIG_DIR/userconfig.json"
MOD_REGISTRY="$CONFIG_DIR/mod_registry.json"
SKIN_REGISTRY="$CONFIG_DIR/skin_registry.json"

# Check if app is installed
if ! command -v fossmodmanager &> /dev/null; then
    echo -e "${RED}FOSSModManager is not installed or not in your PATH${NC}"
    echo "Make sure to build the app with 'cargo tauri build' first"
    exit 1
fi

# Step 1: Clean up existing configuration
echo -e "${BLUE}--- Cleaning up existing configuration ---${NC}"

backup_and_remove() {
    local file_path="$1"
    local file_name=$(basename "$file_path")
    if [ -f "$file_path" ]; then
        echo "Backing up $file_name to ${file_path}.bak..."
        cp "$file_path" "${file_path}.bak" || { echo -e "${RED}✗ Failed to backup $file_name${NC}"; exit 1; }
        rm "$file_path" || { echo -e "${RED}✗ Failed to remove $file_name${NC}"; exit 1; }
        echo -e "${GREEN}✓ Removed existing $file_name${NC}"
    else
        echo "No existing $file_name found."
    fi
}

backup_and_remove "$USER_CONFIG"
backup_and_remove "$MOD_REGISTRY"
backup_and_remove "$SKIN_REGISTRY"

# Step 2: Launch application
echo -e "\n${BLUE}--- Launching FOSSModManager ---${NC}"
echo "IMPORTANT: Complete the following steps manually:"
echo "1. The setup overlay should appear"
echo "2. Click the setup button and select your game executable"
echo "3. Verify that the game is detected correctly"
echo "4. After setup completes, close the application"
echo 
echo -e "${BLUE}Press Enter to continue...${NC}"
read

# Launch the application (this is a blocking call)
# Ensure the application is built and in PATH
fossmodmanager || { echo -e "${RED}✗ Failed to launch FOSSModManager. Is it in PATH and built correctly?${NC}"; exit 1; }

# Step 3: Verify configuration files
echo -e "\n${BLUE}--- Verifying configuration files ---${NC}"

TEST_FAILED=0

verify_file() {
    local file_path="$1"
    local file_name=$(basename "$file_path")
    local optional="$2" # Optional second argument

    if [ -f "$file_path" ]; then
        echo -e "${GREEN}✓ $file_name was created${NC}"
        echo "    Preview:"
        head -n 5 "$file_path" | sed 's/^/    /' # Indent preview
    elif [ "$optional" == "optional" ]; then
         echo -e "${BLUE}? $file_name is optional and was not created (this might be OK).${NC}"
    else
        echo -e "${RED}✗ $file_name was NOT created${NC}"
        TEST_FAILED=1
    fi
}

verify_file "$USER_CONFIG"
verify_file "$MOD_REGISTRY"
verify_file "$SKIN_REGISTRY" "optional" # Mark skin registry as potentially optional for initial setup

# Summary
echo
echo -e "${BLUE}--- Test Summary ---${NC}"
echo "The test has completed. Please verify that:"
echo "1. The setup overlay was displayed correctly"
echo "2. You were able to select your game executable"
echo "3. The configuration files were created as shown above"
echo

if [ $TEST_FAILED -eq 1 ]; then
    echo -e "${RED}=== TEST FAILED: One or more required configuration files were not created ===${NC}"
else
    echo -e "${GREEN}=== TEST POTENTIALLY SUCCESSFUL (File Check) ===${NC}"
    echo "Please confirm the manual setup steps were also successful."
fi

# Restore backups if needed
echo -e "\n${BLUE}--- Restore Original Configuration ---${NC}"
echo -e "${BLUE}Would you like to restore your original configuration files? (y/N)${NC}"
read restore

if [[ "$restore" == "y" || "$restore" == "Y" ]]; then
    if [ -f "${USER_CONFIG}.bak" ]; then
        mv "${USER_CONFIG}.bak" "$USER_CONFIG" || echo -e "${RED}Warning: Failed to restore ${USER_CONFIG}.bak${NC}"
        echo -e "${GREEN}✓ Restored userconfig.json${NC}"
    fi
    
    if [ -f "${MOD_REGISTRY}.bak" ]; then
        mv "${MOD_REGISTRY}.bak" "$MOD_REGISTRY" || echo -e "${RED}Warning: Failed to restore ${MOD_REGISTRY}.bak${NC}"
        echo -e "${GREEN}✓ Restored mod_registry.json${NC}"
    fi
    
    if [ -f "${SKIN_REGISTRY}.bak" ]; then
        mv "${SKIN_REGISTRY}.bak" "$SKIN_REGISTRY" || echo -e "${RED}Warning: Failed to restore ${SKIN_REGISTRY}.bak${NC}"
        echo -e "${GREEN}✓ Restored skin_registry.json${NC}"
    fi
    
    echo -e "${GREEN}Original configuration restored.${NC}"
else
    echo "Keeping new configuration files."
fi

# Exit with status code
exit $TEST_FAILED 