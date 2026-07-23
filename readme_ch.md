<p align="center"><img src="docs/brand/logo-lockup.svg" width="720" alt="Monogatari"></p>

<p align="center">
  <a href="readme_en.md">English</a> · <a href="readme_ch.md">简体中文</a> ·
  <a href="readme_ja.md">日本語</a> · <a href="readme_ko.md">한국어</a> ·
  <a href="readme_de.md">Deutsch</a> · <a href="readme_fr.md">Français</a> ·
  <a href="readme_es.md">Español</a> · <a href="readme_ru.md">Русский</a>
</p>

# Monogatari

Monogatari 是一个开源的 AI 原生视觉小说创作工作台与运行时。它把场景内的自由角色扮演和确定性的叙事规则结合起来：生成式对话可以保持鲜活，但模型无权直接控制路线、分数或结局。

> **项目状态：** v0.9.5 正在积极开发中。核心创作、运行时、验证、Web/PWA 与 Windows 打包链路已实现；在 1.0 之前，界面和数据协议仍可能调整。

## 核心理念

每个实时回合都被拆分为三项职责：

1. **NPC 生成器**只负责可见的角色内回复。
2. 独立的**评估器**提出结构化的分数与证据变化。
3. **确定性状态机**验证这些变化，并独立决定转场或结局。

这条边界让剧情逻辑可审计、世界观不易失控，也让关键路线能够在不调用模型服务的情况下重复验证。

## 主要能力

- 实时角色扮演节点：目标、知识、评分维度、证据规则、推理预算、转场、超时与结局。
- 角色、知识库、场景、分支对话、工作流、事件、结局、音频和项目设置的可视化编辑器。
- 提示注入拦截、上下文限额、回复落地检查与作者预设的降级恢复。
- 可重放的质量套件，覆盖路线、角色稳定性、边界条件与对抗输入。
- 通过 WebGPU 本地推理并支持离线项目资源的 Web/PWA 交付。
- Windows 桌面包，可连接兼容的本地运行时或经过生成验证的 OpenAI 兼容服务。
- Live2D、2D 立绘、GLB/GLTF 3D 模型与逐级降级渲染。
- 使用便携路径和 SHA-256 清单验证的 `.monogatari` 项目导入导出。
- MCP 与仓库 Skill 支持受控的 Agent 辅助创作。

## 工作方式

```text
玩家自由输入
    │
    ├─ 安全与落地边界
    ▼
NPC 回复生成器 ──► 可见对话
    │
    ▼
独立评估器 ──► 分数/证据提案
    │
    ▼
确定性状态机 ──► 下一节点 / 结局
```

一个项目是以 `settings.json` 为根的内容图，包含角色、知识、场景、实时角色扮演、脚本对话、事件、结局、工作流、资源、本地化和可执行质量套件。

## 快速开始

### 环境要求

- Node.js 20 或更高版本
- Rust stable 工具链
- Tauri 2 对应平台的开发依赖
- 仅在测试浏览器本地推理时需要支持 WebGPU 的浏览器与显卡

### Web 创作界面

```bash
git clone https://github.com/SakalioLabs/Monogatari.git
cd Monogatari/frontend
npm install
npm run dev
```

### Windows 桌面应用

```bash
cd Monogatari/rust-engine/crates/tauri-app
cargo tauri dev
```

### 验证

```bash
node scripts/verify-modules.mjs --list
node scripts/verify-modules.mjs
node scripts/verify-release.mjs
```

开发时优先运行对应模块的窄验证；完整发布门禁覆盖更广，耗时也更长。

## 仓库结构

| 路径 | 用途 |
|---|---|
| `frontend/` | Vue 3 + TypeScript 创作界面、试玩与 Web/PWA 构建 |
| `rust-engine/` | Rust 运行时、创作边界、MCP 服务与 Tauri 桌面端 |
| `data/` | 内置项目的权威数据与可执行质量样例 |
| `docs/` | 架构、数据格式、打包、MCP 与发布文档 |
| `.agents/skills/` | 面向 Monogatari 项目的 Agent 创作流程 |
| `scripts/` | 模块、资源、镜像、项目包与发布验证 |
| `src/` | 仍受发布门禁覆盖的旧版 C# 兼容实现 |

## 创作项目

请从 [`docs/DATA_FORMAT.md`](docs/DATA_FORMAT.md) 开始，并始终保持内容图有效。对于 AI 角色扮演，应先创建结局和被引用内容，再编写角色扮演转场，最后用质量套件证明关键路线。仓库中的权威内置项目位于 `data/`，并镜像到 `rust-engine/data/`。

Agent 集成应使用文档化的 [MCP 服务](docs/MCP_SERVER.md)与精确前置条件事务流程。密钥不得写入项目数据。

## 文档

- [架构说明](docs/ARCHITECTURE.md)
- [数据格式](docs/DATA_FORMAT.md)
- [MCP 服务](docs/MCP_SERVER.md)
- [贡献指南](CONTRIBUTING.md)
- [安全策略](SECURITY.md)
- [行为准则](CODE_OF_CONDUCT.md)

## 参与贡献

欢迎提交 Issue 和 Pull Request。请先阅读 [CONTRIBUTING.md](CONTRIBUTING.md)，运行相关验证，并将无关改动保持分离。

## 许可证

Copyright © 2026 SakalioLabs。项目采用宽松的 [MIT License](LICENSE)。
