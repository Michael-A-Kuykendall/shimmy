# Deployment scope — Shimmy 2.6.6

This release is crates.io-only. No new hosted deployment, image, native binary,
Homebrew, Scoop or npm package is produced by publishing those crates.

The tracked cloud and Compose files, and files produced by shimmy init, are
**historical scaffolds, not current deployment instructions**. They retain
settings/images/ports that must be reviewed and replaced for a real deployment.
Do not deploy them unchanged, assume an image exists, or treat generated
Kubernetes/cloud configuration as tested GPU support.

Use the current [local Docker example](../README-DOCKER.md) or
[authenticated server quickstart](../docs/quickstart.md). Explicitly bind inside
containers, expose host ports on loopback, require a private API key, mount only
the needed model directory read-only and configure TLS for remote use.
Inspect device/runtime access for the actual platform. Containers, cloud
platforms, multi-GPU and native GPU correctness remain NOT_RUN here.

The source-built image health command uses curl and port11435. /health does not
prove a model loaded. Legacy template output may use a different tool/port;
its existence is not validation. The final authorization packet excludes these
distribution surfaces until separately verified and approved.
