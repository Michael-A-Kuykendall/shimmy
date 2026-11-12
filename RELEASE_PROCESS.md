# Coordinated release process — current Shimmy release workflow

Publication is a **human approval** decision on exact source, package bytes,
checksums, metadata and support claims. This document does not authorize it.

## Required candidate

For the published November 12, 2026 release, ShimmyTok 0.8.5, libfse 0.1.4 and airframe-observe 0.1.4 were reused unchanged. Airframe 0.4.6 was published before Shimmy 2.6.7 in dependency order. Future releases must establish their own exact versions and approved artifact identities. The private authorization packet identifies
exact public parents, clean source commits, tags, .crate archives/checksums,
registry request bodies, dependency/security/license results and rollback limits.

Documentation changes invalidate old source/archive hashes. Repackage and
reverify the exact new bytes. Preserve old evidence as history; do not pretend
it describes byte-identical new artifacts.

## Before any public operation

1. Pass local source, docs, license, packaging, clean-consumer, dependency
   security and applicable CLI/server gates against the exact candidate.
2. Verify live registry version availability, tag absence and public ancestry.
3. Obtain an explicit human YES for the concrete authorization packet.
4. Execute only the approved source and per-crate registry operations, verifying
   each actual registry checksum/consumer before publishing downstream crates.

Public source updates, tags, crates.io uploads, hosted Actions, GitHub releases
and container/image publication are separate actions. No script, green CI run,
private backup or version bump implicitly authorizes them.
Use the exact approved commands and immutable archives recorded in the packet,
not an obsolete coordinated script or a fresh unreviewed repack.

## Distribution scope and rollback

A crates.io-only release does not create new GitHub binary assets, Docker images,
Homebrew, Scoop or npm packages. Do not imply otherwise in release notes.
Hosted tag workflows can create additional artifacts: inspect their triggers
before any approved source/tag push, and do not silently activate them.

Registry publication is effectively immutable. An occupied version with
different bytes is a stop condition. Yanking can discourage resolution but
does not delete or replace published bytes. Incident response must identify
affected versions, revoke exposed credentials if needed, publish a separately
reviewed correction and communicate scope honestly; do not move release tags.

See [release gates](RELEASE_GATES_CHECKLIST.md), [security](SECURITY.md)
and [CHANGELOG](CHANGELOG.md).
