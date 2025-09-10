# SafeTensors Mac Testing Instructions

## 🍎 **MAC TESTING REQUIRED - CRITICAL FOR v1.2.0 RELEASE**

**Branch**: `safetensors-testing`  
**Priority**: HIGH - This is the final validation before SafeTensors v1.2.0 release

---

## 📋 **Quick Test Checklist**

Run these commands on Mac and report results:

```bash
# 1. Pull the test branch
git checkout safetensors-testing
git pull origin safetensors-testing

# 2. Build with SafeTensors support
cargo build --all-features

# 3. Create test model
cargo run --bin create_test_safetensors

# 4. Test basic functionality
cargo run --bin shimmy -- discover
cargo run --bin shimmy -- probe model
cargo run --bin shimmy -- generate model --prompt "Hello Mac" --max-tokens 20

# 5. Create realistic test model
cargo run --bin create_realistic_safetensors

# 6. Test memory handling
cargo run --bin test_real_safetensors

# 7. Test all SafeTensors tests
cargo test safetensors --all-features -- --nocapture
```

---

## 🧪 **Detailed Testing Protocol**

### **Phase 1: Basic Compilation & Setup**
```bash
cd ~/repos/shimmy  # or wherever you cloned it
git fetch origin
git checkout safetensors-testing
git pull origin safetensors-testing

# Verify Rust version (should be 1.70+)
rustc --version
cargo --version

# Clean build
cargo clean
cargo build --all-features
```

**Expected**: Should compile without errors on Mac

### **Phase 2: SafeTensors Engine Testing**
```bash
# Create minimal test model
cargo run --bin create_test_safetensors

# This should create: test-safetensors-model/
# With files: model.safetensors, config.json, tokenizer.json

# Test discovery
cargo run --bin shimmy -- discover
```

**Expected**: Should find the SafeTensors model in the list

### **Phase 3: Model Loading & Probing**
```bash
# Test model loading
cargo run --bin shimmy -- probe model

# Test generation
cargo run --bin shimmy -- generate model --prompt "Hello from Mac" --max-tokens 20
```

**Expected**: 
- Probe: "ok: loaded model"
- Generate: Should return demo response about SafeTensors loading

### **Phase 4: Realistic Model Testing**
```bash
# Create realistic 90MB model
cargo run --bin create_realistic_safetensors

# Test with larger model
cargo run --bin shimmy -- discover
# Should find the new realistic model

# Test memory handling
cargo run --bin test_real_safetensors
```

**Expected**: No crashes, reasonable performance

### **Phase 5: Comprehensive Test Suite**
```bash
# Run all SafeTensors tests
cargo test safetensors --all-features -- --nocapture

# Run integration tests
cargo test safetensors_integration --all-features -- --nocapture
```

**Expected**: All tests should pass

### **Phase 6: Server Mode Testing**
```bash
# Start server in background
cargo run --bin shimmy -- serve &
SERVER_PID=$!

# Give it time to start
sleep 3

# Test health endpoint
curl -s http://127.0.0.1:11435/health

# Stop server
kill $SERVER_PID
```

**Expected**: Server starts and responds to health check

### **Phase 7: Performance & Memory Testing**
```bash
# Check memory usage during large model loading
# (This will create progressively larger test models)
cargo run --bin test_real_safetensors
```

**Expected**: Should handle up to 100MB models without issues

---

## 🐛 **What to Report Back**

### **If Everything Works:**
```
✅ Mac Testing Results - ALL PASS
- Compilation: SUCCESS  
- Model Discovery: SUCCESS (X models found)
- Model Loading: SUCCESS (probe command works)
- Model Generation: SUCCESS (generates response)
- Test Suite: SUCCESS (X/X tests pass)
- Server Mode: SUCCESS (starts and responds)
- Memory Handling: SUCCESS (handles large models)

Platform: macOS X.X.X (Intel/ARM64)
Rust: 1.XX.X
```

### **If Something Breaks:**
```
❌ Mac Testing Results - ISSUES FOUND

Failed Command: [exact command that failed]

Error Output:
[paste complete error message]

Platform Details:
- macOS: X.X.X
- Architecture: Intel/ARM64  
- Rust: 1.XX.X
- Memory: XXG

Steps to Reproduce:
1. [exact steps]
```

---

## 🔍 **Common Mac-Specific Issues to Watch For**

1. **Permission Issues**: macOS security might block file access
2. **Memory Pressure**: macOS might be more aggressive about memory management
3. **Path Issues**: Different filesystem structure than Windows
4. **ARM64 vs Intel**: Different behavior on Apple Silicon vs Intel Macs
5. **Linking Issues**: Different system libraries on Mac

---

## 📊 **Performance Expectations**

Based on Windows testing:
- **Compilation**: ~30 seconds
- **Small model loading**: <1 second  
- **Large model (100MB) loading**: 2-3 seconds
- **Test suite**: <10 seconds to run
- **Memory usage**: Reasonable for model size

---

## 🚨 **Critical Success Criteria**

For v1.2.0 release, Mac must pass:
1. ✅ Compiles without errors
2. ✅ Discovers SafeTensors models
3. ✅ Loads models without hanging/crashing  
4. ✅ Generates responses (even if demo text)
5. ✅ Handles realistic model sizes (90MB+)
6. ✅ Test suite passes
7. ✅ Server mode works

**If any of these fail, we need to fix before release.**

---

## 🔧 **Debug Commands (If Issues Occur)**

```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin shimmy -- probe model

# Check file permissions
ls -la test-safetensors-model/

# Memory usage monitoring
top -pid $(pgrep shimmy)

# Detailed error tracing
RUST_BACKTRACE=full cargo run --bin shimmy -- generate model --prompt "test"
```

---

**Bottom Line**: Run the quick checklist, and if it all works, we're ready to announce SafeTensors support! If anything breaks, paste the exact error and I'll fix it.