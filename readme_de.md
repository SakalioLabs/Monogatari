<p align="center"><img src="docs/brand/logo-lockup.svg" width="720" alt="Monogatari"></p>

<p align="center">
  <a href="readme_en.md">English</a> · <a href="readme_ch.md">简体中文</a> ·
  <a href="readme_ja.md">日本語</a> · <a href="readme_ko.md">한국어</a> ·
  <a href="readme_de.md">Deutsch</a> · <a href="readme_fr.md">Français</a> ·
  <a href="readme_es.md">Español</a> · <a href="readme_ru.md">Русский</a>
</p>

# Monogatari

Monogatari ist eine quelloffene, KI-native Autorenumgebung mit Laufzeit für Visual Novels. Sie verbindet freie, szenengebundene Rolleninteraktion mit deterministischen Erzählregeln. So bleibt generierter Dialog lebendig, ohne dass ein Modell Routen, Werte oder Enden direkt kontrolliert.

> **Projektstatus:** v0.9.5 wird aktiv entwickelt. Die zentralen Autoren-, Laufzeit-, Prüf-, Web/PWA- und Windows-Paketpfade sind implementiert; Oberflächen und Datenverträge können sich bis 1.0 noch ändern.

## Das besondere Prinzip

Jeder Live-Spielzug trennt drei Verantwortlichkeiten:

1. Der **NPC-Generator** schreibt ausschließlich die sichtbare Antwort in der Rolle.
2. Ein unabhängiger **Evaluator** schlägt strukturierte Wertungs- und Belegänderungen vor.
3. Die **deterministische Zustandsmaschine** prüft diese Vorschläge und wählt allein Übergänge oder Enden.

Diese Grenze macht Erzähllogik prüfbar, schützt den verfassten Kanon und ermöglicht providerfreie Wiederholungsprüfungen wichtiger Routen.

## Funktionen

- Echtzeit-Rollenspielknoten mit Zielen, Knowledge, Werten, Belegen, Budgets, Übergängen, Zeitlimits und Enden.
- Visuelle Editoren für Figuren, Knowledge, Szenen, Dialoge, Workflows, Ereignisse, Endrouten, Audio und Einstellungen.
- Schutz vor Prompt Injection, begrenzter Kontext, Grounding-Prüfungen und verfasste Wiederherstellung.
- Wiederholbare Quality Suites für Routenabdeckung, Figurenstabilität, Grenzfälle und feindliche Eingaben.
- Web/PWA-Auslieferung mit lokaler WebGPU-Inferenz und Offline-Projektinhalten.
- Windows-Pakete mit kompatiblen lokalen Laufzeiten oder verifizierten OpenAI-kompatiblen Diensten.
- Live2D, 2D-Sprites, GLB/GLTF-3D und abgestufte Renderer-Fallbacks.
- Geprüfte `.monogatari`-Pakete mit portablen Pfaden und SHA-256-Inventaren.
- MCP und Repository-Skill für kontrolliertes, agentengestütztes Authoring.

## Ablauf

```text
Freie Spielereingabe
        │
        ├─ Sicherheits- und Grounding-Grenze
        ▼
 NPC-Antwortgenerator ──► sichtbarer Dialog
        │
        ▼
 unabhängiger Evaluator ──► Wertungs-/Belegvorschläge
        │
        ▼
 deterministische Zustandsmaschine ──► nächster Knoten / Ende
```

## Schnellstart

Benötigt werden Node.js 20+, Rust stable und die Plattformvoraussetzungen für Tauri 2. WebGPU ist nur für lokale Browser-Inferenztests erforderlich.

```bash
git clone https://github.com/SakalioLabs/Monogatari.git
cd Monogatari/frontend
npm install
npm run dev
```

Windows-Desktopanwendung:

```bash
cd Monogatari/rust-engine/crates/tauri-app
cargo tauri dev
```

Prüfung:

```bash
node scripts/verify-modules.mjs --list
node scripts/verify-modules.mjs
node scripts/verify-release.mjs
```

## Repository

| Pfad | Zweck |
|---|---|
| `frontend/` | Vue-3-/TypeScript-Autorenoberfläche, Playtest und Web/PWA |
| `rust-engine/` | Rust-Laufzeit, Autorengrenze, MCP und Tauri |
| `data/` | Kanonisches Beispielprojekt und ausführbare Qualitätsfixtures |
| `docs/` | Architektur-, Datenformat-, Paket-, MCP- und Release-Dokumentation |
| `.agents/skills/` | Agenten-Workflow für Monogatari-Projekte |
| `scripts/` | Modul-, Asset-, Spiegel-, Paket- und Release-Prüfung |

Beginne das Authoring mit [`docs/DATA_FORMAT.md`](docs/DATA_FORMAT.md). Agentenintegrationen sollten den dokumentierten [MCP-Server](docs/MCP_SERVER.md) und Transaktionen mit exakten Vorbedingungen verwenden. Zugangsdaten gehören nie in Projektdaten.

## Mitwirken und Lizenz

Issues und Pull Requests sind willkommen. Bitte lies [CONTRIBUTING.md](CONTRIBUTING.md). Copyright © 2026 SakalioLabs. Veröffentlicht unter der freizügigen [MIT License](LICENSE).
