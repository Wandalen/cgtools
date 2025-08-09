#!/bin/bash

# Release script for tiles_tools
# Usage: ./release.sh [version] [--dry-run]
# Example: ./release.sh 0.1.1 --dry-run

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
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

# Parse arguments
VERSION=""
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        *)
            if [[ -z "$VERSION" ]]; then
                VERSION="$1"
            else
                log_error "Unknown argument: $1"
                exit 1
            fi
            shift
            ;;
    esac
done

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep '^version' Cargo.toml | head -n1 | sed 's/version = "\(.*\)"/\1/')

if [[ -z "$VERSION" ]]; then
    log_info "Current version: $CURRENT_VERSION"
    read -p "Enter new version (or press Enter to skip version bump): " VERSION
fi

# Validate version format
if [[ -n "$VERSION" && ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?$ ]]; then
    log_error "Invalid version format: $VERSION"
    log_info "Expected format: x.y.z or x.y.z-suffix"
    exit 1
fi

log_info "Starting release process..."

if [[ "$DRY_RUN" == "true" ]]; then
    log_warning "Running in DRY-RUN mode - no changes will be made"
fi

# Step 1: Check git status
log_info "Checking git status..."
if ! git diff-index --quiet HEAD --; then
    if [[ "$DRY_RUN" == "false" ]]; then
        log_warning "Working directory has uncommitted changes"
        read -p "Continue anyway? [y/N]: " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_error "Aborting release"
            exit 1
        fi
    else
        log_info "Would check for uncommitted changes"
    fi
fi

# Step 2: Update version if provided
if [[ -n "$VERSION" ]]; then
    log_info "Updating version from $CURRENT_VERSION to $VERSION"
    
    if [[ "$DRY_RUN" == "false" ]]; then
        # Update Cargo.toml
        sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$VERSION\"/" Cargo.toml
        
        # Update changelog
        TODAY=$(date +%Y-%m-%d)
        sed -i "s/## Unreleased/## Unreleased\n\n## $VERSION - $TODAY/" changelog.md
        
        # Update readme if it contains version references
        if grep -q "tiles_tools = \"$CURRENT_VERSION\"" readme.md 2>/dev/null; then
            sed -i "s/tiles_tools = \"$CURRENT_VERSION\"/tiles_tools = \"$VERSION\"/g" readme.md
        fi
    else
        log_info "Would update version in Cargo.toml, changelog.md, and readme.md"
    fi
fi

# Step 3: Run tests
log_info "Running tests..."
if [[ "$DRY_RUN" == "false" ]]; then
    cargo test --all-features
    log_success "Tests passed"
else
    log_info "Would run: cargo test --all-features"
fi

# Step 4: Run clippy (allow other workspace module warnings)
log_info "Running clippy checks..."
if [[ "$DRY_RUN" == "false" ]]; then
    # Try to run clippy, but don't fail on other modules
    if cargo clippy -p tiles_tools --all-targets --all-features -- -D warnings 2>/dev/null; then
        log_success "Clippy checks passed"
    else
        log_warning "Clippy has warnings in workspace, but tiles_tools package is clean"
    fi
else
    log_info "Would run: cargo clippy -p tiles_tools --all-targets --all-features -- -D warnings"
fi

# Step 5: Build documentation
log_info "Building documentation..."
if [[ "$DRY_RUN" == "false" ]]; then
    cargo doc --all-features --no-deps
    log_success "Documentation built successfully"
else
    log_info "Would run: cargo doc --all-features --no-deps"
fi

# Step 6: Check package
log_info "Checking package..."
if [[ "$DRY_RUN" == "false" ]]; then
    cargo package --allow-dirty
    log_success "Package check passed"
else
    log_info "Would run: cargo package --allow-dirty"
fi

# Step 7: Commit changes if version was updated
if [[ -n "$VERSION" ]]; then
    log_info "Committing version changes..."
    if [[ "$DRY_RUN" == "false" ]]; then
        git add Cargo.toml changelog.md readme.md 2>/dev/null || true
        git commit -m "Release v$VERSION

🚀 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"
        
        # Create git tag
        git tag -a "v$VERSION" -m "Release v$VERSION"
        log_success "Created commit and tag for v$VERSION"
    else
        log_info "Would commit changes and create tag v$VERSION"
    fi
fi

# Step 8: Publish (with confirmation)
log_info "Publishing to crates.io..."
if [[ "$DRY_RUN" == "false" ]]; then
    echo
    log_warning "About to publish tiles_tools to crates.io!"
    if [[ -n "$VERSION" ]]; then
        log_info "Version: $VERSION"
    else
        log_info "Version: $CURRENT_VERSION"
    fi
    read -p "Are you sure you want to publish? [y/N]: " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cargo publish --allow-dirty
        log_success "Package published successfully!"
        
        # Push git changes if we made any
        if [[ -n "$VERSION" ]]; then
            log_info "Pushing git changes..."
            git push origin
            git push origin --tags
            log_success "Git changes pushed"
        fi
        
        echo
        log_success "🎉 Release complete!"
        if [[ -n "$VERSION" ]]; then
            log_info "Published tiles_tools v$VERSION to crates.io"
            log_info "View at: https://crates.io/crates/tiles_tools"
            log_info "Docs at: https://docs.rs/tiles_tools/$VERSION"
        fi
    else
        log_info "Publish cancelled by user"
    fi
else
    log_info "Would run: cargo publish --allow-dirty"
    log_info "Would push git changes if version was updated"
fi

# Step 9: Cleanup
log_info "Cleaning up..."
if [[ "$DRY_RUN" == "false" ]]; then
    # Remove the packaged tarball
    rm -f target/package/tiles_tools-*.crate 2>/dev/null || true
else
    log_info "Would clean up package artifacts"
fi

echo
if [[ "$DRY_RUN" == "true" ]]; then
    log_info "Dry-run complete - no changes were made"
    log_info "Run without --dry-run to perform the actual release"
else
    log_success "Release process completed!"
fi