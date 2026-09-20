const { chromium } = require('playwright');

(async () => {
  console.log('1. Connecting...');
  const browser = await chromium.connectOverCDP('http://127.0.0.1:9222');
  const ctx = browser.contexts()[0];
  const page = ctx.pages()[0];

  console.log('2. URL:', page.url());

  // 找输入框
  const input = page.locator('textarea[placeholder]').first();
  const question = '用一句话介绍AI Agent';
  await input.fill(question);
  console.log('3. Filled:', question);

  await page.keyboard.press('Enter');
  console.log('4. Sent, waiting for reply...');

  // 轮询等待回复完成
  for (let i = 0; i < 20; i++) {
    await page.waitForTimeout(2000);
    const isDone = await page.evaluate(() => {
      const btns = document.querySelectorAll('button');
      for (const btn of btns) {
        if (btn.disabled && btn.offsetParent !== null) return true;
      }
      return false;
    });
    if (isDone) {
      console.log('5. Reply done at', (i + 1) * 2, 's');
      break;
    }
  }

  // 提取回复
  const reply = await page.evaluate(() => {
    const selectors = ['[class*="markdown"]', '[class*="message-content"]', '.ds-markdown'];
    for (const sel of selectors) {
      const els = document.querySelectorAll(sel);
      if (els.length > 0) {
        const text = els[els.length - 1].innerText;
        if (text && text.length > 10) return text;
      }
    }
    return '';
  });

  console.log('\n=== Reply ===');
  console.log(reply);
  console.log('=============');

  // 输出 JSON
  console.log('\n=== JSON ===');
  console.log(JSON.stringify({
    success: true,
    question,
    answer: reply,
    timestamp: new Date().toISOString()
  }, null, 2));

  await browser.close();
  console.log('Done');
})().catch(e => console.error('ERR:', e.message));
