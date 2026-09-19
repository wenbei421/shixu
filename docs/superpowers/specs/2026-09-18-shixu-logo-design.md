# SHIXU Logo 设计

> 日期：2026-09-18  
> 范围：品牌标识 + Tauri 图标全套替换  
> 状态：已批准（C3-2 · 标准居中）

## 1. 目标

替换 `src-tauri/icons/` 下全套默认 Tauri 图标，使用 SHIXU 自有品牌 logo：

- 圆角矩形容器 + 紫蓝底 + 白色"拾序"双字
- 视觉风格对齐国内主流 App 字标路线（淘宝"淘"、京东"京"、VIVO）
- 32 / 64 / 128 px 等所有应用尺寸下均可识别

## 2. 决策摘要

| 决策 | 选择 |
|---|---|
| 视觉风格 | 字标路线（圆角矩形 + 单字），参考国内 App 拼贴 |
| 容器形状 | 圆角矩形（rx=24，squircle 接近度 22%） |
| 容器底色 | `#A78BFA`（紫蓝 / Violet-400） |
| 字符内容 | "拾序"双字（中文全称） |
| 字符颜色 | `#FFFFFF` 纯白 |
| 字体 | `Source Han Sans CN` / `Noto Sans CJK SC` / `Microsoft YaHei` / `system-ui` fallback |
| 字重 | `font-weight: 900`（粗黑） |
| 字号 | `56`（在 128 viewBox 内） |
| 字间距 | `letter-spacing: -3` |
| 视觉对齐 | 居中（horizontal-anchor middle, baseline central） |
| 设计参考来源 | 用户提供的中国 App 图标拼贴 |

## 3. 视觉规范

### 3.1 几何参数

```svg
<svg viewBox="0 0 128 128">
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

### 3.2 颜色

| 用途 | 值 | 备注 |
|---|---|---|
| 容器底色 | `#A78BFA` | 主品牌色 |
| 字符颜色 | `#FFFFFF` | 纯白，对比度 3.2:1 |
| 深背景预览 | `#0F0F1A` | 仅用于设计预览，非图标本身 |

后续若要单色（透明背景）变体，字符改为 `#A78BFA`，容器改为透明。

### 3.3 字体 Fallback 链

构建产物在不同 OS 上的字体渲染：

| 平台 | 首选 | Fallback |
|---|---|---|
| macOS | PingFang SC / Source Han Sans CN | system-ui |
| Windows | Microsoft YaHei | Noto Sans CJK SC |
| Linux | Noto Sans CJK SC | system-ui |

字符最终以系统字体为准。如需跨平台像素一致，须把"拾序"二字转为 SVG `<path>`（不在本期范围）。

## 4. 资产清单

按 `tauri.conf.json` 当前 `bundle.icon` 配置，需生成以下图标：

| 文件 | 尺寸 | 用途 |
|---|---|---|
| `icons/32x32.png` | 32×32 | Windows 系统托盘、Linux 桌面 |
| `icons/128x128.png` | 128×128 | macOS Dock @1x |
| `icons/128x128@2x.png` | 256×256 | macOS Dock @2x |
| `icons/icon.icns` | 多尺寸 | macOS .app bundle |
| `icons/icon.ico` | 多尺寸 | Windows .exe 资源 |

按 Tauri v2 默认完整套件，还需保留（不替换会导致资源缺失）：

| 文件 | 尺寸 |
|---|---|
| `Square30x30Logo.png` | 30×30 |
| `Square44x44Logo.png` | 44×44 |
| `Square71x71Logo.png` | 71×71 |
| `Square89x89Logo.png` | 89×89 |
| `Square107x107Logo.png` | 107×107 |
| `Square142x142Logo.png` | 142×142 |
| `Square150x150Logo.png` | 150×150 |
| `Square284x284Logo.png` | 284×284 |
| `Square310x310Logo.png` | 310×310 |
| `StoreLogo.png` | 50×50（Windows Store） |

总计 **15 个 PNG + 1 个 ICNS + 1 个 ICO**。

## 5. 生成流程

### 5.1 源文件

源 SVG（单一源）保存到：

```
src-tauri/icons/icon-source.svg
```

内容即 §3.1 完整 `<svg>` 节点。

### 5.2 命令

使用 Tauri CLI 的 `icon` 子命令，从 SVG 源自动生成全套图标：

```bash
pnpm tauri icon src-tauri/icons/icon-source.svg
```

> 该命令依赖 `tauri.conf.json` 中声明的 `bundle.icon` 列表生成 PNG/ICO/ICNS，并写入 `src-tauri/icons/`。

### 5.3 提交清理

`src-tauri/target/release/build/.../tauri-codegen-assets/*.svg` 位于 target 目录，无需手动管理。

## 6. 验收

### 6.1 视觉验收

- [ ] 32×32 下"拾序"二字仍可辨认（不糊成一团）
- [ ] 256×256 下圆角弧度平滑、无锯齿
- [ ] macOS Dock 实际显示效果正确
- [ ] Windows 资源管理器图标显示正确
- [ ] 系统托盘（Windows）32×32 显示正确

### 6.2 构建验收

- [ ] `pnpm tauri build` 成功，无图标缺失警告
- [ ] 生成的 .app / .exe / .msi / .deb 资源中图标正常嵌入
- [ ] 移除任何一张 PNG 后构建会失败（验证配置与文件匹配）

### 6.3 文档验收

- [ ] README.md 中 App Icon 章节可保留原 `pnpm tauri icon` 说明
- [ ] `docs/superpowers/specs/2026-09-18-shixu-logo-design.md` 已 commit

## 7. 不在本期范围

- 暗色变体（深底浅字）
- 单色 / 透明背景变体（用于 favicon 等场景）
- "拾序"二字转 SVG `<path>` 的跨平台像素一致方案
- 启动画面 / 闪屏图（Splash Screen）
- 商店封面 / 营销物料

后续若需要，从同一 SVG 源派生即可。

## 8. 决策记录

| 版本 | 变更 | 原因 |
|---|---|---|
| v1 | 抽象几何（同心 / 点阵 / 中心收敛）多轮 | 用户反馈"不行换个风格"，全部弃用 |
| v2 | 具体物件（归档柜 / 索引卡 / 笔记本 / 磁带） | 用户反馈"不喜欢"，全部弃用 |
| v3 | 字标路线 4 候选（C1-C4） | 用户选 C3"拾序"双字 |
| v4 | C3 三种微调（C3-1/2/3） | 用户选 C3-2 标准居中 |
| 最终 | 圆角矩形 + 紫蓝 + 白色"拾序" | 已锁定 |
