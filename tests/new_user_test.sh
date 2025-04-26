#!/bin/bash

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
echo -e "${BLUE}Cleaning up existing configuration...${NC}"

if [ -f "$USER_CONFIG" ]; then
    echo "Backing up $USER_CONFIG to ${USER_CONFIG}.bak"
    cp "$USER_CONFIG" "${USER_CONFIG}.bak"
    rm "$USER_CONFIG"
    echo -e "${GREEN}✓ Removed existing user config${NC}"
else
    echo "No existing user config found"
fi

if [ -f "$MOD_REGISTRY" ]; then
    echo "Backing up $MOD_REGISTRY to ${MOD_REGISTRY}.bak"
    cp "$MOD_REGISTRY" "${MOD_REGISTRY}.bak"
    rm "$MOD_REGISTRY"
    echo -e "${GREEN}✓ Removed existing mod registry${NC}"
else
    echo "No existing mod registry found"
fi

if [ -f "$SKIN_REGISTRY" ]; then
    echo "Backing up $SKIN_REGISTRY to ${SKIN_REGISTRY}.bak"
    cp "$SKIN_REGISTRY" "${SKIN_REGISTRY}.bak"
    rm "$SKIN_REGISTRY"
    echo -e "${GREEN}✓ Removed existing skin registry${NC}"
else
    echo "No existing skin registry found"
fi

# Step 2: Launch application
echo -e "${BLUE}Launching FOSSModManager...${NC}"
echo "IMPORTANT: Complete the following steps manually:"
echo "1. The setup overlay should appear"
echo "2. Click the setup button and select your game executable"
echo "3. Verify that the game is detected correctly"
echo "4. After setup completes, close the application"
echo 
echo -e "${BLUE}Press Enter to continue...${NC}"
read

# Launch the application (this is a blocking call)
fossmodmanager

# Step 3: Verify configuration files
echo -e "${BLUE}Verifying configuration files...${NC}"

if [ -f "$USER_CONFIG" ]; then
    echo -e "${GREEN}✓ userconfig.json was created${NC}"
    echo "Content preview:"
    head -n 10 "$USER_CONFIG"
else
    echo -e "${RED}✗ userconfig.json was not created${NC}"
fi

if [ -f "$MOD_REGISTRY" ]; then
    echo -e "${GREEN}✓ mod_registry.json was created${NC}"
    echo "Content preview:"
    head -n 10 "$MOD_REGISTRY"
else
    echo -e "${RED}✗ mod_registry.json was not created${NC}"
fi

if [ -f "$SKIN_REGISTRY" ]; then
    echo -e "${GREEN}✓ skin_registry.json was created${NC}"
    echo "Content preview:"
    head -n 10 "$SKIN_REGISTRY"
else
    echo -e "${RED}? skin_registry.json may be created later${NC}"
fi

# Summary
echo
echo -e "${BLUE}=== Test Summary ===${NC}"
echo "The test has completed. Please verify that:"
echo "1. The setup overlay was displayed correctly"
echo "2. You were able to select your game executable"
echo "3. The configuration files were created as shown above"
echo
echo "If all of the above are true, the test was successful!"

# Restore backups if needed
echo -e "${BLUE}Would you like to restore your original configuration files? (y/N)${NC}"
read restore

if [[ "$restore" == "y" || "$restore" == "Y" ]]; then
    if [ -f "${USER_CONFIG}.bak" ]; then
        mv "${USER_CONFIG}.bak" "$USER_CONFIG"
        echo -e "${GREEN}✓ Restored userconfig.json${NC}"
    fi
    
    if [ -f "${MOD_REGISTRY}.bak" ]; then
        mv "${MOD_REGISTRY}.bak" "$MOD_REGISTRY"
        echo -e "${GREEN}✓ Restored mod_registry.json${NC}"
    fi
    
    if [ -f "${SKIN_REGISTRY}.bak" ]; then
        mv "${SKIN_REGISTRY}.bak" "$SKIN_REGISTRY"
        echo -e "${GREEN}✓ Restored skin_registry.json${NC}"
    fi
    
    echo -e "${GREEN}Original configuration restored${NC}"
else
    echo "Keeping new configuration files" 