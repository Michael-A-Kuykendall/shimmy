# Container packaging — Shimmy 2.6.6

The tracked Dockerfiles build the current Rust server and use port11435 and a
curl /health probe. This directory does not contain a separately certified
vision image or a supported Metal/Windows GPU emulator.

The coordinated crates.io release does not publish images. Build/run examples
and authentication requirements are in the current
[Docker guide](../../README-DOCKER.md). Image builds, vendor device access and
container inference are NOT_RUN for the scoped release; do not infer support
from a Dockerfile existing or a health endpoint returning success.

Existing cloud/init scaffolds are historical and must not be deployed unchanged.
No absent Dockerfile, private script or unpublished image is a release dependency.
