#!/bin/bash

# Pre-release validation script for tiles_tools
# Usage: ./pre_release.sh

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m' 
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

ERRORS=0

check_step() {
    local step_name="$1"
    local command="$2"
    
    log_info "Checking: $step_name"
    
    if eval "$command" >/dev/null 2>&1; then
        log_success "$step_name ✓"
    else
        log_error "$step_name ✗"
        ERRORS=$((ERRORS + 1))
    fi
}

echo "🚀 Pre-release validation for tiles_tools"
echo "========================================"

# Check 1: Git status
log_info "Checking git status..."
if git diff-index --quiet HEAD --; then
    log_success "Working directory clean ✓"
else
    log_warning "Working directory has uncommitted changes ⚠"
fi

# Check 2: Version format
VERSION=$(grep '^version' Cargo.toml | head -n1 | sed 's/version = "\(.*\)"/\1/')
if [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    log_success "Version format valid: $VERSION ✓"
else
    log_error "Invalid version format: $VERSION ✗"
    ERRORS=$((ERRORS + 1))
fi

# Check 3: Required files exist
log_info "Checking required files..."
for file in "readme.md" "changelog.md" "license" "Cargo.toml"; do
    if [[ -f "$file" ]]; then
        log_success "$file exists ✓"
    else
        log_error "$file missing ✗"
        ERRORS=$((ERRORS + 1))
    fi
done

# Check 4: Tests
check_step "Tests" "cargo test --all-features"

# Check 5: Documentation
check_step "Documentation build" "cargo doc --all-features --no-deps"

# Check 6: Package check
check_step "Package validation" "cargo package --allow-dirty"

# Check 7: Publish dry-run
check_step "Publish dry-run" "cargo publish --dry-run --allow-dirty"

# Check 8: File naming (lowercase)
log_info "Checking file naming conventions..."
UPPERCASE_FILES=$(find . -name "*.rs" -o -name "*.md" -o -name "*.toml" | grep -E '[A-Z]' | grep -v './target/' | grep -v './Cargo.toml' || true)
if [[ -z "$UPPERCASE_FILES" ]]; then
    log_success "All files use lowercase naming ✓"
else
    log_warning "Files with uppercase found:"
    echo "$UPPERCASE_FILES"
fi

echo
echo "========================================"
if [[ $ERRORS -eq 0 ]]; then
    log_success "🎉 All checks passed! Ready for release."
    echo
    log_info "Next steps:"
    echo "  • Run script/release.sh [version] for full release"  
    echo "  • Run script/quick_release.sh for patch version bump"
    echo "  • Run script/release.sh --dry-run to preview changes"
else
    log_error "❌ $ERRORS errors found. Please fix before releasing."
    exit 1
fi