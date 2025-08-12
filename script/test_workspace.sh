#!/bin/bash

# Test script to verify workspace cargo test functionality
# This script tests different feature combinations to isolate issues

set -e

echo "=== Testing CGTools Workspace Cargo Commands ==="

cd /home/user1/pro/lib/cgtools

echo "1. Testing workspace check..."
if cargo check --workspace --quiet; then
    echo "✅ Workspace check passed"
else
    echo "❌ Workspace check failed"
    exit 1
fi

echo "2. Testing individual problematic crates..."

# Test embroidery_tools without random features
echo "Testing embroidery_tools..."
if cargo check -p embroidery_tools --no-default-features --features enabled; then
    echo "✅ embroidery_tools check passed"
else
    echo "❌ embroidery_tools check failed"
fi

# Test vectorizer without CLI features  
echo "Testing vectorizer..."
if cargo check -p vectorizer --no-default-features --features enabled; then
    echo "✅ vectorizer check passed"
else
    echo "❌ vectorizer check failed"
fi

# Test workspace with basic features only
echo "3. Testing workspace with minimal features..."
if cargo check --workspace --no-default-features; then
    echo "✅ Workspace minimal check passed"
else
    echo "❌ Workspace minimal check failed"
fi

echo "4. Testing workspace tests..."
if cargo test --workspace --quiet --no-default-features; then
    echo "✅ Workspace tests passed"
else
    echo "❌ Workspace tests failed"
fi

echo "=== All tests completed ==="