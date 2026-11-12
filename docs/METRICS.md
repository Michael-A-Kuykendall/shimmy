# Metrics and observation — Shimmy 2.6.7

GET /metrics exposes the server's local MetricsCollector output.
GET /diag exposes diagnostics. Both require credentials when a key is configured.
Treat their output and other logs as potentially sensitive operational data.

The source also contains an opt-in TelemetryCollector with transmission code,
but the production server does not instantiate or call that sending path.
This is not a functioning CLI analytics service or evidence that an old
advertised analytics schema is emitted. Local /metrics and that unused remote
telemetry code are different paths. No blanket anonymity or data-exposure
guarantee applies to every diagnostic facility.

airframe-observe provides inference facts/observers and Dzero coordination.
Optional AIRFRAME_TRACE_TIMING reports scalar timings/counts; it does not certify
GPU math or guarantee better performance. Tensor captures and unrelated debug
logs have different content and costs; inspect/sanitize before sharing.

See [profiling](https://github.com/Michael-A-Kuykendall/airframe/blob/main/docs/PROFILING.md),
[configuration](CONFIGURATION.md) and [security](../SECURITY.md).
