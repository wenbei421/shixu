(() => {
  const taskId = __SHIXU_TASK_ID_JSON__;
  const prompt = __SHIXU_PROMPT_JSON__;

  if (typeof window.__SHIXU_DEEPSEEK_CLEANUP__ === 'function') {
    try {
      window.__SHIXU_DEEPSEEK_CLEANUP__();
    } catch (error) {
      console.warn('[shixu] cleanup previous DeepSeek task failed', error);
    }
  }

  let observer = null;
  let timer = null;
  let boot = null;
  let completed = false;
  let submitted = false;
  let baseline = '';
  let previousText = '';
  let stableCount = 0;

  function publish(type, extra) {
    const prev = window.__SHIXU_DEEPSEEK__ || {};
    const next = extra || {};
    window.__SHIXU_DEEPSEEK__ = {
      taskId,
      type,
      text: next.text ?? prev.text ?? '',
      result: Object.prototype.hasOwnProperty.call(next, 'result') ? next.result : null,
      generating: Boolean(next.generating),
      stableCount: next.stableCount ?? 0,
      error: next.error ?? null,
      message: next.message ?? null,
    };
  }

  function cleanup() {
    if (observer) {
      observer.disconnect();
      observer = null;
    }
    if (timer) {
      clearInterval(timer);
      timer = null;
    }
  }

  function stop() {
    completed = true;
    cleanup();
    if (boot) {
      clearInterval(boot);
      boot = null;
    }
    window.__SHIXU_DEEPSEEK_CLEANUP__ = null;
  }

  window.__SHIXU_DEEPSEEK_CLEANUP__ = stop;

  function isVisible(element) {
    if (!element) return false;
    const rect = element.getBoundingClientRect();
    if (rect.width < 40 || rect.height < 8) return false;
    const style = window.getComputedStyle(element);
    return style.display !== 'none' && style.visibility !== 'hidden' && style.opacity !== '0';
  }

  function findInput() {
    const selectors = [
      'textarea[placeholder]',
      '#chat-input',
      'textarea',
      '[contenteditable="true"]',
    ];
    for (const selector of selectors) {
      const elements = document.querySelectorAll(selector);
      for (const element of elements) {
        if (element.disabled || element.readOnly) continue;
        const rect = element.getBoundingClientRect();
        if (rect.width < 160 || rect.height < 20) continue;
        if (isVisible(element)) return element;
      }
    }
    return null;
  }

  function findSendButton() {
    const selectors = [
      'button[aria-label*="send" i]',
      'button[aria-label*="Send"]',
      'button[aria-label*="发送"]',
      'button[title*="send" i]',
      'button[title*="Send"]',
      'button[title*="发送"]',
      'div[role="button"][aria-label*="发送"]',
      'div[role="button"][aria-label*="Send"]',
    ];
    for (const selector of selectors) {
      const elements = document.querySelectorAll(selector);
      for (const element of elements) {
        if (isVisible(element) && !element.disabled && element.getAttribute('aria-disabled') !== 'true') {
          return element;
        }
      }
    }
    return null;
  }

  function setInputValue(input, value) {
    input.focus();
    if (input instanceof HTMLTextAreaElement || input instanceof HTMLInputElement) {
      const prototype = input instanceof HTMLTextAreaElement
        ? HTMLTextAreaElement.prototype
        : HTMLInputElement.prototype;
      const setter = Object.getOwnPropertyDescriptor(prototype, 'value')?.set;
      if (setter) setter.call(input, value);
      else input.value = value;
      if (input._valueTracker) input._valueTracker.setValue('');
    } else {
      input.textContent = value;
    }
    input.dispatchEvent(new InputEvent('input', {
      bubbles: true,
      inputType: 'insertText',
      data: value,
    }));
    input.dispatchEvent(new Event('change', { bubbles: true }));
  }

  function hasStopButton() {
    const selectors = [
      'button[aria-label*="stop" i]',
      'button[aria-label*="Stop"]',
      'button[aria-label*="停止"]',
      'div[role="button"][aria-label*="停止"]',
      'div[role="button"][aria-label*="Stop"]',
    ];
    for (const selector of selectors) {
      const elements = document.querySelectorAll(selector);
      for (const element of elements) {
        if (isVisible(element)) return true;
      }
    }
    return document.querySelectorAll('svg animate, svg animateTransform').length > 0;
  }

  function readText(element) {
    return (element.innerText || element.textContent || '').replace(/\u00a0/g, ' ').trim();
  }

  function getLastReplyText() {
    // 段落节点也带 markdown 类。从后往前取会只拿到最后一段。
    const assistants = document.querySelectorAll('.ds-assistant-message-main-content');
    if (assistants.length > 0) {
      return readText(assistants[assistants.length - 1]);
    }

    const markdowns = document.querySelectorAll('.ds-markdown');
    for (let index = markdowns.length - 1; index >= 0; index -= 1) {
      const element = markdowns[index];
      if (element.closest('.ds-markdown') !== element) continue;
      const text = readText(element);
      if (text) return text;
    }
    return '';
  }

  function looksLikeLogin() {
    const href = String(location.href || '');
    if (/sign_in|login|passport/i.test(href)) return true;
    const text = (document.body?.innerText || '').slice(0, 1500);
    return /手机号|验证码|密码登录|Sign in|Log in|登录/.test(text);
  }

  function fail(error, message) {
    publish('error', { error, message, generating: false });
    stop();
  }

  function submitPrompt(input) {
    setInputValue(input, prompt);
    const button = findSendButton();
    if (button) {
      button.click();
      return;
    }
    input.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'Enter',
      code: 'Enter',
      keyCode: 13,
      which: 13,
      bubbles: true,
      cancelable: true,
    }));
  }

  function checkReply() {
    if (completed || !submitted) return;
    const text = getLastReplyText();
    const generating = hasStopButton();
    const fresh = text && text !== baseline ? text : '';
    if (fresh && fresh === previousText) stableCount += 1;
    else if (fresh) {
      stableCount = 0;
      previousText = fresh;
    } else {
      stableCount = 0;
    }
    const visible = fresh || previousText;
    publish('progress', {
      text: visible,
      generating,
      stableCount,
      message: generating ? '正在生成' : '正在等待回复稳定',
    });
    if (fresh && !generating && stableCount >= 3) {
      publish('completed', {
        text: fresh,
        result: fresh,
        generating: false,
        stableCount,
        message: '已完成',
      });
      stop();
    }
  }

  try {
    publish('waiting', { message: '正在等待 DeepSeek 页面' });
    const started = Date.now();
    boot = setInterval(() => {
      if (completed) return;
      if (!submitted && Date.now() - started > 25000) {
        if (looksLikeLogin()) {
          fail('LOGIN_REQUIRED', '请先在 DeepSeek 窗口登录');
        } else {
          fail('INPUT_NOT_FOUND', '未找到 DeepSeek 输入框');
        }
        return;
      }
      if (submitted) return;
      const input = findInput();
      if (!input) return;
      baseline = getLastReplyText();
      previousText = '';
      stableCount = 0;
      submitPrompt(input);
      submitted = true;
      publish('submitted', { message: '已发送到 DeepSeek', generating: true });
      observer = new MutationObserver(() => checkReply());
      observer.observe(document.documentElement, {
        subtree: true,
        childList: true,
        characterData: true,
      });
      timer = setInterval(checkReply, 1000);
    }, 500);
  } catch (error) {
    fail('SCRIPT_ERROR', String(error && error.message ? error.message : error));
  }
})();
