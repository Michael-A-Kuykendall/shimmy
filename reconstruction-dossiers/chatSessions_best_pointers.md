# Best Pointers to “whole cloth” writes (chatSessions)

This table lists the earliest occurrence per file of a high-signal marker on the *same JSON line* as the filename.
Use these pointers to jump into the originating chat session JSON and extract the full patch/heredoc/code block.

| Target | apply_patch pointer | heredoc pointer | code_fence pointer |
|---|---|---|---|
| `src/api/token_endpoints.rs` |  |  | `3b7e7706-af34-4381-ad8d-bdb79c060603.json`:441926 |
| `src/api.rs` | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:29251 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:88769 | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:228992 |
| `src/api_errors.rs` |  | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:91991 |  |
| `src/bin/create_realistic_safetensors.rs` |  |  | `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:394543 |
| `src/bin/create_test_safetensors.rs` |  |  | `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:394543 |
| `src/bin/shimmy.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:341399 | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2576461 | `d10e87b9-842f-48f0-b494-73d38dff62fe.json`:315697 |
| `src/cache/response_cache.rs` |  |  |  |
| `src/cli.rs` | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:40300 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:88505 | `89cccbc3-29bd-4713-bafa-edab74cafebb.json`:86193 |
| `src/discovery/auto_register.rs` |  | `38a48b57-84bc-46e1-a313-a29224819811.json`:2119194 | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:773371 |
| `src/discovery/comprehensive_public_tests.rs` |  |  |  |
| `src/discovery/frontend.rs` |  | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2576461 | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:275558 |
| `src/discovery/handlers.rs` |  | `38a48b57-84bc-46e1-a313-a29224819811.json`:2119194 | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:1210435 |
| `src/discovery/http.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2576461 | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:1210435 |
| `src/discovery/integration_tests.rs` |  |  | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:99720 |
| `src/discovery/ipc.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:518471 |  | `d10e87b9-842f-48f0-b494-73d38dff62fe.json`:411797 |
| `src/discovery/ipc_comprehensive_tests.rs` |  |  |  |
| `src/discovery/leader.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:518471 | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2576461 | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:773371 |
| `src/discovery/leader_comprehensive_tests.rs` |  |  |  |
| `src/discovery/mod.rs` | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:29251 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:233535 | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:79200 |
| `src/discovery/performance_tests.rs` |  |  |  |
| `src/discovery/protocol.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2576461 | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:505195 |
| `src/discovery/resource_benchmark_tests.rs` |  |  |  |
| `src/discovery/security_tests.rs` |  |  | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:773371 |
| `src/discovery/service.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 | `38a48b57-84bc-46e1-a313-a29224819811.json`:2119194 | `54696d1b-c02a-4022-b63e-37b8c3ad58bc.json`:771397 |
| `src/discovery/unified.rs` |  |  | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:779261 |
| `src/discovery_invariants_only.rs` |  |  | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:889801 |
| `src/discovery_old_backup.rs` |  |  | `38a48b57-84bc-46e1-a313-a29224819811.json`:3742871 |
| `src/dispatcher.rs` |  |  |  |
| `src/engine/adapter.rs` | `bc90421a-626f-4c2c-8c72-1eb5aeab2b71.json`:594839 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:117470 | `89cccbc3-29bd-4713-bafa-edab74cafebb.json`:86193 |
| `src/engine/huggingface.rs` |  |  | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:99720 |
| `src/engine/llama.rs` | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:29059 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:88769 | `89cccbc3-29bd-4713-bafa-edab74cafebb.json`:86193 |
| `src/engine/mlx.rs` |  |  | `89cccbc3-29bd-4713-bafa-edab74cafebb.json`:44438 |
| `src/engine/mod.rs` | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:29251 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:233535 | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:79200 |
| `src/frontend/64bit/aga_app.rs` |  |  | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:773371 |
| `src/frontend/64bit/websocket_client.rs` |  |  | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:773371 |
| `src/frontend/cyberpunk.rs` |  |  | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:975962 |
| `src/http_adapter.rs` | `bc90421a-626f-4c2c-8c72-1eb5aeab2b71.json`:594839 | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2576461 | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:1154686 |
| `src/invariant_ppt.rs` |  |  | `d10e87b9-842f-48f0-b494-73d38dff62fe.json`:411797 |
| `src/main_integration.rs` |  | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:92006 | `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:394543 |
| `src/metrics.rs` |  | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:91991 | `3b7e7706-af34-4381-ad8d-bdb79c060603.json`:384287 |
| `src/model_manager.rs` |  | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2576461 | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:79200 |
| `src/model_registry.rs` | `54696d1b-c02a-4022-b63e-37b8c3ad58bc.json`:1013178 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:88505 | `89cccbc3-29bd-4713-bafa-edab74cafebb.json`:86193 |
| `src/observability/mod.rs` | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:29251 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:233535 | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:79200 |
| `src/openai_compat.rs` |  | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:160482 | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:229664 |
| `src/orchestrator/discovery_watcher.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 |  | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:628658 |
| `src/orchestrator/license.rs` |  |  | `7d010fce-637d-4da6-a863-03a9ab15f3b4.json`:13180 |
| `src/orchestrator/lifecycle.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 |  | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:628658 |
| `src/orchestrator/supervisor.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 |  | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:628658 |
| `src/orchestrator/verification.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 |  | `89cccbc3-29bd-4713-bafa-edab74cafebb.json`:86193 |
| `src/port_manager.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:625672 |  | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:657815 |
| `src/preloading.rs` |  |  | `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:380248 |
| `src/rustchain_compat.rs` |  | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:91976 | `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:380248 |
| `src/safetensors_adapter.rs` |  | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:117470 | `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:380248 |
| `src/server.rs` | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:47292 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:88769 | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:229832 |
| `src/test_utils.rs` |  |  |  |
| `src/token_meter.rs` |  |  | `3b7e7706-af34-4381-ad8d-bdb79c060603.json`:441926 |
| `src/tools.rs` |  |  | `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:380248 |
