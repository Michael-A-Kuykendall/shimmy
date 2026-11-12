# Security policy — Shimmy 2.6.7

## Supported versions

Security fixes target the latest maintainer-approved 2.6.x patch, currently **released Shimmy 2.6.7 / Airframe 0.4.6** (November 12, 2026). Earlier 2.x users should update; 1.x is unsupported. No long-term-support or universal backport guarantee is made. The security review applies to the exact examined dependency graph and does not certify untested platforms or model configurations.

## Dependency review scope

The coordinated release audit reviewed actual compiled default and fast graphs,
official registry sources and lockfiles. The accepted set includes Tokio 1.53.3,
rustls 0.23.46, rustls-webpki 0.103.16 and webpki-roots 1.0.10.
The recorded audit found zero applicable active vulnerability/unsoundness
advisories in those graphs at audit time. This is not complete security clearance
or a promise against newly disclosed advisories.

Inactive lockfile findings remain visible: Quinn RUSTSEC-2026-0343,
RUSTSEC-2026-0340 and RUSTSEC-2026-0338; inactive Paste RUSTSEC-2024-0436
unmaintained warning. They are excluded only from active-graph claims, not
suppressed or declared fixed. The final authorization record must retain fresh
audit classifications for the exact published candidate.

## Deployment boundaries

Non-loopback binds require SHIMMY_API_KEY; browser access requires explicit
SHIMMY_ALLOWED_ORIGINS. Use TLS and restrict network access.
See [configuration](docs/CONFIGURATION.md) and [Docker instructions](README-DOCKER.md).
Health is credential-exempt but origin checks still apply. Keys, model files,
prompts and diagnostic captures should be handled as sensitive data.
Untrusted GGUF metadata has checked parsing/allocation limits, not a universal
resource-exhaustion guarantee. See
[ShimmyTok parser security](https://github.com/Michael-A-Kuykendall/shimmytok/blob/master/docs/GGUF_SECURITY.md).

Numerical fixture evidence does not prove host/driver stability. Both WSL device
destruction and active-generation device loss remain unresolved; Metal/M4 and
other unexecuted native GPU platforms are NOT_RUN. INT4 and experimental kernels remain uncertified and must not be deployed as certified configurations.

## Private vulnerability reporting

Do not file exploitable vulnerability details in public issues.
Use [GitHub private vulnerability reporting](https://github.com/Michael-A-Kuykendall/shimmy/security/advisories/new)
or email michaelallenkuykendall@gmail.com with SECURITY in the subject.
Include version/commit, platform, reproduction, impact and sanitized logs.
Response and remediation depend on severity and maintainer availability;
no guaranteed response-time SLA is offered.
