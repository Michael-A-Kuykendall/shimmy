# Schema-Driven Theme Architecture Implementation Plan

**Goal**: Implement schema-first architecture for Shimmy themes with automatic theme generation

**Status**: Planning Phase
**Target**: Testable Alpha → Validate against Amiga theme → Auto-generate default theme

---

## Phase 1: Core Schema Infrastructure (Rust Backend)

### 1.1 Add Dependencies
- [ ] Add `schemars = "0.8"` to `console/Cargo.toml`
- [ ] Add `schemars` to root `Cargo.toml` if needed
- [ ] Verify `serde` has `derive` feature enabled
- [ ] Run `cargo build` to verify dependencies resolve

### 1.2 Define WebSocket Message Protocol Structs
- [ ] Create `console/src/contract/mod.rs` module
- [ ] Define `WebSocketMessage` enum with all message types
- [ ] Define `GetModelsRequest` struct
- [ ] Define `ModelsListResponse` struct with model metadata
- [ ] Define `SelectModelRequest` struct
- [ ] Define `ModelSelectedResponse` struct
- [ ] Define `ChatRequest` struct
- [ ] Define `ChatTokenResponse` struct (streaming)
- [ ] Define `GenerationCompleteResponse` struct
- [ ] Define `GetMetricsRequest` struct
- [ ] Define `MetricsResponse` struct
- [ ] Add `#[derive(Serialize, Deserialize, JsonSchema)]` to all structs
- [ ] Document each struct with `/// ` comments for schema descriptions

### 1.3 Define Discovery Protocol Structs
- [ ] Define `DiscoveryResponse` struct (HTTP discovery on 11430)
- [ ] Define `BackendInfo` struct with validation fields
- [ ] Add `#[derive(Serialize, Deserialize, JsonSchema)]`

### 1.4 Create Consolidated Contract Schema
- [ ] Define `ShimmyFrontendContract` root struct
- [ ] Add `websocket_messages: Vec<MessageSpec>` field
- [ ] Add `discovery: DiscoverySpec` field
- [ ] Add `streaming: StreamingSpec` field
- [ ] Add `required_behaviors: Vec<BehaviorSpec>` field
- [ ] Implement schema generation function using `schema_for!()`

### 1.5 Add Schema Introspection Endpoint
- [ ] Add route `GET /__shimmy__/schema` to console HTTP adapter
- [ ] Return generated JSON schema with proper CORS headers
- [ ] Add route `GET /__shimmy__/schema/version` returning semver
- [ ] Test endpoint accessibility after shimmy starts

---

## Phase 2: Schema Export & TypeScript Generation

### 2.1 Schema Export Script
 [ ] Produce the canonical schema using the orchestrator HTTP endpoint or shimmy dev

### 2.2 TypeScript Type Generation
 [ ] Add npm script "generate-contract": "curl http://localhost:11435/__shimmy__/schema -o frontend contract JSON (use `/__shimmy__/schema` or orchestrator export) && node scripts/generate-types.js"
- [ ] Generate TypeScript definitions
- [ ] Output to `theme-types/shimmy-contract.d.ts`
- [ ] Add JSDoc comments from schema descriptions
- [ ] Test generated types compile with `tsc --noEmit`

### 2.3 Build Integration
 [ ] Ensure `generate-contract` uses the orchestrator schema endpoint (e.g. `curl http://localhost:11435/__shimmy__/schema -o frontend contract JSON (use `/__shimmy__/schema` or orchestrator export) && node scripts/generate-types.js`)
- [ ] Add to root `Makefile` as `make contract`
- [ ] Document in README.md

---

## Phase 3: Schema-Driven Validator

### 3.1 Validator Infrastructure
- [ ] Create `theme-validator/package.json`
- [ ] Install dependencies: `ajv`, `ajv-formats`, `ws`, `playwright`
- [ ] Create `theme-validator/validator.js` main entry point
- [ ] Load schema from `frontend contract JSON (use `/__shimmy__/schema` or orchestrator export)`
- [ ] Compile Ajv validator from schema

### 3.2 Discovery Test
- [ ] Test HTTP discovery on 127.0.0.1:11430
- [ ] Validate response against `DiscoveryResponse` schema
- [ ] Extract backend port from validated response
- [ ] Report: Discovery working ✅ or ❌

### 3.3 WebSocket Connection Test
- [ ] Connect to `ws://127.0.0.1:{port}/ws/console`
- [ ] Set 5-second connection timeout
- [ ] Report: WebSocket connected ✅ or ❌

### 3.4 Message Protocol Tests
- [ ] Send `get_models` → validate response against `ModelsListResponse` schema
- [ ] Verify at least 1 model present
- [ ] Send `select_model` → validate response against `ModelSelectedResponse` schema
- [ ] Send `chat` → validate streaming tokens against `ChatTokenResponse` schema
- [ ] Verify `generation_complete` received and matches schema
- [ ] Each test reports: ✅ PASS or ❌ FAIL with schema validation errors

### 3.5 Theme Browser Tests (Playwright)
- [ ] Launch headless browser to theme URL
- [ ] Take screenshot of initial state
- [ ] Verify models displayed (visual + DOM check)
- [ ] Click model CONNECT button
- [ ] Verify chat input enabled
- [ ] Type message and send
- [ ] Verify response appears
- [ ] Verify input re-enables after generation_complete
- [ ] Each step reports: ✅ PASS or ❌ FAIL

### 3.6 Validator CLI
- [ ] Add CLI args: `--theme-url`, `--backend-port`, `--json-output`
- [ ] Generate JSON report with all test results
- [ ] Generate human-readable summary
- [ ] Exit code 0 if all required tests pass, 1 otherwise
- [ ] Add task to `tasks.json`: `VALIDATE_THEME`

---

## Phase 4: Theme Generator

### 4.1 Generator Infrastructure
- [ ] Create `theme-generator/package.json`
- [ ] Install dependencies: `ejs`, `commander`, `chalk`
- [ ] Create `theme-generator/generate.js` main entry point
- [ ] Load schema from `frontend contract JSON (use `/__shimmy__/schema` or orchestrator export)`

### 4.2 Template System
- [ ] Create `theme-generator/templates/react-vite/` directory structure
- [ ] Create base templates:
  - [ ] `package.json.ejs`
  - [ ] `vite.config.ts.ejs`
  - [ ] `tsconfig.json.ejs`
  - [ ] `index.html.ejs`
  - [ ] `src/main.tsx.ejs`
  - [ ] `src/App.tsx.ejs`
  - [ ] `src/hooks/useDiscovery.ts.ejs`
  - [ ] `src/hooks/useWebSocket.ts.ejs`
  - [ ] `src/components/ModelChooser.tsx.ejs`
  - [ ] `src/components/Chat.tsx.ejs`
  - [ ] `src/components/Metrics.tsx.ejs`
  - [ ] `README.md.ejs`

### 4.3 Code Generation Logic
- [ ] Parse schema to extract message types
- [ ] Generate TypeScript interfaces for each message type
- [ ] Generate hook for each WebSocket message pattern
- [ ] Generate component for each required UI surface
- [ ] Wire components together in App.tsx
- [ ] Add discovery client integration
- [ ] Add WebSocket connection with reconnection logic

### 4.4 CLI Interface
- [ ] Add command: `shimmy-theme-gen new <theme-name>`
- [ ] Add flag: `--template react-vite|react-nextjs|vanilla`
- [ ] Add flag: `--contract-path <path>` (default: auto-fetch from running shimmy)
- [ ] Create output directory structure
- [ ] Copy static assets
- [ ] Render all EJS templates
- [ ] Run `npm install` in generated theme
- [ ] Print success message with next steps

### 4.5 Generated Theme Structure
```
generated-theme/
├── package.json (dependencies from schema)
├── vite.config.ts
├── tsconfig.json
├── index.html
├── src/
│   ├── main.tsx (entry point)
│   ├── App.tsx (wired layout)
│   ├── types/
│   │   └── shimmy-contract.d.ts (copied from theme-types)
│   ├── hooks/
│   │   ├── useDiscovery.ts (discovery client)
│   │   ├── useWebSocket.ts (WS connection)
│   │   ├── useModels.ts (get_models logic)
│   │   ├── useChat.ts (chat streaming logic)
│   │   └── useMetrics.ts (metrics polling)
│   └── components/
│       ├── ModelChooser.tsx (auto-wired to schema)
│       ├── Chat.tsx (streaming support)
│       ├── Metrics.tsx (metric display)
│       └── styles.css (minimal baseline)
└── README.md (customization guide)
```

---

## Phase 5: Alpha Testing & Validation

### 5.1 Test Against Running Backend
 - [ ] Run `./target/release/shimmy dev` to start shimmy + 32bit-interface
- [ ] Run `make contract` to generate fresh schema
- [ ] Verify `frontend contract JSON (use `/__shimmy__/schema` or orchestrator export)` exists and is valid JSON
- [ ] Verify `theme-types/shimmy-contract.d.ts` generated correctly
- [ ] Run `VALIDATE_THEME` task against 32bit-interface
- [ ] Review validation report - expect failures due to streaming not implemented

### 5.2 Update Backend to Match Contract
- [ ] Review schema for any drift from actual backend behavior
- [ ] Update Rust structs if schema doesn't match reality
- [ ] Re-export schema
- [ ] Re-run validation - should now pass backend tests

### 5.3 Test Against Amiga Theme
- [ ] Rename `themes/amiga-ai-interface/` to something permanent
- [ ] Run validator against Amiga theme
- [ ] Document which tests pass/fail
- [ ] Update Amiga theme to fix failures (if any)
- [ ] Re-run validator - expect 100% pass

### 5.4 Generate Default Theme
- [ ] Run: `shimmy-theme-gen new shimmy-default --template react-vite`
- [ ] Start generated theme: `cd themes/shimmy-default && npm run dev`
- [ ] Test manually: open browser, select model, send chat
- [ ] Run validator against generated theme
- [ ] Fix any issues in generator templates
- [ ] Re-generate and re-test until 100% pass

---

## Phase 6: Documentation & Integration

### 6.1 Update Documentation
- [ ] Update `FRONTEND_CONTRACT.md` to reference schema as source of truth
- [ ] Add "Schema-First Architecture" section
- [ ] Document `/__shimmy__/schema` endpoint
- [ ] Document validator usage
- [ ] Document theme generator usage
- [ ] Add examples of generated TypeScript types

### 6.2 Developer Experience
- [ ] Add VS Code task: `GENERATE_CONTRACT`
- [ ] Add VS Code task: `GENERATE_THEME`
 - [ ] Update orchestrator (`shimmy dev`) to regenerate contract if schema changed
- [ ] Add pre-commit hook to validate schema consistency
- [ ] Document full workflow in README.md

### 6.3 CI/CD Integration
- [ ] Add GitHub Actions workflow to validate schema on push
- [ ] Add workflow to generate and commit types on schema change
- [ ] Add workflow to run validator against example themes
- [ ] Fail CI if validator reports failures

---

## Success Criteria

### Alpha Completion Checklist
- [ ] Schema endpoint accessible and returns valid JSON Schema
- [ ] TypeScript types generated from schema compile successfully
- [ ] Validator runs and produces report (JSON + human-readable)
- [ ] Validator passes all tests against running backend
- [ ] Amiga theme validated successfully
- [ ] Generated default theme works end-to-end
- [ ] Generated theme passes validator 100%
- [ ] Documentation complete and accurate

### What Success Looks Like
1. **Zero Manual Sync**: Change Rust code → schema updates automatically → types regenerate → validator uses new schema
2. **Instant Validation**: Run one command to verify any theme is contract-compliant
3. **5-Minute Themes**: Generate working theme skeleton in <5 minutes
4. **No Drift**: Contract, code, validator, and types always aligned
5. **Self-Documenting**: Schema contains all information theme developers need

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Schema too verbose | Start minimal, add details incrementally |
| Validator too strict | Use required vs optional fields appropriately |
| Generated themes too generic | Provide multiple templates + customization guide |
| Build process too complex | Document step-by-step, add to Makefile |
| CI slowdown | Cache dependencies, parallelize jobs |

---

## Timeline Estimate

- **Phase 1**: 4-6 hours (Rust schema infrastructure)
- **Phase 2**: 2-3 hours (TypeScript generation)
- **Phase 3**: 4-5 hours (Schema-driven validator)
- **Phase 4**: 6-8 hours (Theme generator)
- **Phase 5**: 3-4 hours (Alpha testing)
- **Phase 6**: 2-3 hours (Documentation)

**Total**: ~20-30 hours to testable alpha

---

## Next Steps

1. Review this plan carefully
2. Identify any missing pieces
3. Prioritize phases if needed
4. Begin Phase 1.1: Add schemars dependency
