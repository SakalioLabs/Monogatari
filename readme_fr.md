<p align="center"><img src="docs/brand/logo-lockup.svg" width="720" alt="Monogatari"></p>

<p align="center">
  <a href="readme_en.md">English</a> · <a href="readme_ch.md">简体中文</a> ·
  <a href="readme_ja.md">日本語</a> · <a href="readme_ko.md">한국어</a> ·
  <a href="readme_de.md">Deutsch</a> · <a href="readme_fr.md">Français</a> ·
  <a href="readme_es.md">Español</a> · <a href="readme_ru.md">Русский</a>
</p>

# Monogatari

Monogatari est un atelier de création et un moteur d’exécution open source pour les visual novels natifs de l’IA. Il associe le jeu de rôle libre, ancré dans une scène, à des règles narratives déterministes : les dialogues restent vivants sans laisser le modèle contrôler directement les routes, les scores ou les fins.

> **État du projet :** la v0.9.5 est en développement actif. Les principaux parcours de création, d’exécution, de validation et de distribution Web/PWA et Windows sont en place ; les interfaces et contrats de données peuvent encore évoluer avant la 1.0.

## Principe distinctif

Chaque tour en direct sépare trois responsabilités :

1. Le **générateur de PNJ** produit uniquement la réponse visible, dans le rôle.
2. Un **évaluateur** indépendant propose des changements structurés de scores et de preuves.
3. La **machine à états déterministe** valide ces propositions et choisit seule la transition ou la fin.

Cette frontière rend la logique narrative auditable, protège le canon écrit et permet de rejouer les routes importantes sans fournisseur de modèle.

## Fonctionnalités

- Nœuds de jeu de rôle en temps réel avec objectifs, Knowledge, scores, preuves, budgets, transitions, délais et fins.
- Éditeurs visuels pour personnages, Knowledge, scènes, dialogues, workflows, événements, fins, audio et paramètres.
- Confinement des injections de prompt, contexte borné, contrôle d’ancrage et reprises écrites par l’auteur.
- Quality Suites rejouables pour les routes, la stabilité des personnages, les limites et les entrées hostiles.
- Distribution Web/PWA avec inférence WebGPU locale et ressources de projet hors ligne.
- Paquets Windows avec moteurs locaux compatibles ou services compatibles OpenAI dont la génération a été vérifiée.
- Live2D, sprites 2D, 3D GLB/GLTF et chaîne de repli du rendu.
- Paquets `.monogatari` vérifiés par chemins portables et inventaires SHA-256.
- MCP et Skill du dépôt pour une création assistée par Agent sous contrôle.

## Flux d’un tour

```text
Saisie libre du joueur
          │
          ├─ frontière de sécurité et d’ancrage
          ▼
 générateur du PNJ ──► dialogue visible
          │
          ▼
 évaluateur indépendant ──► propositions de scores/preuves
          │
          ▼
 machine à états déterministe ──► nœud suivant / fin
```

## Démarrage rapide

Prérequis : Node.js 20+, Rust stable et les dépendances de plateforme de Tauri 2. WebGPU n’est nécessaire que pour tester l’inférence locale dans le navigateur.

```bash
git clone https://github.com/SakalioLabs/Monogatari.git
cd Monogatari/frontend
npm install
npm run dev
```

Application Windows :

```bash
cd Monogatari/rust-engine/crates/tauri-app
cargo tauri dev
```

Validation :

```bash
node scripts/verify-modules.mjs --list
node scripts/verify-modules.mjs
node scripts/verify-release.mjs
```

## Organisation du dépôt

| Chemin | Rôle |
|---|---|
| `frontend/` | Interface Vue 3 + TypeScript, Playtest et build Web/PWA |
| `rust-engine/` | Moteur Rust, frontière de création, MCP et Tauri |
| `data/` | Projet canonique intégré et fixtures de qualité exécutables |
| `docs/` | Architecture, format de données, paquets, MCP et livraison |
| `.agents/skills/` | Processus de création Agent pour Monogatari |
| `scripts/` | Validation des modules, ressources, miroirs, paquets et versions |

Commencez par [`docs/DATA_FORMAT.md`](docs/DATA_FORMAT.md). Les intégrations Agent doivent employer le [serveur MCP](docs/MCP_SERVER.md) documenté et des transactions à préconditions exactes. Les identifiants secrets doivent rester hors des données du projet.

## Contribution et licence

Issues et Pull Requests sont les bienvenus. Consultez [CONTRIBUTING.md](CONTRIBUTING.md). Copyright © 2026 SakalioLabs. Distribué sous la permissive [licence MIT](LICENSE).
