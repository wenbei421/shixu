/**
 * DeepSeek 聊天自动化 - 主脚本（优化版）
 * 返回 JSON 格式结果
 *
 * 优化内容：
 * 1. 修复轮询完成检测：改用"停止生成"按钮消失 + 文本稳定双重判断
 * 2. 修复回复提取：精确定位最后一条 AI 消息，避免截断
 * 3. 新建对话：每次调用前新建会话，避免历史污染
 * 4. 自适应轮询：前期快轮询(500ms)，后期慢轮询(2000ms)
 * 5. 导航保障：如当前页面非 DeepSeek，自动导航
 */
const { chromium } = require('playwright');

const CDP_URL = process.env.DEEPSEEK_CDP_URL || 'http://127.0.0.1:9222';
const DEEPSEEK_URL = 'https://chat.deepseek.com/';

/**
 * 等待 DeepSeek 回复完成
 * 判断依据：
 *   1. "停止生成"按钮（带 stop/pause 图标的可见按钮）消失
 *   2. 连续两次文本内容相同（稳定）
 */
async function waitForReplyDone(page, timeout = 120000) {
  const start = Date.now();
  let lastText = null;
  let stableCount = 0;
  let interval = 500; // 初始快轮询

  while (Date.now() - start < timeout) {
    await page.waitForTimeout(interval);

    // 判断"停止生成"按钮是否还在（DeepSeek 生成中会显示此按钮）
    const isGenerating = await page.evaluate(() => {
      // DeepSeek 生成时会有一个带有停止图标的按钮可见
      // 通过 aria-label、title 或特定 class 检测
      const stopSelectors = [
        'button[aria-label*="stop"]',
        'button[aria-label*="停止"]',
        'button[title*="stop"]',
        'button[title*="停止"]',
        // DeepSeek 特有：发送按钮在生成时变为停止按钮，通常有 disabled 状态变化
        // 更可靠：检查是否存在 "regenerate" 或 "copy" 按钮（回复完成后出现）
      ];
      for (const sel of stopSelectors) {
        const el = document.querySelector(sel);
        if (el && el.offsetParent !== null) return true;
      }

      // 备用：检查发送按钮区域是否有 SVG 动画（生成中）
      const svgAnimations = document.querySelectorAll('svg animateTransform, svg animate');
      if (svgAnimations.length > 0) return true;

      return false;
    });

    // 获取当前最后一条 AI 消息文本
    const currentText = await extractLastReply(page);

    if (!isGenerating) {
      // 停止按钮消失了，但再确认一次文本稳定
      if (currentText && currentText === lastText) {
        stableCount++;
        if (stableCount >= 2) {
          return { done: true, reason: 'stable' };
        }
      } else {
        stableCount = 0;
        lastText = currentText;
      }
    } else {
      stableCount = 0;
      lastText = currentText;
      // 生成中时降低轮询频率节省资源
      interval = 1000;
    }
  }

  return { done: false, reason: 'timeout' };
}

/**
 * 提取最后一条 AI 回复内容
 * 精确匹配 DeepSeek 的消息气泡结构
 */
async function extractLastReply(page) {
  return page.evaluate(() => {
    // DeepSeek 的 AI 消息通常有以下结构特征
    // 优先级从高到低尝试
    const strategies = [
      // 策略1：ds-markdown 类（DeepSeek 专属 markdown 渲染容器）
      () => {
        const els = document.querySelectorAll('.ds-markdown');
        if (els.length > 0) {
          return els[els.length - 1].innerText.trim();
        }
        return null;
      },
      // 策略2：class 含 markdown 的最后一个元素
      () => {
        const els = document.querySelectorAll('[class*="markdown"]');
        if (els.length > 0) {
          // 从后往前找第一个有实质内容的
          for (let i = els.length - 1; i >= 0; i--) {
            const text = els[i].innerText.trim();
            if (text.length > 10) return text;
          }
        }
        return null;
      },
      // 策略3：找 AI 角色消息容器（通常有 role="assistant" 或特定 class）
      () => {
        const roles = document.querySelectorAll('[data-role="assistant"], [class*="assistant"]');
        if (roles.length > 0) {
          const last = roles[roles.length - 1];
          const text = last.innerText.trim();
          if (text.length > 10) return text;
        }
        return null;
      },
      // 策略4：message-content 类
      () => {
        const els = document.querySelectorAll('[class*="message-content"]');
        if (els.length > 0) {
          for (let i = els.length - 1; i >= 0; i--) {
            const text = els[i].innerText.trim();
            if (text.length > 10) return text;
          }
        }
        return null;
      },
    ];

    for (const strategy of strategies) {
      const result = strategy();
      if (result) return result;
    }
    return '';
  });
}

/**
 * 新建一个 DeepSeek 对话（点击"新建对话"按钮）
 */
async function startNewChat(page) {
  try {
    // 尝试点击新建对话按钮
    const newChatSelectors = [
      'button[aria-label*="new"]',
      'button[aria-label*="新建"]',
      'a[href="/"]',
      '[class*="new-chat"]',
      '[class*="newChat"]',
      // DeepSeek 左侧边栏通常有新建按钮，带有铅笔/加号图标
      'button[class*="compose"]',
    ];

    for (const sel of newChatSelectors) {
      const btn = page.locator(sel).first();
      if (await btn.isVisible().catch(() => false)) {
        await btn.click();
        await page.waitForTimeout(800);
        return true;
      }
    }

    // 备用：直接导航到首页（会打开新对话）
    await page.goto(DEEPSEEK_URL, { waitUntil: 'domcontentloaded', timeout: 15000 });
    await page.waitForTimeout(1000);
    return true;
  } catch (e) {
    // 新建失败不影响主流程，继续用当前页面
    return false;
  }
}

/**
 * 执行 DeepSeek 对话
 * @param {string} question - 用户问题
 * @param {Object} options - 选项
 * @param {number} options.timeout - 等待超时(ms)，默认 120000
 * @param {number} options.retries - 失败重试次数，默认 2
 * @param {boolean} options.newChat - 是否新建对话，默认 true
 * @returns {Object} JSON 结果
 */
async function chatDeepSeek(question, options = {}) {
  const {
    timeout = 120000,
    retries = 2,
    newChat = true,
  } = options;

  let lastError = null;

  for (let attempt = 0; attempt <= retries; attempt++) {
    let browser = null;
    try {
      browser = await chromium.connectOverCDP(CDP_URL);
      const ctx = browser.contexts()[0];
      const page = ctx.pages()[0];

      // 1. 确保当前页面是 DeepSeek
      const currentUrl = page.url();
      if (!currentUrl.includes('deepseek.com')) {
        await page.goto(DEEPSEEK_URL, { waitUntil: 'domcontentloaded', timeout: 15000 });
        await page.waitForTimeout(1500);
      }

      // 2. 检查登录状态
      const finalUrl = page.url();
      if (finalUrl.includes('sign_in') || finalUrl.includes('/login')) {
        await browser.close();
        return {
          success: false,
          error: 'NOT_LOGGED_IN',
          message: '请先在 Chrome 中登录 DeepSeek',
        };
      }

      // 3. 新建对话（避免历史干扰）
      if (newChat) {
        await startNewChat(page);
      }

      // 4. 找输入框并等待可用
      const inputSelectors = [
        'textarea[placeholder]',
        '#chat-input',
        '[contenteditable="true"]',
      ];

      let input = null;
      for (const sel of inputSelectors) {
        const el = page.locator(sel).first();
        if (await el.isVisible().catch(() => false)) {
          input = el;
          break;
        }
      }

      if (!input) {
        await browser.close();
        return {
          success: false,
          error: 'INPUT_NOT_FOUND',
          message: '未找到输入框，页面可能尚未加载完成',
        };
      }

      // 5. 输入问题并发送
      await input.click();
      await input.fill(question);
      await page.waitForTimeout(200); // 等待输入稳定
      await page.keyboard.press('Enter');

      // 6. 等待回复完成
      const { done, reason } = await waitForReplyDone(page, timeout);

      // 7. 提取回复
      const reply = await extractLastReply(page);

      await browser.close();

      return {
        success: true,
        question,
        answer: reply,
        timestamp: new Date().toISOString(),
        meta: {
          replyDone: done,
          doneReason: reason,
          attempt: attempt + 1,
        },
      };

    } catch (e) {
      lastError = e;
      if (browser) {
        try { await browser.close(); } catch (_) {}
      }
      if (attempt < retries) {
        process.stderr.write(`Attempt ${attempt + 1} failed: ${e.message}, retrying...\n`);
        await new Promise(r => setTimeout(r, 2000));
      }
    }
  }

  return {
    success: false,
    error: 'MAX_RETRIES_EXCEEDED',
    message: lastError?.message || '未知错误',
  };
}

// CLI 入口 —— 始终输出 JSON，stdout 只有 JSON，日志走 stderr
if (require.main === module) {
  const args = process.argv.slice(2);
  const noNewChat = args.includes('--no-new-chat');
  const question = args.find(a => !a.startsWith('-')) || '';

  if (!question) {
    const usage = { success: false, error: 'MISSING_QUESTION', message: 'Usage: node chat.js "问题" [--no-new-chat]' };
    console.log(JSON.stringify(usage, null, 2));
    process.exit(1);
  }

  chatDeepSeek(question, { newChat: !noNewChat }).then(result => {
    console.log(JSON.stringify(result, null, 2));
    process.exit(result.success ? 0 : 1);
  }).catch(e => {
    const err = { success: false, error: 'EXCEPTION', message: e.message };
    console.log(JSON.stringify(err, null, 2));
    process.exit(1);
  });
}

module.exports = { chatDeepSeek };
