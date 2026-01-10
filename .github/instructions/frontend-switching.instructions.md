---
applyTo: "**"
---
# Frontend Switching Between Test and Live

## Overview
Instructions for switching the Shimmy Vision frontend between test and live environments.

## API Endpoint Configuration
The frontend code contains hardcoded API endpoints that need to be updated.

### Files to Update
- `shimmy-vision/src/config.js` or similar (locate the API base URL)
- Any environment files

### Test Endpoints
- Buy: `https://shimmy-license-webhook-test.michaelallenkuykendall.workers.dev/buy`
- Portal: `https://shimmy-license-webhook-test.michaelallenkuykendall.workers.dev/portal`
- License: `https://shimmy-license-webhook-test.michaelallenkuykendall.workers.dev/license`

### Live Endpoints
- Buy: `https://shimmy-license-webhook.michaelallenkuykendall.workers.dev/buy`
- Portal: `https://shimmy-license-webhook.michaelallenkuykendall.workers.dev/portal`
- License: `https://shimmy-license-webhook.michaelallenkuykendall.workers.dev/license`

## Process
1. Update code with live endpoints
2. Build and deploy to GitHub Pages
3. Test with live Stripe (use real payment methods)

## Verification
- Frontend loads live site
- API calls go to live worker
- Stripe checkout uses live mode

## Previous Usage
- Frontend currently uses test endpoints
- Manual code updates required for switching
- No automated switching process established