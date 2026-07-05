# Gate Design System

> 整个软件统一的 UI 基础。后续任何页面都必须基于本设计系统开发，禁止页面自带样式。

本设计系统是项目的 **单一事实来源（Single Source of Truth）**。所有颜色、字号、间距、圆角、阴影、动画均以 CSS Variables 形式集中定义在 `styles/tokens.css`，动画在 `styles/animations.css`。

---

## 一、技术栈与定位

| 维度 | 选型 |
| --- | --- |
| 框架 | Vue 3 + TypeScript |
| 桌面 | Tauri 2 |
| UI 库 | Naive UI（Provider 注入，业务弹层走 `useFeedback` 封装） |
| 图标 | Lucide Icons（统一经 `GIcon` + `icons/registry.ts`） |
| 主题 | Dark（默认） / Light（预留，`.theme-light`） |
| 定位参考 | Cursor / Arc / Docker Desktop / Linear |

**风格关键词**：Developer Tool、紧凑、信息密度、低对比中性底 + 单一强调色、清晰层级。

---

## 二、组件目录结构

```
client/src/
├── styles/
│   ├── tokens.css          # 设计令牌（颜色/字体/间距/圆角/阴影/动画/布局/层级）
│   ├── animations.css      # keyframes + 动画工具类 + 减弱动效
│   ├── global.scss         # （遗留，已被 tokens.css 覆盖）
│   └── variables/_colors.scss
│
├── components/
│   ├── icons/              # 图标系统
│   │   ├── GIcon.vue       # 统一图标包装（尺寸/颜色/spin/disabled）
│   │   └── registry.ts     # 图标名 → Lucide 组件 集中注册
│   │
│   ├── base/               # 原子基础组件（无状态、最通用、可复用）
│   │   ├── GButton.vue     # Primary/Secondary/Ghost/Danger/Text + Loading/Disabled + S/M/L
│   │   ├── GIconButton.vue # 纯图标按钮
│   │   ├── GCard.vue       # 卡片基底（plain/outlined/elevated/interactive）
│   │   └── GBadge.vue      # 徽章（solid/soft/outline/dot，6 色语义）
│   │
│   ├── form/               # 表单控件
│   │   ├── GInput.vue      # 文本输入基底（prefix/suffix/clearable/state）
│   │   ├── GPasswordInput.vue
│   │   ├── GTextarea.vue
│   │   ├── GNumberInput.vue
│   │   ├── GSearchInput.vue
│   │   ├── GPortInput.vue      # 端口（1-65535 校验着色）
│   │   ├── GHostInput.vue      # 主机/域名
│   │   ├── GTokenInput.vue     # Token/密钥（掩码+复制）
│   │   ├── GFormField.vue      # label/控件/提示/错误 容器
│   │   └── GLabel.vue
│   │
│   ├── feedback/           # 反馈类
│   │   ├── GSpinner.vue
│   │   ├── GSkeleton.vue       # text/rect/circle 骨架
│   │   ├── GProgress.vue       # 线性进度
│   │   ├── GCircleProgress.vue # 环形进度
│   │   ├── GEmptyState.vue     # 空状态
│   │   └── GErrorState.vue     # 错误状态 + 重试
│   │
│   ├── status/             # 状态展示
│   │   ├── GStatusDot.vue      # 圆点（脉冲/ping）
│   │   └── GStatusBadge.vue    # 圆点+文字（online/offline/connecting/...）
│   │
│   ├── cards/              # 卡片变体（半业务，可复用）
│   │   ├── GStatCard.vue       # 数字统计卡
│   │   └── GActionCard.vue     # 快捷操作卡
│   │
│   ├── layout/             # 布局构件
│   │   ├── GPageContainer.vue  # 页面容器（max-width/内边距）
│   │   ├── GPageHeader.vue     # 页面标题+描述+操作
│   │   └── GSectionHeader.vue  # 区块小标题+操作
│   │   （Sidebar/Toolbar/Inspector/StatusBar 见 layouts/DefaultLayout.vue）
│   │
│   ├── business/           # 业务组件（基于 base 组合，无业务逻辑）
│   │   ├── ProjectCard.vue
│   │   ├── TunnelCard.vue
│   │   ├── ServerCard.vue
│   │   └── StatisticsCard.vue
│   │
│   └── common/             # 通用非设计系统组件（遗留）
│       ├── NavItem.vue
│       └── LangSwitch.vue
│
├── composables/
│   ├── useFeedback.ts      # 统一 Toast/Confirm/Notify（封装 Naive）
│   └── useLocaleSwitcher.ts
│
└── plugins/
    └── designSystem.ts     # 全局注册所有 G* 组件
```

> `layouts/DefaultLayout.vue` 已实现 Sidebar / Header(Toolbar) / Content / Inspector / StatusBar 的整体网格布局，并使用本设计系统的令牌。

---

## 三、命名规范

| 范畴 | 规范 | 示例 |
| --- | --- | --- |
| 组件名 | `G` 前缀 + PascalCase | `GButton`、`GStatusBadge` |
| 组件文件 | 与组件名一致 | `GButton.vue` |
| CSS 变量（令牌） | `--<域>-<语义>` | `--color-primary`、`--space-4`、`--radius-md` |
| 组件内 class | `g-<组件名 kebab>__元素--修饰` (BEM) | `g-btn__icon--trailing` |
| 图标 | kebab-case 字符串，对齐 Lucide | `<GIcon name="arrow-right" />` |
| Props 枚举 | 小写语义 | `variant="primary"`、`size="md"`、`state="error"` |
| 尺寸档位 | `sm` / `md` / `lg`（必要时 `xs`/`xl`） | `size="sm"` |

**禁止**：页面内硬编码颜色/尺寸；散落内联 SVG；业务直接调 naive 原生 message/dialog。

---

## 四、Design Tokens 总览

全部位于 `styles/tokens.css`，分 12 个域。

### 1. Color System
- 语义色板：`--color-primary` / `--color-success` / `--color-warning` / `--color-error` / `--color-info`（各含 `-hover` / `-active` / `-muted` / `-fg`）
- 次品牌：`--color-secondary`
- 背景：`--color-background` / `--color-surface` / `--color-surface-hover` / `--color-surface-active` / `--color-surface-raised` / `--color-card` / `--color-overlay` / `--color-glass`
- 边框：`--color-border` / `-subtle` / `-strong` / `-accent` / `-focus`
- 文字：`--color-text-primary` / `-secondary` / `-tertiary` / `-disabled` / `-inverse` / `-on-primary`
- 状态：`--status-online/offline/warning/error/info/starting/connecting/reconnecting/updating/maintenance`（各含 `-bg`）

### 2. Typography
- 字体族：`--font-ui` / `--font-mono`
- 字号阶梯：`--text-xs(11)` / `sm(12)` / `base(13)` / `md(14)` / `lg(15)` / `xl(20)` / `2xl(24)` / `3xl(30)`
- 语义字号：`--font-size-title/subtitle/body/caption/code/button/input`
- 字重：`--weight-regular(400)/medium(500)/semibold(600)/bold(700)`
- 行高：`--leading-tight/normal/relaxed/loose`
- 字距：`--tracking-tight/normal/wide/wider`

### 3. Spacing（4px 栅格）
`--space-0/1(4)/2(8)/3(12)/4(16)/5(20)/6(24)/8(32)/10(40)/12(48)/16(64)`

### 4. Radius
- 基础：`--radius-none/xs(4)/sm(6)/md(8)/lg(10)/xl(12)/2xl(14)/3xl(18)/full`
- 语义：`--radius-card/button/input/dialog/popover/badge/tag`

### 5. Shadow
`--shadow-none/xs/sm/md/lg/floating/popup/hover/focus`（+ `--shadow-color`）

### 6. Animation
- 时长：`--duration-fast(150ms)` / `base(200ms)` / `slow(300ms)`（兼容别名 `micro/standard/entrance/spring`）
- 缓动：`--ease-linear/out/in/in-out/spring`
- 组合：`--transition-fast/base/slow`
- keyframes 与工具类见 `animations.css`（fade/slide/scale/dialog/toast/spin/pulse/ping/shimmer/blink）
- 页面切换：`<transition name="g-page">`
- 尊重 `prefers-reduced-motion`

### 7. Layout
`--sidebar-width(220)/sidebar-collapsed-width(48)/inspector-width(320)/toolbar-height(40)/statusbar-height(26)/header-height(48)/content-max-width(1200)/page-padding(24)`
控件高度：`--control-height-sm(28)/md(32)/lg(36)/xl(40)`

### 8. Z-Index
`--z-base/dropdown/sticky/fixed/overlay/modal/popover/toast/tooltip`

### Light Theme（预留）
`.theme-light` 类覆盖背景/边框/文字/阴影/滚动条；语义色板复用 Dark。

---

## 五、组件用途速查

### 基础组件（base，最通用，可复用）
| 组件 | 用途 | 复用度 |
| --- | --- | --- |
| `GButton` | 所有按钮，5 variant × 3 size + loading/disabled/icon | ★★★ |
| `GIconButton` | 工具栏/行内方形图标按钮 | ★★★ |
| `GCard` | 所有卡片容器基底 | ★★★ |
| `GBadge` | 状态/协议/计数标签 | ★★★ |

### 图标（icons）
| 组件 | 用途 |
| --- | --- |
| `GIcon` | 唯一图标入口，按名渲染 Lucide，统一尺寸/颜色/状态 |
| `registry.ts` | 集中注册图标名，新增图标只在此文件加一行 |

### 表单（form）
| 组件 | 用途 |
| --- | --- |
| `GInput` | 文本输入基底，prefix/suffix/clearable/state |
| `GPasswordInput` | 密码（显隐切换） |
| `GTextarea` | 多行 |
| `GNumberInput` | 数字（步进器） |
| `GSearchInput` | 搜索（搜索图标+清除+回车） |
| `GPortInput` | 端口（1-65535 自动着色） |
| `GHostInput` | 主机/域名（等宽） |
| `GTokenInput` | Token/密钥（掩码+复制） |
| `GFormField` | label+控件+hint/error 容器 |
| `GLabel` | 标签（必填标记） |

### 反馈（feedback）
| 组件 | 用途 |
| --- | --- |
| `GSpinner` | 行内加载 |
| `GSkeleton` | 骨架屏（text/rect/circle） |
| `GProgress` | 线性进度（含 indeterminate） |
| `GCircleProgress` | 环形进度 |
| `GEmptyState` | 空状态 |
| `GErrorState` | 错误状态 + 重试 |
| `useFeedback` | Toast/Confirm/ConfirmDanger/Notify（封装 Naive） |

### 状态（status）
| 组件 | 用途 |
| --- | --- |
| `GStatusDot` | 状态圆点（脉冲/ping） |
| `GStatusBadge` | 圆点+文字，内置 9 种运行态映射 |

### 卡片变体（cards，半业务可复用）
| 组件 | 用途 |
| --- | --- |
| `GStatCard` | 数字统计卡（图标+数值+标签+趋势） |
| `GActionCard` | 快捷操作入口卡（图标+文字+快捷键） |

### 布局（layout）
| 组件 | 用途 |
| --- | --- |
| `GPageContainer` | 页面外层（max-width + 内边距） |
| `GPageHeader` | 页面标题+描述+操作 |
| `GSectionHeader` | 区块小标题+操作 |
| `DefaultLayout` | Sidebar/Toolbar/Content/Inspector/StatusBar 整体网格 |

---

## 六、业务组件（business）

业务组件基于基础组件组合，**仅含展示结构与事件透传，不含数据获取/状态管理逻辑**（数据由父级传入）。

| 组件 | 组合自 | 说明 |
| --- | --- | --- |
| `ProjectCard` | GCard + GIcon + GStatusBadge + GStatusDot + GIconButton | 项目概览卡 |
| `TunnelCard` | GCard + GBadge + GStatusBadge + GIcon + GIconButton | 隧道卡（本地→远程路由 + 协议 + 流量） |
| `ServerCard` | GCard + GIcon + GStatusBadge + GIconButton + GCircleProgress | 服务器卡（延迟/负载/隧道数） |
| `StatisticsCard` | GCard + GStatCard + GSectionHeader | 统计区块（多 StatCard 聚合） |

> 这些是**模板外壳**，本阶段不实现 Tunnel/Server/Rust 业务逻辑。后续业务开发时直接填充数据与事件即可。

---

## 七、复用关系总览

```
业务组件 ──组合──▶ 卡片变体 ──组合──▶ 基础组件 ──使用──▶ 图标 / 令牌
  ProjectCard      GStatCard      GButton       GIcon
  TunnelCard       GActionCard    GCard         registry
  ServerCard                      GBadge        tokens.css
  StatisticsCard                  GInput        animations.css
                                  GFormField
                                  GStatusBadge ──▶ GStatusDot
                                  GCircleProgress
```

- **可复用（跨页面通用）**：`base/*`、`icons/*`、`form/*`、`feedback/*`、`status/*`、`cards/*`、`layout/*`
- **业务组件（特定领域）**：`business/*`（ProjectCard/TunnelCard/ServerCard/StatisticsCard）
- **非设计系统通用件**：`common/*`（NavItem、LangSwitch）

---

## 八、使用方式

所有 `G*` 组件已通过 `plugins/designSystem.ts` **全局注册**，页面内直接使用，无需 import：

```vue
<template>
  <GPageContainer>
    <GPageHeader description="项目总览">
      项目
      <template #actions>
        <GButton variant="primary" icon="plus">新建项目</GButton>
      </template>
    </GPageHeader>

    <GCard variant="plain" padding="md">
      <GFormField label="服务器地址" required>
        <GHostInput v-model="host" />
        <template #hint>支持域名或 IP</template>
      </GFormField>
      <GFormField label="端口">
        <GPortInput v-model="port" />
      </GFormField>
    </GCard>

    <ProjectCard
      name="My API" :tunnel-count="8" :online-count="3"
      status="online" last-active="刚刚"
      @click="..." @action="..."
    />
  </GPageContainer>
</template>

<script setup lang="ts">
import { useFeedback } from "@composables/useFeedback"
const { toast, confirm } = useFeedback()
</script>
```

---

## 九、约束（必须遵守）

1. 颜色/字号/间距/圆角/阴影/动画一律用令牌变量，**禁止硬编码**。
2. 图标一律走 `<GIcon name="..." />` + `registry.ts`，**禁止内联 SVG**。
3. 按钮一律用 `GButton` / `GIconButton`，**禁止页面自写 `.btn`**。
4. 通知/确认一律走 `useFeedback`，**禁止业务直调 naive message/dialog**。
5. 新增业务卡片：优先组合 `GCard` + 基础件，**不要重写容器样式**。
6. 新增图标：只在 `registry.ts` 增一行，全项目复用。
7. 主题切换：操作 `useTheme()` + `theme-dark`/`theme-light` 类，令牌自动跟随。
