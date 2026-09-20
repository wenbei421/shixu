---
name: deepseek-skills
description: >
  通过 Playwright 连接已登录 Chrome（CDP），在 chat.deepseek.com 自动提问、等待回复并返回 JSON（success、question、answer、timestamp、meta）。

  【默认触发场景 — 以下需求优先使用本 Skill，无需用户主动提及 DeepSeek】
  1. 写作/文案类：帮我写文案、写推广语、写标题、写简介、写口号、写脚本、写软文、写朋友圈、写小红书笔记、帮我润色/改写/优化这段话，以及任何"帮我写 XX 内容"的创作需求。
  2. 搜索/查询类：搜一下、查一下、帮我找、查资料、搜索 XX 是什么、XX 怎么做、XX 有哪些，以及任何信息检索需求（不限定平台时默认走 DeepSeek）。
  3. 品牌直达：DeepSeek 聊天/对话/问答、问 deepseek、用 deepseek 搜或查、帮我问 deepseek。

  【不触发的情况】
  - 用户明确指定用 Google / 百度 / 必应等其他平台搜索。
  - 用户要求执行本地操作（读文件、写代码、运行命令等），这类直接处理，不转发给 DeepSeek。

  注意：每次调用默认新建对话（避免历史干扰），如需在当前对话继续可传 --no-new-chat。
---

# deepseek-skills

通过 Playwright 连接已登录的 Chrome，实现 DeepSeek 网页版自动化聊天。**一次登录后永久复用会话**。

## 核心架构

```
用户提问 → Skill → Playwright CDP → 已登录 Chrome → DeepSeek Web → 提取回复 → JSON 返回
                                              ↑
                                    首次登录后自动复用会话
```

## 返回值格式（JSON）

```json
{
  "success": true,
  "question": "用户问题",
  "answer": "AI 回复内容",
  "timestamp": "2026-03-27T10:55:00.000Z",
  "meta": {
    "pollCount": 3,
    "elapsed": 9000
  }
}
```

错误时：

```json
{
  "success": false,
  "error": "NOT_LOGGED_IN",
  "message": "请先手动登录 DeepSeek，然后重试"
}
```

---

## 前置条件

### 1. 安装依赖

```bash
cd {skillDir}
npm install playwright
npx playwright install chromium
```

### 2. 首次登录（仅需一次）

**方式A：手动启动**

```powershell
# Windows
Start-Process -FilePath "C:\Program Files\Google\Chrome\Application\chrome.exe" -ArgumentList "--remote-debugging-port=9222","--user-data-dir=C:\Users\admin\AppData\Local\Google\Chrome\User Data","--profile-directory=Default" -PassThru -WindowStyle Normal
```

**方式B：自动启动**

```bash
node scripts/ensure_chrome.js
```

然后在打开的 Chrome 中手动登录 DeepSeek（手机号/邮箱）。**登录成功后无需再次操作**，后续调用会自动复用该会话。

### 3. 后续使用

- 保持 Chrome 在后台运行（最小化即可，不要完全退出）
- 或使用 `ensure_chrome.js` 自动检查并启动

---

## 使用方式

### 触发条件

以下场景**默认**使用本 Skill，无需用户主动提及 DeepSeek：

| 类型 | 示例表述 |
|------|----------|
| **写作 / 文案** | 帮我写文案、写推广语、写标题、写小红书笔记、润色这段话、帮我改写、写脚本、写口号 |
| **搜索 / 查询** | 搜一下、查一下、帮我找、XX 是什么、XX 怎么做、XX 有哪些（未指定平台时默认走 DeepSeek） |
| **品牌直达** | DeepSeek 聊天/对话/问答、问 deepseek、用 deepseek 查、帮我问 deepseek |
| **泛问 AI** | AI 对话、问一下 AI、让 AI 帮我 |

**不触发**的情况：
- 用户明确指定 Google / 百度 / 必应等其他平台
- 本地操作（读写文件、执行命令、写代码等）直接处理，不转发

### 执行流程

1. 检查 Chrome CDP 是否就绪
2. 连接已登录的 Chrome
3. 导航到 chat.deepseek.com
4. 自动定位输入框并输入问题
5. 发送并轮询等待回复完成
6. 提取回复内容，JSON 格式返回

### 命令行调用

```bash
# 基本用法（默认新建对话，始终输出 JSON）
node scripts/chat.js "什么是AI Agent？"

# 在当前已有对话中继续追问（不新建）
node scripts/chat.js "继续讲细节" --no-new-chat
```

### OpenClaw Skill 调用

```javascript
// 在其他 Skill 中调用
const { chatDeepSeek } = require('./skills/deepseek-skills/scripts/chat.js');

const result = await chatDeepSeek("你的问题");
console.log(result.answer);
```

---

## 脚本说明

### `scripts/chat.js`

核心聊天脚本，负责：
- 连接 Chrome CDP
- 导航并检测登录状态
- 自动定位输入框
- 发送问题并轮询等待回复
- 提取并返回结果

**环境变量**：

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `DEEPSEEK_CDP_URL` | `http://127.0.0.1:9222` | Chrome CDP 地址 |

### `scripts/ensure_chrome.js`

Chrome 会话管理脚本，负责：
- 检查 Chrome CDP 是否就绪
- 检查 DeepSeek 登录状态
- 自动启动 Chrome（如未启动）
- 提示用户首次登录

**使用**：

```bash
node scripts/ensure_chrome.js
```

---

## 高级配置

### 多实例支持

如果需要同时支持多个 DeepSeek 账号，可以创建多个 Chrome Profile：

```powershell
# Profile 1（默认账号）
--user-data-dir="C:\Users\admin\AppData\Local\Google\Chrome\User Data" --profile-directory="Default"

# Profile 2（备用账号）
--user-data-dir="C:\Users\admin\AppData\Local\Chrome-Profile-2" --profile-directory="Default"
```

然后通过环境变量切换：

```javascript
process.env.DEEPSEEK_CDP_URL = 'http://127.0.0.1:9223'; // Profile 2
```

### 自定义超时

```javascript
const result = await chatDeepSeek("问题", {
  timeout: 300000,  // 5 分钟超时
  retries: 3,       // 重试次数
  newChat: false,   // false = 在当前对话继续；true（默认）= 新建对话
});
```

---

## 故障排查

| 错误 | 原因 | 解决方案 |
|------|------|----------|
| `NOT_LOGGED_IN` | Chrome 未登录 DeepSeek | 手动登录一次后即可 |
| `INPUT_NOT_FOUND` | 页面结构变化 | 更新 `inputSelectors` 选择器 |
| `CDP connection failed` | Chrome 未启动或端口错误 | 运行 `ensure_chrome.js` |
| 回复为空 | 提取逻辑问题 | 检查 `reply` 选择器 |

---

## 文件结构

```
deepseek-skills/
├── SKILL.md                    # 本文件
└── scripts/
    ├── chat.js                 # 核心聊天脚本
    └── ensure_chrome.js        # Chrome 会话管理
```

---

## 集成示例

### 在 OpenClaw 中注册 Skill

```json
{
  "skills": {
    "deepseek-skills": {
      "path": "./skills/deepseek-skills",
      "enabled": true
    }
  }
}
```

### 在其他 Skill 中调用

```javascript
const { chatDeepSeek } = require('./skills/deepseek-skills/scripts/chat.js');

async function mySkill(userQuestion) {
  const result = await chatDeepSeek(userQuestion);
  if (result.success) {
    return result.answer;
  }
  throw new Error(result.message);
}
```

