<p align="center">
  <img src="docs/brand/logo-lockup.svg" width="760" alt="Monogatari — AI Narrative Engine">
</p>

<p align="center">
  <strong>Author living stories. Keep the rules in your hands.</strong><br>
  AI-native visual novel authoring and deterministic scene-roleplay runtime for Web/PWA and Windows.
</p>

<p align="center">
  <a href="https://github.com/SakalioLabs/Monogatari/actions/workflows/ci.yml"><img src="https://github.com/SakalioLabs/Monogatari/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-E4573D.svg" alt="MIT License"></a>
  <img src="https://img.shields.io/badge/version-0.9.5-17181C.svg" alt="Version 0.9.5">
</p>

<p align="center">
  <a href="readme_en.md">English</a> ·
  <a href="readme_ch.md">简体中文</a> ·
  <a href="readme_ja.md">日本語</a> ·
  <a href="readme_ko.md">한국어</a> ·
  <a href="readme_de.md">Deutsch</a> ·
  <a href="readme_fr.md">Français</a> ·
  <a href="readme_es.md">Español</a> ·
  <a href="readme_ru.md">Русский</a>
</p>

---

Monogatari is an open-source authoring workbench and runtime toolkit for AI-native visual novels. Players speak freely inside authored scenes; an NPC model writes the response, a separate evaluator proposes evidence and score changes, and only a deterministic state machine may select the next scene or ending.

This separation lets creators use generative characters without surrendering story structure, safety boundaries, or route logic.

### Why Monogatari

- **Living scene roleplay** — free-form player input grounded in characters, lore, goals, and bounded context.
- **Deterministic narrative control** — authored transitions, evidence, scores, timeouts, and endings remain inspectable and testable.
- **Visual authoring** — editors for characters, Knowledge, scenes, dialogue, workflows, events, endings, audio, and packages.
- **Local-first delivery** — WebGPU in Web/PWA builds, compatible local runtimes on desktop, and optional OpenAI-compatible services.
- **Quality as content** — replayable suites cover routes, endings, character stability, scoring boundaries, and adversarial input.
- **Portable projects** — validated `.monogatari` packages with bounded paths and SHA-256 inventories.

Start with the [English documentation](readme_en.md), [中文说明](readme_ch.md), or the [architecture guide](docs/ARCHITECTURE.md).

Agent clients can use `monogatari-mcp`, which exposes fourteen standard stdio tools
for project inspection, validation, deterministic previews, Quality
execution, reviewed transactions, and package exchange. See the
[MCP server guide](docs/MCP_SERVER.md).

## License

Monogatari is available under the permissive [MIT License](LICENSE).
