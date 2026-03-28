# a2native

AI 代理的**原生桌面 UI 渲染器** —— 通过 egui 表单收集结构化用户输入，无需对话循环。

一个 JSON 进 → 原生窗口 → 一个 JSON 出。无需对话循环，无需 Web 服务器，无需浏览器。

> ⚠ **安全提示** —— a2native 渲染的每个窗口顶部均会显示安全警告横幅。
> 部署前请阅读[安全注意事项](#安全注意事项)。

---

## 在智能体协议栈中的位置

a2native 填补了智能体协议栈中的**原生桌面**层：

| 层级 | 协议 | 用途 |
|---|---|---|
| 代理 ↔ 工具/数据 | [MCP](https://modelcontextprotocol.io) | 让代理访问工具、文件、API |
| 代理 ↔ 代理 | [A2A](https://google.github.io/A2A/) | 代理之间的协调通信 |
| 代理 ↔ Web UI | [AG-UI](https://github.com/ag-ui-protocol/ag-ui) | 代理与浏览器之间的实时流式集成 |
| 代理 ↔ 生成式 UI | [**A2UI**](https://github.com/google/a2ui)（Google） | 面向 Web / Flutter 的声明式 JSON UI 规范 |
| **代理 ↔ 原生桌面** | **a2native** | **通过原生 OS 窗口同步收集表单输入** |

> **三个听起来相似但不同的东西：**
>
> | | [AG-UI](https://github.com/ag-ui-protocol/ag-ui) | [Google A2UI](https://github.com/google/a2ui) | **a2native** |
> |---|---|---|---|
> | 本质 | SSE 事件协议 | 声明式 JSON UI 规范 | CLI 渲染二进制 |
> | 传输层 | SSE / WebSocket | AG-UI 或 A2A | **stdin / stdout** |
> | 渲染环境 | 浏览器（Web） | Web + Flutter | **原生 OS 窗口（egui）** |
> | 交互模型 | 实时流式 | 增量曲面更新 | 请求 → 表单 → 响应 |
> | 部署依赖 | Node.js + SDK | 需要对应渲染器 | **单一二进制，零依赖** |
> | 适用场景 | 嵌入 Web 应用的代理交互 | 跨平台生成式 UI | **CLI/脚本代理流水线** |
>
> 三者**互补**：AG-UI 和 Google A2UI 处理 Web/应用端，a2native 处理原生桌面端。
> a2native 的输入格式在概念上受 A2UI（扁平组件列表、声明式）启发，
> 但针对同步 CLI 使用进行了适配 —— **并非** [Google A2UI 规范](https://github.com/google/a2ui)的实现。

a2native 是一个**人机协作（Human-in-the-loop，HITL）**工具 —— 它将控制权交给用户，通过**原生生成式 UI**（由代理在运行时生成组件的表单）收集结构化输入，再将控制权归还给代理。原本需要 10 轮对话的向导，可以变成一个简单的表单。

---

## 问题背景

AI 代理经常需要从用户处收集结构化输入 —— 一个选择、一个文件路径、一个日期范围、或一个多步骤配置向导。传统做法是来回对话：代理问一个问题，用户输入一个答案，如此循环。对于超过单个字段的场景，这既慢又容易出错，用户体验很差。

**a2native 用原生 UI 表单解决了这个问题：**

```
代理生成 JSON 表单规格
        ↓
   a2n 从 stdin 读取
        ↓
 原生窗口弹出（基于 egui）
        ↓
  用户填写字段并点击提交
        ↓
   JSON 结果写入 stdout
        ↓
     代理读取答案
```

一个原本需要 10 轮对话才能完成的向导，可以变成一个简单的表单。

---

## 输入格式

a2native 使用简洁的 JSON 输入格式 —— 带有可选标题、超时和主题的扁平组件列表。
该格式在概念上受 [Google A2UI](https://github.com/google/a2ui) 扁平组件列表方式的启发，
针对同步 CLI 使用进行了适配。

机器可读的 Schema 可通过以下方式获取：

- [schema/a2ui-v0.1.schema.json](schema/a2ui-v0.1.schema.json)（本仓库内）
- [`https://a2native.github.io/schema/a2ui-v0.1.schema.json`](https://a2native.github.io/schema/a2ui-v0.1.schema.json)（在线托管）
- `a2n schema` —— 在任意安装了 a2n 的机器上直接输出

```jsonc
{
  "title":   "我的表单",         // 可选，窗口标题
  "timeout": 60,                 // 可选，N 秒后自动关闭
  "theme": {                     // 可选
    "dark_mode": true,
    "accent_color": "#6C63FF"
  },
  "components": [ /* ... */ ]    // 必填
}
```

### 输出格式

```jsonc
{
  "status": "submitted" | "cancelled" | "timeout",
  "values": {
    "字段id": "用户输入值",    // 字符串、数字、布尔值或数组
    // ...
  }
}
```

### 组件参考

#### 展示类

| type | 必填字段 | 说明 |
|---|---|---|
| `text` | `id`, `content` | 纯文本标签 |
| `markdown` | `id`, `content` | 支持标题（`#`/`##`/`###`）和 `**粗体**` |
| `image` | `id`, `src` | 图片（文件路径或 URL），`alt` 可选 |
| `divider` | `id` | 水平分隔线 |

#### 输入类

| type | 关键字段 | 输出值类型 |
|---|---|---|
| `text-field` | `label`, `placeholder`, `required`, `default_value` | `string` |
| `textarea` | `label`, `placeholder`, `required`, `default_value` | `string` |
| `number-input` | `label`, `min`, `max`, `step`, `default_value` | `number` |
| `date-picker` | `label`, `required`, `default_value`（YYYY-MM-DD）| `string` |
| `time-picker` | `label`, `required`, `default_value`（HH:MM）| `string` |
| `dropdown` | `label`, `options`, `required`, `default_value` | `string` |
| `checkbox` | `label`, `default_value` | `boolean` |
| `checkbox-group` | `label`, `options`, `default_values` | `string[]` |
| `radio-group` | `label`, `options`, `required`, `default_value` | `string` |
| `slider` | `label`, `min`（0）, `max`（100）, `step`, `default_value` | `number` |
| `file-upload` | `label`, `accept`, `multiple` | `string`（路径，多选时用 `;` 分隔）|

`options` / `default_values` 使用 `{ "value": "...", "label": "..." }` 对象。

#### 操作类

| type | 关键字段 | 说明 |
|---|---|---|
| `button` | `label`, `action` | `action`：`"submit"`（默认）、`"cancel"`、`"custom"` |

#### 布局类

| type | 关键字段 | 说明 |
|---|---|---|
| `card` | `title`, `children` | 带边框的分组容器，children 可嵌套任意组件 |

---

## 安装

### 预构建二进制文件

从 [GitHub Releases](https://github.com/a2native/a2native/releases) 下载最新版本。

### 从源码构建

```bash
git clone https://github.com/a2native/a2native.git
cd a2native
cargo build --release
# 二进制文件：target/release/a2n（Windows 上为 a2n.exe）
```

需要 Rust ≥ 1.75。

---

## 使用方法

### 快速参考

```
a2n [JSON]                   一次性模式：内联 JSON 表单规格
echo '{...}' | a2n           一次性模式：通过 stdin 管道传入 JSON
a2n schema                   输出 a2native 输入 JSON Schema
a2n help                     显示使用说明
a2n --help                   显示参数帮助
a2n --version                显示版本
```

> **输入优先级：** 内联 JSON 参数 → stdin 管道 → （两者均无时 → 显示帮助）

### 一次性模式

可以通过内联参数或 stdin 管道提供 JSON：

```bash
# 内联参数
a2n '{"title":"部署确认","components":[
  {"id":"env","type":"dropdown","label":"部署环境",
   "options":[{"value":"prod","label":"生产环境"},
              {"value":"stag","label":"预发布环境"}]},
  {"id":"confirm","type":"checkbox","label":"我已确认变更内容"},
  {"id":"go","type":"button","label":"开始部署","action":"submit"}
]}'

# 或通过 stdin
echo '{"title":"部署确认","components":[...]}' | a2n
```

用户交互后的输出：

```json
{"status":"submitted","values":{"env":"prod","confirm":true}}
```

### 会话模式 —— 长生命周期窗口

使用 `--session <UUID>` 可让窗口在多个代理轮次之间保持开启。
每次代理调用 `a2n --session <UUID>` 时，窗口会更新显示新的表单，而不是关闭后重新打开。

```bash
# 第 1 轮 —— 第一个表单
echo '{"title":"步骤 1/3","components":[...]}' | a2n --session my-wizard-abc123
# → {"status":"submitted","values":{...}}

# 第 2 轮 —— 同一个窗口，显示新表单
echo '{"title":"步骤 2/3","components":[...]}' | a2n --session my-wizard-abc123
# → {"status":"submitted","values":{...}}

# 第 3 轮
echo '{"title":"步骤 3/3","components":[...]}' | a2n --session my-wizard-abc123

# 完成 —— 关闭窗口
a2n --close my-wizard-abc123
```

**工作原理：**

第一次 `--session` 调用会在后台派生一个守护进程来管理窗口。后续调用都是短暂的客户端进程，通过本地 TCP 套接字连接到守护进程，发送新的表单 JSON，并等待用户结果。这与 [agent-browser](https://github.com/vercel-labs/agent-browser) 使用的客户端-守护进程模式相同。

会话端口文件存储在系统临时目录中：
`{TMPDIR}/a2n-session-<UUID>.port`

### 关闭会话

```bash
a2n --close <UUID>
```

窗口关闭，守护进程干净退出。

### 帮助与 Schema

```bash
# 显示使用说明（无 JSON 参数且无 stdin 管道时自动显示）
a2n help

# 输出完整的 a2native 输入 JSON Schema
a2n schema

a2n --help      # 参数说明
a2n --version   # 版本信息
```

---

## 完整示例

```jsonc
{
  "title": "新项目向导",
  "timeout": 120,
  "theme": { "dark_mode": true, "accent_color": "#6C63FF" },
  "components": [
    { "id": "h",    "type": "markdown", "content": "## 配置你的项目" },
    { "id": "name", "type": "text-field", "label": "项目名称",
      "placeholder": "my-app", "required": true },
    { "id": "lang", "type": "radio-group", "label": "编程语言",
      "options": [
        {"value":"rust","label":"Rust"},
        {"value":"ts","label":"TypeScript"},
        {"value":"py","label":"Python"}
      ], "required": true },
    { "id": "oss",  "type": "checkbox", "label": "开源项目", "default_value": true },
    { "id": "div",  "type": "divider" },
    { "id": "ok",   "type": "button", "label": "创建项目", "action": "submit" },
    { "id": "no",   "type": "button", "label": "取消", "action": "cancel" }
  ]
}
```

---

## 安全注意事项

a2native 设计为由 AI 代理自动调用。这带来了用户和集成者必须了解的真实风险：

### 安全警告横幅

**a2native 渲染的每个窗口顶部都会显示永久的琥珀色警告横幅：**

> ⚠ 此界面由 AI 代理生成 —— 除非您信任该来源，否则请勿输入敏感信息。

此横幅无法被表单规格抑制。

### 已知风险

| 风险 | 描述 |
|---|---|
| **通过表单内容进行提示注入** | 被攻击的代理可能生成在视觉上模仿可信应用的表单（例如，伪造的系统密码对话框、伪造的银行登录页）。|
| **凭证窃取** | 恶意代理可能要求用户输入密码、API 密钥或个人数据，然后将其泄露。|
| **社会工程学** | Markdown 和文本组件允许任意消息；攻击者可以精心设计具有欺骗性的文字来误导用户。|
| **会话劫持** | 如果会话 UUID 可被猜测或不安全地复用，同一台机器上的其他进程可能向已开启的会话发送表单。|

### 缓解措施

- **仅从您信任的代理运行 a2native** —— 将其视为执行任意代码。
- 使用较短的 `timeout` 值，减少空闲会话窗口的暴露时间。
- 切勿在 a2native 表单中输入密码、私钥或金融数据。
- 优先使用随机、不可猜测的 UUID 作为会话标识（例如 UUID v4）。
- 在多用户系统上，注意 `TMPDIR` 中的会话端口文件可能对其他用户可见。

---

## 许可证

Apache-2.0 —— 详见 [LICENSE](LICENSE)。

| | |
|---|---|
| 输入格式 | [a2native schema v0.1](schema/a2ui-v0.1.schema.json) |
| 相关协议 | [Google A2UI](https://github.com/google/a2ui) · [AG-UI](https://github.com/ag-ui-protocol/ag-ui) |
| 渲染器 | [egui](https://github.com/emilk/egui) 0.29 |
| 文件选择器 | [rfd](https://github.com/PolyMeilex/rfd) 0.15 |
| CLI | [clap](https://github.com/clap-rs/clap) 4 |

---

[English README](README.md)
