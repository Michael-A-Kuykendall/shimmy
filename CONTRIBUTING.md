# Contributing to Shimmy

Shimmy is open source, not open contribution. Unsolicited pull requests are not
accepted. Prospective co-maintainers must email
michaelallenkuykendall@gmail.com, follow the maintainer's directions and receive
approval before contributing. Issues and respectful discussion are welcome;
never put vulnerability details or credentials in public reports.

## Approved work

Use a focused feature branch, source-grounded design, explicit acceptance and
failure cases. Preserve tested inference contracts, exact fixtures and numerical
bounds. Do not claim a model/platform works based on parsing, loading or mocks.
Run relevant local checks before any hosted workflow.

```bash
cargo fmt --check
cargo test --locked --lib -- --test-threads=1
cargo clippy --locked --all-targets -- -D warnings
```

Generic tests are additional quality gates, not numerical certification.
See [release gates](RELEASE_GATES_CHECKLIST.md) and [methodology](docs/METHODOLOGY.md).

## Rights and publication

Shimmy is Apache-2.0; see [LICENSE](LICENSE). Preserve third-party attribution,
component licenses and patent notices. Airframe's runtime is MIT; libfse has
separate noncommercial/evaluation and commercial-permission terms.
Approved contributors sign off each commit with git commit --signoff;
see [DCO](DCO.md). Sign-off does not imply co-maintainer approval or waive rights.

Public source integration, tags, publication, hosted releases and images need
explicit maintainer approval of the exact result. A private push is not public
release authority.
