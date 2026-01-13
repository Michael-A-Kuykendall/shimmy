# chatSessions first/last pointers per file

Computed from `reconstruction-dossiers/chatSessions_hits.txt` (single rg scan).

| Target | apply_patch (first→last) | heredoc (first→last) | code_fence (first→last) |
|---|---|---|---|
| `src/api/token_endpoints.rs` |  |  | `3b7e7706-af34-4381-ad8d-bdb79c060603.json`:441926 → `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:277866 |
| `src/api.rs` | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:29251 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:518471 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:88769 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:228992 → `38a48b57-84bc-46e1-a313-a29224819811.json`:6124467 |
| `src/api_errors.rs` |  | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:91991 → `f1afcde9-1501-460e-8d28-21c2151725a6.json`:118252 |  |
| `src/bin/create_realistic_safetensors.rs` |  |  | `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:394543 → `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:394543 |
| `src/bin/create_test_safetensors.rs` |  |  | `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:394543 → `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:394543 |
| `src/bin/shimmy.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:341399 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:3167205 | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2576461 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `d10e87b9-842f-48f0-b494-73d38dff62fe.json`:315697 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:3157727 |
| `src/cache/response_cache.rs` |  |  |  |
| `src/cli.rs` | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:40300 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:3167205 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:88505 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `89cccbc3-29bd-4713-bafa-edab74cafebb.json`:86193 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:3157727 |
| `src/discovery/auto_register.rs` |  | `38a48b57-84bc-46e1-a313-a29224819811.json`:2245648 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:773371 → `38a48b57-84bc-46e1-a313-a29224819811.json`:3742871 |
| `src/discovery/comprehensive_public_tests.rs` |  |  |  |
| `src/discovery/frontend.rs` |  | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2576461 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:275558 → `38a48b57-84bc-46e1-a313-a29224819811.json`:5904194 |
| `src/discovery/handlers.rs` |  | `38a48b57-84bc-46e1-a313-a29224819811.json`:2245648 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:1210435 → `38a48b57-84bc-46e1-a313-a29224819811.json`:3742871 |
| `src/discovery/http.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2576461 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:1210435 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609589 |
| `src/discovery/integration_tests.rs` |  |  | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:99720 → `38a48b57-84bc-46e1-a313-a29224819811.json`:463813 |
| `src/discovery/ipc.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:518471 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 |  | `d10e87b9-842f-48f0-b494-73d38dff62fe.json`:411797 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609589 |
| `src/discovery/ipc_comprehensive_tests.rs` |  |  |  |
| `src/discovery/leader.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:518471 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2576461 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:773371 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609589 |
| `src/discovery/leader_comprehensive_tests.rs` |  |  |  |
| `src/discovery/mod.rs` | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:29251 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:3167205 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:233535 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:79200 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:3157727 |
| `src/discovery/performance_tests.rs` |  |  |  |
| `src/discovery/protocol.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2576461 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:505195 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609589 |
| `src/discovery/resource_benchmark_tests.rs` |  |  |  |
| `src/discovery/security_tests.rs` |  |  | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:773371 → `7d010fce-637d-4da6-a863-03a9ab15f3b4.json`:267620 |
| `src/discovery/service.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 | `38a48b57-84bc-46e1-a313-a29224819811.json`:2245648 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `54696d1b-c02a-4022-b63e-37b8c3ad58bc.json`:771397 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609589 |
| `src/discovery/unified.rs` |  |  | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:779261 → `38a48b57-84bc-46e1-a313-a29224819811.json`:3742871 |
| `src/discovery_invariants_only.rs` |  |  | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:889801 → `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:889801 |
| `src/discovery_old_backup.rs` |  |  | `38a48b57-84bc-46e1-a313-a29224819811.json`:3742871 → `38a48b57-84bc-46e1-a313-a29224819811.json`:3742871 |
| `src/dispatcher.rs` |  |  |  |
| `src/engine/adapter.rs` | `bc90421a-626f-4c2c-8c72-1eb5aeab2b71.json`:594839 → `54696d1b-c02a-4022-b63e-37b8c3ad58bc.json`:1013178 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:117470 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `89cccbc3-29bd-4713-bafa-edab74cafebb.json`:86193 → `38a48b57-84bc-46e1-a313-a29224819811.json`:6124467 |
| `src/engine/huggingface.rs` |  |  | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:99720 → `38a48b57-84bc-46e1-a313-a29224819811.json`:441901 |
| `src/engine/llama.rs` | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:29059 → `54696d1b-c02a-4022-b63e-37b8c3ad58bc.json`:1013178 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:88769 → `f1afcde9-1501-460e-8d28-21c2151725a6.json`:118261 | `89cccbc3-29bd-4713-bafa-edab74cafebb.json`:86193 → `38a48b57-84bc-46e1-a313-a29224819811.json`:6022574 |
| `src/engine/mlx.rs` |  |  | `89cccbc3-29bd-4713-bafa-edab74cafebb.json`:44438 → `38a48b57-84bc-46e1-a313-a29224819811.json`:3378405 |
| `src/engine/mod.rs` | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:29251 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:3167205 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:233535 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:79200 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:3157727 |
| `src/frontend/64bit/aga_app.rs` |  |  | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:773371 → `38a48b57-84bc-46e1-a313-a29224819811.json`:3742871 |
| `src/frontend/64bit/websocket_client.rs` |  |  | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:773371 → `38a48b57-84bc-46e1-a313-a29224819811.json`:5708039 |
| `src/frontend/cyberpunk.rs` |  |  | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:975962 → `38a48b57-84bc-46e1-a313-a29224819811.json`:4712866 |
| `src/http_adapter.rs` | `bc90421a-626f-4c2c-8c72-1eb5aeab2b71.json`:594839 → `bc90421a-626f-4c2c-8c72-1eb5aeab2b71.json`:610048 | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2576461 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:1154686 → `38a48b57-84bc-46e1-a313-a29224819811.json`:6124467 |
| `src/invariant_ppt.rs` |  |  | `d10e87b9-842f-48f0-b494-73d38dff62fe.json`:411797 → `38a48b57-84bc-46e1-a313-a29224819811.json`:224544 |
| `src/main_integration.rs` |  | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:92006 → `f1afcde9-1501-460e-8d28-21c2151725a6.json`:118252 | `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:394543 → `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:394543 |
| `src/metrics.rs` |  | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:91991 → `f1afcde9-1501-460e-8d28-21c2151725a6.json`:118261 | `3b7e7706-af34-4381-ad8d-bdb79c060603.json`:384287 → `38a48b57-84bc-46e1-a313-a29224819811.json`:6022469 |
| `src/model_manager.rs` |  | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2576461 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:79200 → `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:244199 |
| `src/model_registry.rs` | `54696d1b-c02a-4022-b63e-37b8c3ad58bc.json`:1013178 → `54696d1b-c02a-4022-b63e-37b8c3ad58bc.json`:1013178 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:88505 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `89cccbc3-29bd-4713-bafa-edab74cafebb.json`:86193 → `38a48b57-84bc-46e1-a313-a29224819811.json`:6061317 |
| `src/observability/mod.rs` | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:29251 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:3167205 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:233535 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:79200 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:3157727 |
| `src/openai_compat.rs` |  | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:160482 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:229664 → `38a48b57-84bc-46e1-a313-a29224819811.json`:6181884 |
| `src/orchestrator/discovery_watcher.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 |  | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:628658 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609589 |
| `src/orchestrator/license.rs` |  |  | `7d010fce-637d-4da6-a863-03a9ab15f3b4.json`:13180 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:1766937 |
| `src/orchestrator/lifecycle.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:3167205 |  | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:628658 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:3894508 |
| `src/orchestrator/supervisor.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 |  | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:628658 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:3894508 |
| `src/orchestrator/verification.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:3167205 |  | `89cccbc3-29bd-4713-bafa-edab74cafebb.json`:86193 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:3157727 |
| `src/port_manager.rs` | `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:625672 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:629302 |  | `0c9dd6c4-d7a1-44da-acf9-61ce5749738b.json`:657815 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609589 |
| `src/preloading.rs` |  |  | `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:380248 → `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:399324 |
| `src/rustchain_compat.rs` |  | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:91976 → `f1afcde9-1501-460e-8d28-21c2151725a6.json`:127320 | `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:380248 → `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:399324 |
| `src/safetensors_adapter.rs` |  | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:117470 → `f1afcde9-1501-460e-8d28-21c2151725a6.json`:118261 | `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:380248 → `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:399324 |
| `src/server.rs` | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:47292 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:986164 | `f1afcde9-1501-460e-8d28-21c2151725a6.json`:88769 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2609436 | `e77f0d7b-f524-428e-9169-58a47ef34ba9.json`:229832 → `b748dbe4-e6a4-4714-9737-bb0f4c3edb72.json`:2610290 |
| `src/test_utils.rs` |  |  |  |
| `src/token_meter.rs` |  |  | `3b7e7706-af34-4381-ad8d-bdb79c060603.json`:441926 → `54696d1b-c02a-4022-b63e-37b8c3ad58bc.json`:2096401 |
| `src/tools.rs` |  |  | `503c3f4b-1d06-4e9f-8d09-75f92d20ba83.json`:380248 → `7d010fce-637d-4da6-a863-03a9ab15f3b4.json`:651502 |
