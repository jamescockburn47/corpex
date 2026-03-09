#!/bin/bash
# Corpex Smoke Tests
# Verifies the native build compiles and basic functionality works.
# Run: bash tests/smoke_test.sh

set -e

# Ensure cargo is on PATH (Windows / CI)
export PATH="$HOME/.cargo/bin:$PATH"

PASS=0
FAIL=0

pass() { echo "  ✓ $1"; PASS=$((PASS + 1)); }
fail() { echo "  ✗ $1"; FAIL=$((FAIL + 1)); }

echo "=== Corpex Smoke Tests ==="
echo ""

# ── 1. Cargo check (type-checks without full build) ─────────────────
echo "1. Cargo check..."
if cargo check 2>&1 | grep -q "Finished"; then
  pass "cargo check succeeds"
else
  fail "cargo check failed"
fi

# ── 2. Cargo build release ───────────────────────────────────────────
echo "2. Cargo build --release..."
if cargo build --release 2>&1 | grep -q "Finished"; then
  pass "release build succeeds"
else
  fail "release build failed"
fi

# ── 3. Binary exists and is executable ───────────────────────────────
echo "3. Binary check..."
BINARY="target/release/corpex"
if [ -f "$BINARY" ] || [ -f "$BINARY.exe" ]; then
  pass "binary exists"
else
  fail "binary not found at $BINARY"
fi

# ── 4. No WASM artifacts (ensure clean native-only) ──────────────────
echo "4. No WASM contamination..."
if [ -f "src/platform.rs" ]; then
  fail "src/platform.rs exists (WASM artifact)"
elif [ -f "src/lib.rs" ]; then
  fail "src/lib.rs exists (WASM artifact)"
elif [ -f "Trunk.toml" ]; then
  fail "Trunk.toml exists (WASM artifact)"
elif grep -q 'wasm-bindgen' Cargo.toml 2>/dev/null; then
  fail "Cargo.toml contains wasm-bindgen dependency"
elif grep -q 'crate-type.*cdylib' Cargo.toml 2>/dev/null; then
  fail "Cargo.toml has cdylib crate type (WASM artifact)"
else
  pass "no WASM artifacts found"
fi

# ── 5. Key source files exist ────────────────────────────────────────
echo "5. Source file integrity..."
MISSING=0
for f in src/main.rs src/app.rs src/config.rs src/cache.rs src/ch_api/client.rs \
         src/extraction/pdf.rs src/export/mod.rs src/ui/analysis_panel.rs; do
  if [ ! -f "$f" ]; then
    fail "missing: $f"
    MISSING=$((MISSING + 1))
  fi
done
if [ $MISSING -eq 0 ]; then
  pass "all key source files present"
fi

# ── 6. Cargo.toml has expected native dependencies ───────────────────
echo "6. Native dependencies..."
DEPS_OK=true
for dep in tokio crossbeam-channel pdf-extract dotenvy reqwest; do
  if ! grep -q "$dep" Cargo.toml; then
    fail "missing dependency: $dep"
    DEPS_OK=false
  fi
done
if $DEPS_OK; then
  pass "all native dependencies present"
fi

# ── 7. Dockerfile and demo.json exist ────────────────────────────────
echo "7. Demo files..."
if [ -f "Dockerfile" ] && [ -f "demo.json" ] && [ -f "docker-entrypoint.sh" ]; then
  pass "Dockerfile, demo.json, docker-entrypoint.sh present"
else
  fail "missing demo deployment files"
fi

# ── Summary ──────────────────────────────────────────────────────────
echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
if [ $FAIL -gt 0 ]; then
  exit 1
fi
