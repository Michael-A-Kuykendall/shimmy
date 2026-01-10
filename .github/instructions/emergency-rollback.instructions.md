---
applyTo: "**"
---
# Emergency Rollback Instructions

## Overview
Instructions for rolling back Shimmy Vision to test mode in case of production issues.

## Trigger Conditions
- Critical API failures
- Payment processing errors
- Data corruption
- Security breaches

## Rollback Steps
1. **Frontend**: Switch back to test endpoints
   - Update code to use `shimmy-license-webhook-test.michaelallenkuykendall.workers.dev`
   - Build and deploy to GitHub Pages

2. **Stripe**: Toggle back to test mode
   - Dashboard: Switch to "Test mode"
   - Verify test products active

3. **Worker**: Ensure test secrets active
   - No changes needed if test env separate

4. **Communication**: Notify users of temporary issues

## Verification
- Site loads with test endpoints
- Test purchases work
- No live data affected

## Recovery
- Fix root cause
- Test fixes in test environment
- Gradually re-enable live features

## Previous Usage
- No rollbacks performed
- Process not tested
- Manual execution required