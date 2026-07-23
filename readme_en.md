<p align="center"><img src="docs/brand/logo-lockup.svg" width="720" alt="Monogatari"></p>

<p align="center">
  <a href="readme_en.md">English</a> · <a href="readme_ch.md">简体中文</a> ·
  <a href="readme_ja.md">日本語</a> · <a href="readme_ko.md">한국어</a> ·
  <a href="readme_de.md">Deutsch</a> · <a href="readme_fr.md">Français</a> ·
  <a href="readme_es.md">Español</a> · <a href="readme_ru.md">Русский</a>
</p>

# Monogatari

Monogatari is an open-source, AI-native visual novel authoring workbench and runtime. It combines free-form, scene-bound character roleplay with deterministic narrative rules, so generated dialogue can feel alive without giving a model control over routes, scores, or endings.

> **Project status:** v0.9.5 is under active development. The core authoring, runtime, validation, Web/PWA, and Windows packaging paths are implemented; interfaces and data contracts may still evolve before 1.0.

## What makes it different

Each live turn is split into three explicit responsibilities:

1. The **NPC generator** writes only the visible in-character response.
2. A separate **evaluator** proposes structured score and evidence observations.
3. The **deterministic state machine** validates those observations and alone selects transitions or endings.

That boundary makes story logic auditable, guards authored canon, and allows provider-free replay of important routes.

## Highlights

- Real-time roleplay nodes with goals, Knowledge, score dimensions, evidence rules, budgets, transitions, timeouts, and endings.
- Visual editors for characters, Knowledge, scenes, branching dialogue, workflows, events, ending routes, audio, and project settings.
- Prompt-injection containment, bounded context, grounded response checks, and authored recovery behavior.
- Replayable Quality Suites for route coverage, character stability, boundary conditions, and adversarial turns.
- Web/PWA delivery with local WebGPU inference and offline project assets.
- Windows desktop packages with compatible local runtimes or generation-verified OpenAI-compatible endpoints.
- Live2D, 2D sprite, GLB/GLTF 3D, and graceful renderer fallback paths.
- Verified `.monogatari` project import/export with portable paths and SHA-256 inventories.
- MCP and repository Skill support for controlled agent-assisted authoring.

## How it fits together

```text
Free-form player input
          │
          ├─ safety + grounding boundary
          ▼
 NPC response generator ──► visible dialogue
          │
          ▼
  independent evaluator ──► proposed scores/evidence
          │
          ▼
 deterministic state machine ──► next node / ending
```

Projects are content graphs rooted in `settings.json`, with characters, Knowledge, scenes, roleplays, dialogue, events, endings, workflows, assets, locales, and executable quality suites.

## Quick start

### Requirements

- Node.js 20 or newer
- Rust stable toolchain
- Tauri 2 platform prerequisites
- A WebGPU-capable browser/GPU only when testing local browser inference

### Web authoring UI

```bash
git clone https://github.com/SakalioLabs/Monogatari.git
cd Monogatari/frontend
npm install
npm run dev
```

### Windows desktop app

```bash
cd Monogatari/rust-engine/crates/tauri-app
cargo tauri dev
```

### Verification

```bash
node scripts/verify-modules.mjs --list
node scripts/verify-modules.mjs
node scripts/verify-release.mjs
```

Use the narrow module gates while developing; the complete release gate is intentionally broader and slower.

## Repository map

| Path | Purpose |
|---|---|
| `frontend/` | Vue 3 + TypeScript authoring UI, Playtest, and Web/PWA build |
| `rust-engine/` | Rust runtime, authoring boundary, MCP server, and Tauri desktop app |
| `data/` | Canonical built-in project and executable quality fixtures |
| `docs/` | Architecture, data format, packaging, MCP, and release documentation |
| `.agents/skills/` | Agent authoring workflow for Monogatari projects |
| `scripts/` | Module, asset, mirror, package, and release verification |
| `src/` | Legacy C# compatibility implementation covered by release checks |

## Authoring a project

Start from [`docs/DATA_FORMAT.md`](docs/DATA_FORMAT.md) and keep the project graph valid. For AI roleplay, author endings and referenced content before roleplay transitions, then add Quality Suites that prove the critical routes. The checked-in project in `data/` is mirrored into `rust-engine/data/`.

Agent integrations should use the documented [MCP server](docs/MCP_SERVER.md) and exact-precondition transaction flow. Credentials must stay outside project data.

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Data format](docs/DATA_FORMAT.md)
- [MCP server](docs/MCP_SERVER.md)
- [Contributing](CONTRIBUTING.md)
- [Security policy](SECURITY.md)
- [Code of conduct](CODE_OF_CONDUCT.md)

## Contributing

Issues and pull requests are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md), use the relevant narrow verification gates, and keep unrelated changes separate.

## License

Copyright © 2026 SakalioLabs. Released under the permissive [MIT License](LICENSE).
