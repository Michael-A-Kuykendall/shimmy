# Reconstruction Playbook (Shimmy)

This repo contains multiple evidence sources that can be used to reconstruct zeroed/null-byte source files.

## Evidence Sources (in this repo)

- `zeroed_files_inventory.txt`
  - Ground-truth list of files that must be rebuilt.
- `SHIMMY_CHAT_LOGS_COMPLETE.md`
  - Consolidated Copilot chat history. This is the most likely place to find full file bodies (often pasted as code fences or created via heredoc snippets in the chat).
- `logs-supplement/vscode_logs_hits_repos_shimmy.txt`
- `logs-supplement/vscode_logs_hits_key_files.txt`
- `logs-supplement/vscode_logs_hits_actions.txt`
  - Correlation logs that record tool calls, terminal commands, and file/path references.

## Systematic Workflow (per file)

1. **Create an evidence dossier**
   - Run:
     - `python tools/reconstruction/extract_evidence.py --target src/server.rs`
   - This writes a markdown dossier under `reconstruction-dossiers/`.

2. **Extract the best candidate for the file body**
   - Prefer the latest *complete file dump* (heredoc or full code fence).
   - Fall back to reconstructing from `apply_patch` blocks if the chat only contains patches.

3. **Recreate the file**
   - Write the reconstructed content back into the workspace file.

4. **Update the register**
   - Mark the file’s status + evidence links in `CONSOLE_RECREATION_LOG.md`.

5. **Compile/verify (when you request it)**
   - The user controls runtime. Once a coherent set of base files exists, run targeted `cargo check`/tests incrementally.

## Notes on Heredoc-heavy sessions

If you find a full file body, prefer that over partial patches.

Many files were created using patterns like:
- `cat > path/to/file <<'EOF'` … `EOF`

These can appear in either:
- the chat transcript (assistant recommending exact commands), or
- terminal logs (commands actually executed)

The extractor script is designed to capture both kinds of evidence context, even if it can’t perfectly “understand” every session.
