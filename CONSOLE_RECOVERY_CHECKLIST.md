# CONSOLE FEATURE RECOVERY CHECKLIST
## Zeroed Console Files (23 files requiring recovery)

### Core Infrastructure
- [ ] `console/src/lib.rs` - Main library entry point
- [ ] `console/src/config.rs` - Configuration management

### WebSocket Layer
- [ ] `console/src/websocket/mod.rs` - WebSocket module

### License System
- [ ] `console/src/license/client.rs` - License client implementation

### Tools System (13 files)
- [ ] `console/src/tools/mod.rs` - Tools module
- [ ] `console/src/tools/analysis.rs` - Code analysis tool
- [ ] `console/src/tools/analysis_tests.rs` - Analysis tool tests
- [ ] `console/src/tools/command.rs` - Command execution tool
- [ ] `console/src/tools/command_git_tests.rs` - Git command tests
- [ ] `console/src/tools/docs_tests.rs` - Documentation tests
- [ ] `console/src/tools/file_ops.rs` - File operations tool
- [ ] `console/src/tools/image.rs` - Image processing tool
- [ ] `console/src/tools/system.rs` - System operations tool

### Commands (5 files)
- [ ] `console/src/commands/analyze.rs` - Analyze command
- [ ] `console/src/commands/chat.rs` - Chat command
- [ ] `console/src/commands/edit.rs` - Edit command
- [ ] `console/src/commands/sessions.rs` - Session management command

### Adapters
- [ ] `console/src/adapters/ws_adapter.rs` - WebSocket adapter

### Tests (4 files)
- [ ] `console/tests/integration_tests.rs` - Integration tests
- [ ] `console/tests/model_helpers.rs` - Model helper tests
- [ ] `console/tests/websocket_chat_tests.rs` - WebSocket chat tests
- [ ] `console/src/bin/system_test.rs` - System test binary

## Recovery Sources
- Primary: `reconstruction-dossiers/chatSessions_hits.txt` (59M file with recovered content)
- Secondary: `SHIMMY_CHAT_LOGS_COMPLETE.md` (243K consolidated logs)
- Tertiary: Other branches/git history for inference

## Recovery Strategy
1. Search recovery data for complete file content (prefer heredoc blocks `<< 'EOF'`...`EOF`)
2. Extract and restore complete file
3. Mark checkbox only when full content recovered from known source
4. For missing files: use inference from other branches or partial reconstructions

## Progress Tracking
- Files recovered: 0/23
- Files requiring inference: TBD
- Last updated: $(date)</content>
<parameter name="filePath">c:\Users\micha\repos\shimmy\CONSOLE_RECOVERY_CHECKLIST.md