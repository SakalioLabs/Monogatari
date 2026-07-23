<p align="center"><img src="docs/brand/logo-lockup.svg" width="720" alt="Monogatari"></p>

<p align="center">
  <a href="readme_en.md">English</a> · <a href="readme_ch.md">简体中文</a> ·
  <a href="readme_ja.md">日本語</a> · <a href="readme_ko.md">한국어</a> ·
  <a href="readme_de.md">Deutsch</a> · <a href="readme_fr.md">Français</a> ·
  <a href="readme_es.md">Español</a> · <a href="readme_ru.md">Русский</a>
</p>

# Monogatari

Monogatari は、オープンソースの AI ネイティブなビジュアルノベル制作ワークベンチ兼ランタイムです。シーン内での自由入力によるロールプレイと決定論的な物語ルールを組み合わせ、生成会話の自然さを保ちながら、モデルがルート・スコア・エンディングを直接操作できない設計になっています。

> **開発状況：** v0.9.5 は活発に開発中です。制作、ランタイム、検証、Web/PWA、Windows パッケージングの主要経路は実装済みですが、1.0 までは UI とデータ契約が変更される可能性があります。

## 設計上の特徴

ライブターンは、次の三つの責務に分離されます。

1. **NPC ジェネレーター**は、プレイヤーに見えるキャラクター内の返答だけを生成します。
2. 独立した**評価器**が、構造化されたスコアと証拠の変更を提案します。
3. **決定論的ステートマシン**が提案を検証し、遷移またはエンディングを単独で決定します。

この境界により、物語ロジックを監査でき、設定を守りながら、重要ルートをモデルなしでも再現検証できます。

## 主な機能

- 目標、Knowledge、スコア、証拠、予算、遷移、タイムアウト、エンディングを持つリアルタイムロールプレイノード。
- キャラクター、Knowledge、シーン、分岐会話、ワークフロー、イベント、エンディング、音声、設定のビジュアルエディター。
- プロンプトインジェクション対策、制限付きコンテキスト、グラウンディング検査、作者定義の復旧動作。
- ルート網羅、キャラクター安定性、境界値、敵対的入力を検証する再生可能な Quality Suite。
- WebGPU ローカル推論とオフライン素材を備えた Web/PWA 配布。
- 互換ローカルランタイムまたは検証済み OpenAI 互換サービスを利用する Windows デスクトップ版。
- Live2D、2D、GLB/GLTF 3D と段階的なレンダラーフォールバック。
- ポータブルパスと SHA-256 インベントリで検証される `.monogatari` パッケージ。
- MCP とリポジトリ Skill による、制御された Agent 支援制作。

## 処理の流れ

```text
プレイヤーの自由入力
        │
        ├─ 安全性・グラウンディング境界
        ▼
 NPC 返答生成 ──► 表示される会話
        │
        ▼
 独立評価器 ──► スコア / 証拠の提案
        │
        ▼
 決定論的ステートマシン ──► 次ノード / エンディング
```

## クイックスタート

必要環境は Node.js 20 以降、Rust stable、Tauri 2 のプラットフォーム依存ツールです。ブラウザ内ローカル推論のテスト時のみ、WebGPU 対応環境が必要です。

```bash
git clone https://github.com/SakalioLabs/Monogatari.git
cd Monogatari/frontend
npm install
npm run dev
```

Windows デスクトップ版：

```bash
cd Monogatari/rust-engine/crates/tauri-app
cargo tauri dev
```

検証：

```bash
node scripts/verify-modules.mjs --list
node scripts/verify-modules.mjs
node scripts/verify-release.mjs
```

## リポジトリ構成

| パス | 役割 |
|---|---|
| `frontend/` | Vue 3 + TypeScript の制作 UI、Playtest、Web/PWA |
| `rust-engine/` | Rust ランタイム、制作境界、MCP、Tauri |
| `data/` | 標準プロジェクトと実行可能な品質フィクスチャ |
| `docs/` | アーキテクチャ、データ形式、パッケージ、MCP、リリース文書 |
| `.agents/skills/` | Monogatari 向け Agent 制作ワークフロー |
| `scripts/` | モジュール、素材、ミラー、パッケージ、リリース検証 |

プロジェクト制作は [`docs/DATA_FORMAT.md`](docs/DATA_FORMAT.md) から始めてください。Agent 連携には [MCP サーバー](docs/MCP_SERVER.md)と厳密な事前条件付きトランザクションを使用し、認証情報をプロジェクトへ保存しないでください。

## コントリビューションとライセンス

Issue と Pull Request を歓迎します。詳しくは [CONTRIBUTING.md](CONTRIBUTING.md) をご覧ください。Copyright © 2026 SakalioLabs。本プロジェクトは寛容な [MIT License](LICENSE) で公開されています。
