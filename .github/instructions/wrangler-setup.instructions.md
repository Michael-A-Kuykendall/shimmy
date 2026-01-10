---
applyTo: "**"
---
# Wrangler Setup Instructions for Shimmy License Webhook

## Overview
Instructions for deploying and configuring the Cloudflare Worker for Shimmy Vision license fulfillment.

## Authentication
Wrangler uses API tokens set as Windows environment variables:
- `CLOUDFLARE_API_TOKEN`: General Cloudflare API access
- `CLOUDFLARE_WORKERS_TOKEN`: Workers-specific operations with proper permissions

**CRITICAL**: For ALL Wrangler operations, use `CLOUDFLARE_WORKERS_TOKEN`:
```bash
export CLOUDFLARE_API_TOKEN=$CLOUDFLARE_WORKERS_TOKEN
wrangler whoami  # Should show Account API Token
```

**DO NOT use CLOUDFLARE_API_TOKEN directly for Wrangler** - it may be a User token without Workers permissions.

## Current Test Environment Status
All required secrets are set:
- KEYGEN_ACCOUNT_ID
- KEYGEN_PRODUCT_TOKEN  
- STRIPE_PRICE_DEVELOPER
- STRIPE_PRICE_PROFESSIONAL
- STRIPE_PRICE_STARTUP
- STRIPE_PRICE_ENTERPRISE
- STRIPE_PRICE_LIFETIME
- STRIPE_SECRET_KEY
- STRIPE_WEBHOOK_SECRET

Test environment is ready for deployment.

## Environment Setup
The worker has two environments:
- Production: `shimmy-license-webhook.michaelallenkuykendall.workers.dev`
- Test: `shimmy-license-webhook-test.michaelallenkuykendall.workers.dev`

## Deploying
```bash
cd cloudflare-worker
wrangler deploy --env test  # For test environment
wrangler deploy            # For production
```

## Setting Secrets
All secrets must be set via wrangler secret put. They are environment-specific.

### Required Secrets for Both Environments
```bash
# Keygen Account ID
wrangler secret put KEYGEN_ACCOUNT_ID --env test
# Enter: 6270bf9c-23ad-4483-9296-3a6d9178514a

# Keygen Product Token
wrangler secret put KEYGEN_PRODUCT_TOKEN --env test
# Enter: prod-... (from Keygen dashboard / your secure env; NEVER paste into chat)

# Stripe Webhook Secret (from Stripe dashboard)
wrangler secret put STRIPE_WEBHOOK_SECRET --env test
# Enter: whsec_test_... (get from Stripe test webhooks)

# Stripe Secret Key
wrangler secret put STRIPE_SECRET_KEY --env test
# Enter: sk_test_... (from .env or Windows env)

# Stripe Price IDs (map to tiers)
wrangler secret put STRIPE_PRICE_DEVELOPER --env test
# Enter: price_1SmK831g5xy1QMw5hCY8u2I4

wrangler secret put STRIPE_PRICE_PROFESSIONAL --env test
# Enter: price_1SmK861g5xy1QMw5z9EmJfK5

wrangler secret put STRIPE_PRICE_STARTUP --env test
# Enter: price_1SmK881g5xy1QMw5BPyc895U

wrangler secret put STRIPE_PRICE_ENTERPRISE --env test
# Enter: price_1SmK9k1g5xy1QMw5a7PnDwDw

wrangler secret put STRIPE_PRICE_LIFETIME --env test
# Enter: price_1SmK9o1g5xy1QMw5cHBG18Xr
```

### Environment-Specific Variables
In wrangler.toml:
- Test env has `BUY_ENDPOINT_ENABLED = "1"` and `STRIPE_REQUIRE_CUSTOMER = "1"`
- Production env does not have these (for security)

## Verification
```bash
# Check health
curl https://shimmy-license-webhook-test.michaelallenkuykendall.workers.dev/health

# Test buy endpoint (test env only)
curl "https://shimmy-license-webhook-test.michaelallenkuykendall.workers.dev/buy?tier=developer&email=test@example.com"
```

## Logs
```bash
wrangler tail --env test
```

## Previous Usage
- Secrets were set manually in previous sessions
- Test environment deployed and functional
- Production environment ready for live secrets