#!/usr/bin/env python3
"""Extract reconstruction evidence for a target file.

This script is intentionally heuristic: the goal is to produce a per-file "dossier"
that collects all relevant chat/log context so the human can reconstruct the file
body deterministically.

Inputs (repo-local):
- SHIMMY_CHAT_LOGS_COMPLETE.md
- logs-supplement/vscode_logs_hits_repos_shimmy.txt
- logs-supplement/vscode_logs_hits_key_files.txt (optional)
- logs-supplement/vscode_logs_hits_actions.txt (optional)

Output:
- A markdown dossier under reconstruction-dossiers/

Only uses Python stdlib.
"""

from __future__ import annotations

import argparse
import os
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List, Optional, Sequence, Tuple


RE_HEREDOC_START = re.compile(
    r"(?P<prefix>^|\s)(?P<cmd>(cat|tee)\s+)(?P<redir>>+\s*)(?P<path>[^\s]+)\s*<<\s*['\"]?(?P<delim>[A-Za-z0-9_\-]+)['\"]?"
)
RE_APPLY_PATCH_START = re.compile(r"\*\*\* Begin Patch")
RE_APPLY_PATCH_END = re.compile(r"\*\*\* End Patch")
RE_UPDATE_FILE = re.compile(r"\*\*\* (Update|Add|Delete) File:\s+(?P<path>.+?)\s*$")
RE_CODE_FENCE = re.compile(r"^```")


@dataclass
class Snippet:
    source: str
    start_line: int
    end_line: int
    title: str
    lines: List[str]


def _read_lines(path: Path) -> List[str]:
    return path.read_text(encoding="utf-8", errors="replace").splitlines(True)


def _normalize_for_match(s: str) -> str:
    return s.replace("\\", "/").lower()


def _target_matchers(target: str) -> Tuple[str, str]:
    t_norm = _normalize_for_match(target)
    base_norm = _normalize_for_match(os.path.basename(target))
    return (t_norm, base_norm)


def _find_line_hits(lines: Sequence[str], needles: Sequence[str]) -> List[int]:
    hits: List[int] = []
    for i, line in enumerate(lines):
        hay = _normalize_for_match(line)
        if any(n in hay for n in needles if n):
            hits.append(i)
    return hits


def _capture_window(lines: Sequence[str], center: int, radius: int) -> Tuple[int, int, List[str]]:
    start = max(0, center - radius)
    end = min(len(lines), center + radius + 1)
    return start, end, list(lines[start:end])


def _is_inside_code_fence(lines: Sequence[str], idx: int) -> bool:
    fence_count = 0
    for i in range(0, idx + 1):
        if RE_CODE_FENCE.match(lines[i].rstrip("\r\n")):
            fence_count += 1
    return fence_count % 2 == 1


def _capture_code_fence_block(lines: Sequence[str], idx: int) -> Optional[Tuple[int, int, List[str]]]:
    # Find the nearest opening fence above idx and its closing fence below.
    start = idx
    while start >= 0 and not RE_CODE_FENCE.match(lines[start].rstrip("\r\n")):
        start -= 1
    if start < 0:
        return None

    end = start + 1
    while end < len(lines) and not RE_CODE_FENCE.match(lines[end].rstrip("\r\n")):
        end += 1
    if end >= len(lines):
        return None

    return start, end + 1, list(lines[start : end + 1])


def _capture_apply_patch_block(lines: Sequence[str], idx: int) -> Optional[Tuple[int, int, List[str]]]:
    # Expand to the whole *** Begin Patch ... *** End Patch block.
    start = idx
    while start >= 0 and not RE_APPLY_PATCH_START.search(lines[start]):
        start -= 1
    if start < 0:
        return None

    end = start
    while end < len(lines) and not RE_APPLY_PATCH_END.search(lines[end]):
        end += 1
    if end >= len(lines):
        return None

    return start, end + 1, list(lines[start : end + 1])


def _ranked_unique(snippets: Iterable[Snippet]) -> List[Snippet]:
    seen = set()
    out: List[Snippet] = []
    for s in snippets:
        key = (s.source, s.start_line, s.end_line, s.title)
        if key in seen:
            continue
        seen.add(key)
        out.append(s)
    return out


def extract_from_chat(chat_path: Path, target: str, window: int) -> List[Snippet]:
    lines = _read_lines(chat_path)
    t_norm, base_norm = _target_matchers(target)
    hit_idxs = _find_line_hits(lines, [t_norm, base_norm])

    snippets: List[Snippet] = []

    for idx in hit_idxs:
        # Prefer capturing full code fences if the match is inside one.
        if _is_inside_code_fence(lines, idx):
            block = _capture_code_fence_block(lines, idx)
            if block is not None:
                s, e, block_lines = block
                snippets.append(
                    Snippet(
                        source=str(chat_path.name),
                        start_line=s + 1,
                        end_line=e,
                        title="Chat: code fence containing match",
                        lines=block_lines,
                    )
                )
                continue

        # Capture apply_patch blocks if nearby.
        patch_block = _capture_apply_patch_block(lines, idx)
        if patch_block is not None:
            s, e, block_lines = patch_block
            snippets.append(
                Snippet(
                    source=str(chat_path.name),
                    start_line=s + 1,
                    end_line=e,
                    title="Chat: apply_patch block near match",
                    lines=block_lines,
                )
            )
            continue

        s, e, window_lines = _capture_window(lines, idx, window)
        snippets.append(
            Snippet(
                source=str(chat_path.name),
                start_line=s + 1,
                end_line=e,
                title="Chat: context window",
                lines=window_lines,
            )
        )

    return _ranked_unique(snippets)


def extract_from_log(log_path: Path, target: str, window: int) -> List[Snippet]:
    lines = _read_lines(log_path)
    t_norm, base_norm = _target_matchers(target)
    hit_idxs = _find_line_hits(lines, [t_norm, base_norm])

    snippets: List[Snippet] = []
    for idx in hit_idxs:
        s, e, window_lines = _capture_window(lines, idx, window)
        snippets.append(
            Snippet(
                source=str(log_path.name),
                start_line=s + 1,
                end_line=e,
                title="Log: context window",
                lines=window_lines,
            )
        )

    return _ranked_unique(snippets)


def write_dossier(out_path: Path, target: str, snippets: Sequence[Snippet]) -> None:
    out_path.parent.mkdir(parents=True, exist_ok=True)

    def w(line: str = ""):
        f.write(line + "\n")

    with out_path.open("w", encoding="utf-8", newline="\n") as f:
        w(f"# Reconstruction Dossier: `{target}`")
        w("")
        w("This file was auto-generated by `tools/reconstruction/extract_evidence.py`.")
        w("")
        w("## How to use")
        w("- Scan snippets for the latest full file body (often a heredoc or code fence).")
        w("- Prefer complete file dumps over incremental patches when possible.")
        w("- Once reconstructed, update the register in `CONSOLE_RECREATION_LOG.md`.")
        w("")

        if not snippets:
            w("## Snippets")
            w("No evidence snippets found for this target.")
            return

        w("## Snippets")
        for snip in snippets:
            w("")
            w(f"### {snip.title}")
            w(f"Source: `{snip.source}` lines {snip.start_line}-{snip.end_line}")
            w("```text")
            for line in snip.lines:
                f.write(line)
            if snip.lines and not snip.lines[-1].endswith("\n"):
                f.write("\n")
            w("```")


def main() -> int:
    parser = argparse.ArgumentParser(description="Extract evidence snippets for a given target file.")
    parser.add_argument("--target", required=True, help="Repo-relative path like src/server.rs")
    parser.add_argument(
        "--chat",
        default="SHIMMY_CHAT_LOGS_COMPLETE.md",
        help="Path to consolidated chat logs (default: SHIMMY_CHAT_LOGS_COMPLETE.md)",
    )
    parser.add_argument(
        "--logs",
        nargs="*",
        default=[
            "logs-supplement/vscode_logs_hits_repos_shimmy.txt",
            "logs-supplement/vscode_logs_hits_key_files.txt",
            "logs-supplement/vscode_logs_hits_actions.txt",
        ],
        help="One or more log-hit files to scan",
    )
    parser.add_argument("--window", type=int, default=25, help="Context lines before/after each hit")
    parser.add_argument(
        "--out",
        default=None,
        help="Output dossier path (default: reconstruction-dossiers/<target>.md with / replaced)",
    )

    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[2]
    target = args.target

    chat_path = (repo_root / args.chat).resolve()
    if not chat_path.exists():
        raise SystemExit(f"Chat log not found: {chat_path}")

    snippets: List[Snippet] = []
    snippets.extend(extract_from_chat(chat_path, target, args.window))

    for log_rel in args.logs:
        log_path = (repo_root / log_rel).resolve()
        if not log_path.exists():
            continue
        snippets.extend(extract_from_log(log_path, target, args.window))

    snippets = _ranked_unique(snippets)

    out_path: Path
    if args.out:
        out_path = (repo_root / args.out).resolve()
    else:
        safe_name = target.replace("\\", "/").replace("/", "__")
        out_path = repo_root / "reconstruction-dossiers" / f"{safe_name}.md"

    write_dossier(out_path, target, snippets)
    print(str(out_path))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
