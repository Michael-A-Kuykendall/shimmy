# docs/internal

Purpose: a single, intentional place to move or archive documentation that is no longer authoritative.

Why this exists
- To reduce noisy / conflicting guidance across the repository that confuses maintainers and automated tools.
- To keep history (for data mining) while making the root docs surface clean and authoritative.

Guidelines
- Do not delete files outright — move them here (preserve file metadata if necessary).
- Add a small deprecation header to each moved file explaining the replacement and date.
- Metadata fields to add when archiving a doc (append to top):
  - Deprecated: YYYY-MM-DD
  - Replaced-by: <path-to-authoritative-doc>
  - Reason: short explanation of why moved

Acceptance
- The migration plan uses this directory as the archive target during the hostile audit phase.
