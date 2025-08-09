#!/bin/bash

# Quick release script for tiles_tools patch versions
# Usage: ./quick_release.sh
# Automatically bumps patch version and releases

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Get current version and calculate next patch version
CURRENT_VERSION=$(grep '^version' Cargo.toml | head -n1 | sed 's/version = "\(.*\)"/\1/')
MAJOR=$(echo $CURRENT_VERSION | cut -d. -f1)
MINOR=$(echo $CURRENT_VERSION | cut -d. -f2)  
PATCH=$(echo $CURRENT_VERSION | cut -d. -f3)
NEW_PATCH=$((PATCH + 1))
NEW_VERSION="$MAJOR.$MINOR.$NEW_PATCH"

log_info "Current version: $CURRENT_VERSION"
log_info "New version: $NEW_VERSION"

echo
read -p "Proceed with quick release? [y/N]: " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    "$(dirname "$0")/release.sh" "$NEW_VERSION"
else
    log_info "Release cancelled"
fi