#!/usr/bin/env node
/**
 * Diagnostic: Capture browser console and check discovery status
 */

import { chromium } from 'playwright';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function main() {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  
  // Capture console messages
  const consoleLogs = [];
  page.on('console', msg => {
    consoleLogs.push({
      type: msg.type(),
      text: msg.text(),
      args: msg.args().length
    });
  });

  // Capture errors
  const errors = [];
  page.on('pageerror', err => {
    errors.push(err.message);
  });

  try {
    console.log('📍 Navigating to http://localhost:8080...');
    await page.goto('http://localhost:8080', { waitUntil: 'networkidle', timeout: 15000 });
    
    console.log('⏳ Waiting 5 seconds for discovery to attempt...');
    await page.waitForTimeout(5000);
    
    // Take screenshot
    const screenshotPath = path.join(__dirname, 'screenshots', 'console-diagnostic.png');
    await page.screenshot({ path: screenshotPath, fullPage: true });
    console.log(`✅ Screenshot: ${screenshotPath}`);
    
    // Check page title and content
    const title = await page.title();
    console.log(`📄 Page title: ${title}`);
    
    // Look for discovery-related elements
    const discoveryText = await page.textContent('body');
    if (discoveryText.includes('30%')) {
      console.log('🔴 Found "30%" in page - stuck at discovery');
    }
    if (discoveryText.includes('OFFLINE')) {
      console.log('🔴 Found "OFFLINE" - not connected');
    }
    if (discoveryText.includes('Discovery')) {
      console.log('🟡 Found "Discovery" text in page');
    }
    
    // Print console logs
    console.log('\n📋 CONSOLE LOGS:');
    if (consoleLogs.length === 0) {
      console.log('  (no console output)');
    } else {
      consoleLogs.forEach((log, i) => {
        console.log(`  [${i}] ${log.type.toUpperCase()}: ${log.text.substring(0, 100)}`);
      });
    }
    
    // Print errors
    if (errors.length > 0) {
      console.log('\n🔴 ERRORS:');
      errors.forEach((err, i) => {
        console.log(`  [${i}] ${err}`);
      });
    } else {
      console.log('\n✅ No JavaScript errors');
    }
    
    // Try to inspect discovery client state
    console.log('\n🔍 Checking discovery-client state...');
    const state = await page.evaluate(() => {
      // Try to access React state or global vars
      return {
        href: window.location.href,
        hasDiscoveryClient: typeof window.__SHIMMY_DISCOVERY !== 'undefined'
      };
    });
    console.log(`  URL: ${state.href}`);
    console.log(`  Discovery client global: ${state.hasDiscoveryClient}`);
    
  } catch (error) {
    console.error(`❌ Error: ${error.message}`);
  } finally {
    await browser.close();
  }
}

main();
