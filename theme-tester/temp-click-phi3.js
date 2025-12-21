import { chromium } from 'playwright';

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  await page.goto('http://localhost:8080');
  await page.waitForTimeout(5000);

  // Try a different approach - look for the phi-3 model container first
  const phi3Container = page.locator('div').filter({ hasText: 'phi-3-mini-4k-instruct-q4' }).filter({ hasText: 'Select Model' });
  const containerCount = await phi3Container.count();
  console.log('Found phi-3 containers:', containerCount);

  if (containerCount > 0) {
    // Get the first phi-3 container and click its button (disable strict mode)
    const selectButton = phi3Container.first().locator('button:has-text("Select Model")').first();
    console.log('Clicking phi-3 select button...');
    await selectButton.click();
    console.log('Clicked successfully!');

    await page.waitForTimeout(3000);
    await page.screenshot({ path: 'phi3-chat-started.png', fullPage: true });
    console.log('Screenshot saved');
  } else {
    console.log('No phi-3 container found');
  }

  await browser.close();
})();