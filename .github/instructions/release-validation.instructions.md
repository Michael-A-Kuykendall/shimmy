---
applyTo: "**"
---
# Release Validation (No Playwright)

Goal: prove test-mode backend works, and GitHub Pages points at the same Worker.

## Inputs
- Environment: `test` or `live`
- No secrets printed. Only pass/fail and masked IDs.

## One-command flow (test)
```bash
py scripts/release_validate.py backend-smoke --env test
py scripts/release_validate.py frontend-verify --env test

# If the default Pages URL 404s, override it explicitly:
py scripts/release_validate.py frontend-verify --env test --pages-url https://<your-user>.github.io/<your-repo>/
```

## After manual checkout payment (test)
1) Run `backend-smoke` and open the Stripe checkout URL (browser) to complete payment.
2) Take the `cs_test_...` session id (don’t paste full keys in chat).
3) Poll success page until license appears:
```bash
py scripts/release_validate.py success-poll --env test --session-id cs_test_...
```

## Invariants ("glue")
- Backend `/health` returns 200.
- Backend `/buy` is **GET** and returns 302/303 to `checkout.stripe.com`.
- GitHub Pages HTML/JS references **exactly one** worker host.
- That host matches the expected env:
  - test: `shimmy-license-webhook-test.michaelallenkuykendall.workers.dev`
  - live: `shimmy-license-webhook.michaelallenkuykendall.workers.dev`
