# SHIXU Logo 实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 用"拾序"字标替换 `src-tauri/icons/` 下全套默认 Tauri 图标，构建可正常打包。

**架构：** 单一 SVG 源 → `pnpm tauri icon` 自动派生 15 个 PNG + ICO + ICNS。视觉规格已在 [`docs/superpowers/specs/2026-09-18-shixu-logo-design.md`](../specs/2026-09-18-shixu-logo-design.md) 锁定（C3-2 标准居中）。

**技术栈：** Tauri v2 CLI、SVG、PNG/ICO/ICNS 多尺寸资源。

---

## 文件结构

**创建：**
- `src-tauri/icons/icon-source.svg` — 单一 SVG 源，C3-2 视觉规范的最终载体

**修改（由 Tauri CLI 自动覆盖）：**
- `src-tauri/icons/32x32.png`
- `src-tauri/icons/128x128.png`
- `src-tauri/icons/128x128@2x.png`
- `src-tauri/icons/icon.icns`
- `src-tauri/icons/icon.ico`
- `src-tauri/icons/Square30x30Logo.png`
- `src-tauri/icons/Square44x44Logo.png`
- `src-tauri/icons/Square71x71Logo.png`
- `src-tauri/icons/Square89x89Logo.png`
- `src-tauri/icons/Square107x107Logo.png`
- `src-tauri/icons/Square142x142Logo.png`
- `src-tauri/icons/Square150x150Logo.png`
- `src-tauri/icons/Square284x284Logo.png`
- `src-tauri/icons/Square310x310Logo.png`
- `src-tauri/icons/StoreLogo.png`

**未修改：**
- `src-tauri/tauri.conf.json` — `bundle.icon` 数组已包含目标文件，无需改

---

## 任务 1：创建 SVG 源文件

**文件：**
- 创建：`src-tauri/icons/icon-source.svg`

- [ ] **步骤 1：写入 SVG 内容**

在 `src-tauri/icons/icon-source.svg` 中写入以下内容（精确匹配设计文档 §3.1）：

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128">
  <rect width="128" height="128" rx="24" fill="#A78BFA"/>
  <text x="64" y="66"
        text-anchor="middle"
        dominant-baseline="central"
        font-family="Source Han Sans CN, Noto Sans CJK SC, Microsoft YaHei, system-ui, sans-serif"
        font-size="56"
        font-weight="900"
        fill="#FFFFFF"
        letter-spacing="-3">拾序</text>
</svg>
```

- [ ] **步骤 2：视觉验证**

运行：

```bash
# 在浏览器或 IDE 中打开文件
start src-tauri/icons/icon-source.svg   # Windows
# 或
open src-tauri/icons/icon-source.svg    # macOS
```

预期：在浏览器/SVG 预览中显示一个紫蓝圆角矩形，中央白色"拾序"二字，字重粗黑。

如果显示不一致（最常见原因：系统缺少中文字体导致回退到默认字体），运行：

```bash
# 检查可用字体
fc-list :lang=zh   # Linux
# 或在 PowerShell 检查
[System.Drawing.Text.InstalledFontCollection]::new().Families | Where-Object { $_.Name -match 'YaHei|Source|Noto' }
```

- [ ] **步骤 3：Commit**

```bash
git add src-tauri/icons/icon-source.svg
git commit -m "feat(icons): add shixu logo SVG source (C3-2 字标)"
```

---

## 任务 2：用 Tauri CLI 生成全套图标

**文件：**
- 修改：`src-tauri/icons/` 下 17 个 PNG/ICO/ICNS 文件（由 CLI 覆盖）

- [ ] **步骤 1：运行 tauri icon 命令**

```bash
pnpm tauri icon src-tauri/icons/icon-source.svg
```

预期输出（典型）：

```
        AppxManifest.xml
        Square30x30Logo.png
        Square44x44Logo.png
        Square71x71Logo.png
        Square89x89Logo.png
        Square107x107Logo.png
        Square142x142Logo.png
        Square150x150Logo.png
        Square284x284Logo.png
        Square310x310Logo.png
        StoreLogo.png
        icon.icns
        icon.ico
        32x32.png
        128x128.png
        128x128@2x.png
```

- [ ] **步骤 2：验证文件存在**

```bash
ls src-tauri/icons/ | grep -E "\.(png|ico|icns)$"
```

预期：列出全部 17 个目标文件。

- [ ] **步骤 3：验证文件大小合理**

```bash
# 每个 PNG 应大于 500 字节（避免空白/损坏）
ls -la src-tauri/icons/*.png src-tauri/icons/*.ico src-tauri/icons/*.icns
```

预期：所有文件大小 > 500 字节，icon.ico > 50KB（包含多尺寸），icon.icns > 100KB。

- [ ] **步骤 4：Commit**

```bash
git add src-tauri/icons/
git commit -m "feat(icons): regenerate tauri icon set from shixu source"
```

---

## 任务 3：清理意外新增的 SVG 资源

**文件：**
- 检查：`src-tauri/target/release/build/.../*.svg`

`tauri icon` 命令会在 `target/` 目录下生成临时 SVG 资源。这些位于 `target/` 不应被提交。

- [ ] **步骤 1：检查 git status 中是否包含 target/ 下的 SVG**

```bash
git status --short | grep "target/.*\.svg"
```

预期：**空输出**（target/ 已被 .gitignore 忽略）。

如果意外有 SVG 出现在 untracked 或 staged 列表，**不要 commit 它们**。检查 `.gitignore` 是否包含 `target/`。

- [ ] **步骤 2：确认 .gitignore 包含 target**

```bash
grep -E "^/?target/?$" .gitignore
```

预期：至少一行匹配 `target`。

---

## 任务 4：本地构建验证

**文件：**
- 验证：`src-tauri/target/release/bundle/` 下产物图标正确嵌入

- [ ] **步骤 1：运行构建**

```bash
pnpm tauri build
```

预期：构建成功，无 "icon not found" 警告。

- [ ] **步骤 2：验证产物图标（macOS）**

```bash
# 仅 macOS
ls src-tauri/target/release/bundle/macos/SHIXU\ OS.app/Contents/Resources/
```

预期：`icon.icns` 存在且 > 100KB。

- [ ] **步骤 3：验证产物图标（Windows）**

```bash
# 仅 Windows
ls src-tauri/target/release/bundle/msi/
```

预期：`.msi` 安装包生成。

- [ ] **步骤 4：视觉抽检**

在资源管理器/Dock 中查看生成的图标。预期：紫蓝圆角矩形 + 白色"拾序"二字（在不同 OS 上字体可能略有差异，这是预期的）。

---

## 任务 5：更新 README（可选）

**文件：**
- 修改：`README.md`

README 已有 App Icon 章节引用了 `pnpm tauri icon` 命令，无需修改。

仅当用户在审阅后发现需要补充示例图或链接到设计文档时再修改。

---

## 自检

| 检查 | 结果 |
|---|---|
| 规格覆盖度 | §2-§3 → 任务 1；§4 → 任务 2；§5 → 任务 1-2；§6 → 任务 4；§7 不在本期 → 无任务 |
| 占位符扫描 | 无 TODO/待定/类似任务 |
| 类型一致性 | 文件路径与设计文档一致；`tauri.conf.json` 的 `bundle.icon` 与任务 2 列表完全对齐 |
| 验证步骤 | 任务 2 步骤 2-3 文件存在/大小检查；任务 4 构建验证 |

---

## 执行交接

计划已完成并保存到 `docs/superpowers/plans/2026-09-18-shixu-logo.md`。两种执行方式：

**1. 子代理驱动（推荐）** - 每个任务调度一个新的子代理，任务间进行审查，快速迭代

**2. 内联执行** - 在当前会话中使用 executing-plans 执行任务，批量执行并设有检查点

选哪种方式？
