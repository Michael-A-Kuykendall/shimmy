# Schema Generation Pipeline Audit Report
**Date:** November 18, 2025
**Issue:** Generated theme fails discovery with "health_check field missing"

---

## 🔴 ROOT CAUSE: Schema/Reality Mismatch

The contract schema in `console/src/contract/mod.rs` does NOT match the actual HTTP responses from `src/discovery/http.rs`.

### Discovery Response Comparison

| Field | Schema (`contract/mod.rs`) | Reality (`discovery/http.rs`) | Match |
|-------|----------------------------|-------------------------------|-------|
| `discovery_port` | ❌ Missing | ✅ `pub discovery_port: u16` | ❌ FAIL |
| `backends` | ❌ Wrong type | ✅ `pub backends: Vec<BackendInfo>` | ❌ FAIL |
| `last_updated` | ❌ Missing | ✅ `pub last_updated: String` | ❌ FAIL |
| `epoch` | ❌ Missing | ✅ `pub epoch: u64` | ❌ FAIL |
| `health_check` | ✅ Defined | ❌ Does NOT exist | ❌ FAIL |
| `websocket_port` | ✅ Defined | ❌ Does NOT exist | ❌ FAIL |

### BackendInfo Comparison

**Schema Definition** (`console/src/contract/mod.rs` line 518):
```rust
pub struct BackendInfo {
    pub backend_type: String,
    pub version: String,
    pub api_versions: Vec<String>,
    pub capabilities: BackendCapabilities,  // Complex nested struct
    pub hardware: HardwareInfo,
    pub validation: ValidationInfo,
    pub protocols: ProtocolSupport,
}
```

**Actual Runtime** (`src/discovery/http.rs` line 57):
```rust
pub struct BackendInfo {
    pub id: String,
    pub url: String,
    pub port: u16,
    pub models: Vec<ModelMetadata>,
    pub capabilities: Vec<String>,  // Simple string array!
    pub health: Health,
    pub started_at: String,
    pub pid: u32,
}
```

**Match:** 0% - Completely different structures!

---

## 📊 Full Pipeline Audit

### Layer 1: Rust Backend Implementation ✅ WORKING

| Component | File | Status | Notes |
|-----------|------|--------|-------|
| Discovery HTTP Server | `src/discovery/http.rs` | ✅ PASS | Serving on port 11430 |
| Discovery Response Struct | `src/discovery/http.rs:67` | ✅ PASS | `DiscoverySnapshot` correctly defined |
| Backend Registration | `src/discovery/` | ✅ PASS | Backends registering correctly |
| WebSocket Server | `console/src/websocket/mod.rs` | ✅ PASS | Listening on port 49428 |
| Models Loaded | Runtime | ✅ PASS | 9 models available |
| Discovery Endpoint | `GET /api/discovery` | ✅ PASS | Responding with JSON |
| Schema Endpoint | `GET /__shimmy__/schema` | ✅ PASS | Returning contract JSON |

**Verdict:** Backend runtime is 100% functional.

---

### Layer 2: Contract Schema Definition ❌ BROKEN

| Component | File | Status | Issue |
|-----------|------|--------|-------|
| Contract Module | `console/src/contract/mod.rs` | ❌ FAIL | Defines WRONG structs |
| `DiscoveryResponse` | `contract/mod.rs:500` | ❌ FAIL | Missing `discovery_port`, `backends`, `epoch` |
| `BackendInfo` | `contract/mod.rs:518` | ❌ FAIL | Completely wrong structure |
| Schema Generation | `mod.rs` | ❌ FAIL | Generates schema from WRONG types |
| `/__shimmy__/schema` endpoint | `src/server.rs` | ⚠️ WARNING | Serves schema from wrong source |

**Problem:** The contract module defines idealized/theoretical types that don't match the actual runtime implementation.

**Root Cause:** Two separate teams/implementations:
1. `src/discovery/` - Actual working implementation
2. `console/src/contract/` - Theoretical schema (never tested against reality)

---

### Layer 3: Schema Export Pipeline ⚠️ PARTIALLY WORKING

| Component | File | Status | Issue |
|-----------|------|--------|-------|
| Export Script | `REMOVED` | ⚠️ REMOVED | The legacy `orchestrator schema export (`/__shimmy__/schema` endpoint)` has been removed from the primary repository. Use the orchestrator HTTP endpoint `/__shimmy__/schema` or `shimmy dev --verify` to produce the canonical frontend contract JSON |
| Schema File | frontend contract JSON | ❌ FAIL | Contains wrong types |
| Schema Endpoint Fetch | `/__shimmy__/schema` | ⚠️ WARNING | Ensure the running orchestrator is serving the canonical frontend schema at this endpoint; use `curl` or an HTTP client to fetch the contract |
| Type Generation Script | `scripts/generate-types.js` | ✅ PASS | Works correctly (but on wrong input) |
| TypeScript Types | `theme-types/shimmy-contract.d.ts` | ❌ FAIL | Generated from wrong schema |

**Verdict:** Pipeline works but garbage-in → garbage-out.

---

### Layer 4: Theme Generator ⚠️ PARTIALLY WORKING

| Component | File | Status | Issue |
|-----------|------|--------|-------|
| Generator CLI | `theme-generator/generate.js` | ✅ PASS | CLI works correctly |
| Template System | `theme-generator/templates/` | ✅ PASS | EJS templating functional |
| React-Vite Template | `templates/react-vite/` | ✅ PASS | Template structure correct |
| Schema Consumption | `generate.js` | ❌ FAIL | Uses wrong schema as input |
| Hook Generation | `useDiscovery.ts` | ❌ FAIL | Generated with wrong types |
| Type Validation | `useDiscovery.ts:17` | ❌ FAIL | Expects `health_check` field |

**Verdict:** Generator works correctly but produces broken code due to bad schema.

---

### Layer 5: Generated Theme Runtime ❌ BROKEN

| Component | File | Status | Issue |
|-----------|------|--------|-------|
| Theme Served | `localhost:8080` | ✅ PASS | Vite serving HTML correctly |
| Discovery Hook | `useDiscovery.ts` | ❌ FAIL | Validation fails on `health_check` |
| WebSocket Hook | `useWebSocket.ts` | ⚠️ UNKNOWN | Not tested yet (blocked by discovery) |
| Model Chooser | `ModelChooser.tsx` | ⚠️ UNKNOWN | Not tested yet |
| Chat Component | `Chat.tsx` | ⚠️ UNKNOWN | Not tested yet |

**Actual Error in Browser Console:**
```
Missing required fields: health_check
```

**Expected Response:**
```typescript
{
  health_check: boolean,
  websocket_port?: number
}
```

**Actual Response:**
```json
{
  "discovery_port": 11430,
  "last_updated": "2025-11-18T19:09:21.815948+00:00",
  "epoch": 1,
  "backends": [{
    "id": "shimmy-49428",
    "port": 49428,
    ...
  }]
}
```

---

## 🔧 Required Fixes

### Fix #1: Align Contract Schema with Reality (HIGH PRIORITY)

**File:** `console/src/contract/mod.rs`

**Action:** Replace theoretical types with actual runtime types from `src/discovery/http.rs`

**Changes Needed:**
1. Import `DiscoverySnapshot` from `src/discovery/http.rs`
2. Import `BackendInfo` from `src/discovery/http.rs`
3. Remove duplicate/wrong definitions in `contract/mod.rs`
4. Add `#[derive(JsonSchema)]` to the real types in `discovery/http.rs`
5. Update schema generation to use real types

**Affected Lines:**
- `console/src/contract/mod.rs:500-550` - Wrong `DiscoveryResponse` definition
- `console/src/contract/mod.rs:518-565` - Wrong `BackendInfo` definition

---

### Fix #2: Regenerate Schema from Corrected Types

**File:** frontend contract JSON

**Action:** Export fresh schema after Fix #1

**Command:**
```bash
curl http://localhost:11435/__shimmy__/schema -o <local-contract-file>.json
```

**Validation:**
- Check `discovery` section matches `DiscoverySnapshot`
- Verify `backends` array structure matches runtime
- Confirm `discovery_port`, `epoch`, `last_updated` present

---

### Fix #3: Regenerate TypeScript Types

**File:** `theme-types/shimmy-contract.d.ts`

**Action:** Generate types from corrected schema

**Command:**
```bash
node scripts/generate-types.js
```

**Validation:**
- `DiscoverySnapshot` interface should have `discovery_port`
- `BackendInfo` interface should have `id`, `url`, `port`, `models`
- No `health_check` or `websocket_port` in discovery response

---

### Fix #4: Regenerate Default Theme

**File:** `theme-generator/themes/shimmy-default/`

**Action:** Delete and regenerate from corrected schema

**Commands:**
```bash
rm -rf theme-generator/themes/shimmy-default
node theme-generator/generate.js new shimmy-default --template react-vite
```

**Validation:**
- `useDiscovery.ts` should validate correct fields
- Discovery should extract port from `backends[0].port`
- No references to `health_check` or `websocket_port`

---

### Fix #5: Update Theme Package Documentation

**File:** `theme-validator/THEME_PACKAGE.md`

**Action:** Update discovery protocol section with correct response structure

**Changes:**
```markdown
**Response Schema:**
```typescript
interface DiscoveryResponse {
  discovery_port: number;
  last_updated: string;
  epoch: number;
  backends: BackendInfo[];
}

interface BackendInfo {
  id: string;
  url: string;
  port: number;  // WebSocket port here!
  models: ModelMetadata[];
  capabilities: string[];
  health: { healthy: boolean; last_check: string };
}
```
```

---

## 🎯 Testing Protocol (Post-Fix)

### Test 1: Schema Correctness
```bash
# Compare schema to actual response
curl http://127.0.0.1:11430/api/discovery > actual.json
curl http://127.0.0.1:49428/__shimmy__/schema > schema.json
# Manually verify DiscoverySnapshot definition matches actual.json structure
```

### Test 2: Theme Discovery
```bash
# Start stack
./target/release/shimmy dev shimmy-default --verify
# Check browser console - should have NO errors
# Discovery should succeed
# Models should load
```

### Test 3: End-to-End Chat
```bash
# Take screenshot
node theme-tester/tester.js screenshot http://localhost:8080
# Click model
node theme-tester/tester.js click http://localhost:8080 ".model-card:first-child"
# Send chat
node theme-tester/tester.js type-send http://localhost:8080 "input" "button" "Hello world"
# Verify streaming response
```

---

## 📈 Impact Assessment

### Current State
- ✅ Backend: 100% functional
- ❌ Schema: 0% accurate
- ❌ Generated themes: 0% working
- ⏸️ Multi-theme vision: Blocked

### Post-Fix State (Estimated)
- ✅ Backend: 100% functional (no changes)
- ✅ Schema: 100% accurate (aligned with reality)
- ✅ Generated themes: 100% working (correct types)
- ✅ Multi-theme vision: Unblocked

### Time Estimate
- Fix #1 (Align schema): 30-60 minutes
- Fix #2 (Regenerate schema): 2 minutes
- Fix #3 (Regenerate types): 2 minutes
- Fix #4 (Regenerate theme): 5 minutes
- Fix #5 (Update docs): 15 minutes
- Testing: 30 minutes

**Total:** 1-2 hours to full working state

---

## 🚨 Critical Insight

**The Problem:** Two sources of truth living in parallel:
1. `src/discovery/http.rs` - The REAL implementation (works)
2. `console/src/contract/mod.rs` - The THEORETICAL schema (fantasy)

**The Solution:** Make contract/mod.rs import and re-export types from discovery/http.rs instead of redefining them.

**Why This Happened:** Claude Code's 10-agent implementation likely:
1. Created schema from spec/imagination
2. Never validated against running backend
3. No integration test catching the mismatch

**Prevention:** Add integration test that:
1. Starts shimmy backend
2. Fetches `/api/discovery`
3. Validates response against exported schema
4. Fails build if mismatch detected

---

**End of Audit Report**
