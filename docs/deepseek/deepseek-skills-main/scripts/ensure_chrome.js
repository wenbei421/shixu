/**
 * 确保 Chrome 已启动并处于登录状态
 * 使用已存在的 Chrome 实例（不重复启动）
 */
const { chromium } = require('playwright');
const { exec, spawn } = require('child_process');

const CDP_URL = process.env.DEEPSEEK_CDP_URL || 'http://127.0.0.1:9222';

async function checkChromeConnection() {
  try {
    const browser = await chromium.connectOverCDP(CDP_URL);
    const ctx = browser.contexts()[0];
    const pages = ctx.pages();
    const page = pages[0];
    
    // 检查是否已登录 DeepSeek
    await page.goto('https://chat.deepseek.com/', { waitUntil: 'networkidle', timeout: 15000 });
    const url = page.url();
    await browser.close();
    
    return !url.includes('sign_in') && !url.includes('/login');
  } catch (e) {
    return false;
  }
}

async function startChrome() {
  const isWin = process.platform === 'win32';
  
  // Windows 启动命令
  const cmd = `Start-Process -FilePath "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe" -ArgumentList "--remote-debugging-port=9222","--user-data-dir=C:\\Users\\admin\\AppData\\Local\\Google\\Chrome\\User Data","--profile-directory=Default" -PassThru -WindowStyle Minimized`;
  
  return new Promise((resolve, reject) => {
    exec(cmd, { shell: 'powershell.exe' }, (err) => {
      if (err) {
        reject(err);
        return;
      }
      // 等待 Chrome 启动
      setTimeout(() => resolve(true), 4000);
    });
  });
}

async function main() {
  console.log('🔍 检查 Chrome 状态...');
  
  let isConnected = false;
  let isLoggedIn = false;
  
  try {
    isConnected = await checkChromeConnection();
    if (isConnected) {
      // 进一步检查登录状态
      const browser = await chromium.connectOverCDP(CDP_URL);
      const ctx = browser.contexts()[0];
      const page = ctx.pages()[0];
      const url = page.url();
      await browser.close();
      
      isLoggedIn = !url.includes('sign_in') && !url.includes('/login');
    }
  } catch (e) {
    console.log('⚠️ Chrome 未就绪');
  }
  
  if (!isConnected) {
    console.log('🚀 启动 Chrome...');
    await startChrome();
    console.log('✅ Chrome 已启动');
    console.log('');
    console.log('⚠️ 请在打开的 Chrome 中手动登录 DeepSeek');
    console.log('💡 登录成功后，后续调用将自动复用会话，无需再次登录');
  } else if (!isLoggedIn) {
    console.log('⚠️ Chrome 已启动但 DeepSeek 未登录');
    console.log('请在 Chrome 中打开 https://chat.deepseek.com 并登录');
  } else {
    console.log('✅ Chrome 已就绪，DeepSeek 已登录');
  }
}

main().catch(console.error);
