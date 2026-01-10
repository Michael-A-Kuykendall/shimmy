---
applyTo: "**"
---
# 🔄 Full Lifecycle Testing Methodology

## Overview
This document outlines the methodology for testing the complete Shimmy Vision customer lifecycle using API calls in Stripe test mode. This approach verifies all components work together without requiring manual UI interactions or real credit cards.

## Prerequisites
- Test Stripe secret key (`sk_test_...`)
- Test Stripe publishable key (`pk_test_...`)
- Keygen admin token and account ID
- Cloudflare Worker test endpoint
- Vision server running locally (optional for vision testing)

## Test Data Setup
Use deterministic test data to ensure reproducible results:

```javascript
const TEST_EMAIL = `test-${Date.now()}@example.com`;
const TEST_TIER = 'developer'; // or 'professional', 'startup', 'enterprise', 'lifetime'
```

## Phase 1: Purchase Flow Testing

### 1.1 Create Checkout Session
```bash
# Create Stripe checkout session via Cloudflare Worker
curl -X POST https://shimmy-license-webhook-test.michaelallenkuykendall.workers.dev/buy \
  -d "tier=$TEST_TIER&email=$TEST_EMAIL" \
  -H "Content-Type: application/x-www-form-urlencoded"
```

**Expected Response:**
- HTTP 200 with checkout session URL
- Session contains correct price, product metadata
- Success/cancel URLs point to correct endpoints

### 1.2 Simulate Payment Completion
```bash
# Get session details (requires session ID from previous response)
curl -u "$STRIPE_TEST_SECRET:" \
  "https://api.stripe.com/v1/checkout/sessions/$SESSION_ID"
```

**Verify:**
- `payment_status: paid`
- Customer created with correct email
- Subscription active (if applicable)

## Phase 2: Webhook Processing

### 2.1 Verify Webhook Delivery
```bash
# Check recent webhooks in Stripe dashboard or via API
curl -u "$STRIPE_TEST_SECRET:" \
  "https://api.stripe.com/v1/events?type=checkout.session.completed&limit=5"
```

**Verify:**
- Webhook sent to Cloudflare Worker
- Worker processed event successfully
- No errors in worker logs

### 2.2 Check License Creation
```bash
# Query Keygen for new license
curl -H "Authorization: Bearer $KEYGEN_ADMIN_TOKEN" \
     -H "Accept: application/vnd.api+json" \
     "https://api.keygen.sh/v1/accounts/$KEYGEN_ACCOUNT_ID/licenses?limit=5&sort=-created"
```

**Verify:**
- New license created with correct policy
- License associated with customer email
- License key generated and valid

## Phase 3: Portal Access Testing

### 3.1 Test Portal Session Creation
```bash
# Create customer portal session
curl -X POST https://shimmy-license-webhook-test.michaelallenkuykendall.workers.dev/portal \
  -d "email=$TEST_EMAIL" \
  -H "Content-Type: application/x-www-form-urlencoded"
```

**Expected Response:**
- HTTP 200 with portal URL
- Portal allows subscription management
- License information displayed

### 3.2 Verify License Retrieval
```bash
# Test license API directly
curl -X POST https://shimmy-license-webhook-test.michaelallenkuykendall.workers.dev/license \
  -d "email=$TEST_EMAIL" \
  -H "Content-Type: application/x-www-form-urlencoded"
```

**Verify:**
- Returns valid license key
- License matches Keygen records
- No authentication bypass possible

## Phase 4: Vision Feature Testing (Optional)

### 4.1 Start Local Vision Server
```bash
cargo run --features llama,vision -- serve --bind 127.0.0.1:11435
```

### 4.2 Test Vision API with License
```bash
curl -X POST http://127.0.0.1:11435/api/generate \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $LICENSE_KEY" \
  -d '{
    "model": "vision-model",
    "prompt": "Describe this image",
    "image": "data:image/jpeg;base64,'$(base64 -w 0 test-image.jpg)'",
    "stream": false,
    "max_tokens": 200
  }'
```

**Verify:**
- License authentication works
- Vision processing succeeds
- Proper error for invalid/expired licenses

## Phase 6: Frontend Integration Verification

### 6.1 Verify GitHub Pages Deployment
```bash
# Check site loads
curl -s -I https://michaelallenkuykendall.github.io/shimmy-vision/ | head -5

# Verify API endpoint configuration
curl -s https://michaelallenkuykendall.github.io/shimmy-vision/ | grep -o 'shimmy-license-webhook[^"]*' | head -1
```

**Expected:** Site loads (200 OK), uses correct worker URL (live or test)

### 6.2 API Call Consistency Check
```bash
# Extract API calls from frontend (requires puppeteer or manual inspection)
# Compare with tested endpoints:
# - /buy endpoint matches Phase 1
# - /portal endpoint matches Phase 3
# - /license endpoint matches Phase 3
```

**Verify:** Frontend uses identical API endpoints and parameters as tested calls

### 6.3 Payment Method Verification
```bash
# Check lifetime product payment methods
curl -u "$STRIPE_TEST_SECRET:" \
  "https://api.stripe.com/v1/products/$LIFETIME_PRODUCT_ID" | jq '.metadata'

# Verify Alipay/WeChat enabled for lifetime tier
curl -u "$STRIPE_TEST_SECRET:" \
  "https://api.stripe.com/v1/prices?product=$LIFETIME_PRODUCT_ID" | jq '.data[0].payment_method_types'
```

**Expected:** Lifetime tier includes `alipay`, `wechat_pay` in payment methods

### 6.4 Endpoint Accessibility Test
```bash
# Test all frontend-accessible endpoints
curl -X POST https://shimmy-license-webhook-test.michaelallenkuykendall.workers.dev/buy \
  -d "tier=lifetime&email=test@example.com" \
  -w "%{http_code}\n"

curl -X POST https://shimmy-license-webhook-test.michaelallenkuykendall.workers.dev/portal \
  -d "email=test@example.com" \
  -w "%{http_code}\n"
```

**Expected:** All endpoints return 200 or appropriate success codes

## Success Criteria

✅ **All API calls return expected HTTP status codes**  
✅ **Stripe checkout sessions create successfully**  
✅ **Webhooks process without errors**  
✅ **Keygen licenses generate correctly**  
✅ **Portal access works for valid customers**  
✅ **Vision API respects license validation**  
✅ **Error cases handled gracefully**  
✅ **Frontend uses identical API endpoints as tested**  
✅ **GitHub Pages site loads and connects properly**  
✅ **Lifetime tier includes Alipay/WeChat Pay options**  
✅ **No sensitive data exposed in responses**  

## Automation Script

Create `test-full-lifecycle.js`:

```javascript
const { execSync } = require('child_process');
const fs = require('fs');

// Load environment
require('dotenv').config();

// Test configuration
const TEST_EMAIL = `test-${Date.now()}@example.com`;
const WORKER_URL = 'https://shimmy-license-webhook-test.michaelallenkuykendall.workers.dev';

// Helper function to run curl commands
function curl(command) {
  try {
    return execSync(command, { encoding: 'utf8' });
  } catch (error) {
    console.error(`Command failed: ${command}`);
    console.error(error.stdout || error.stderr);
    throw error;
  }
}

// Phase 1: Purchase
console.log('🛒 Testing purchase flow...');
const checkoutResponse = curl(`curl -X POST ${WORKER_URL}/buy -d "tier=developer&email=${TEST_EMAIL}"`);
console.log('Checkout response:', checkoutResponse);

// Extract session ID and verify
// ... continue with verification steps

console.log('✅ Full lifecycle test completed');
```

## Integration with CI/CD

- Run this test suite before deployments
- Use different test emails per run to avoid conflicts
- Clean up test data after successful runs
- Alert on failures with detailed logs

## Troubleshooting

**Webhook not firing:** Check Cloudflare Worker logs and Stripe webhook configuration  
**License not created:** Verify Keygen API credentials and policy IDs  
**Portal access fails:** Check Stripe customer portal configuration  
**Vision API rejects license:** Verify license format and Keygen validation logic  

This methodology ensures complete end-to-end verification using only API calls, making it suitable for automated testing and CI/CD pipelines.