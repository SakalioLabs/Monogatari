<p align="center"><img src="docs/brand/logo-lockup.svg" width="720" alt="Monogatari"></p>

<p align="center">
  <a href="readme_en.md">English</a> · <a href="readme_ch.md">简体中文</a> ·
  <a href="readme_ja.md">日本語</a> · <a href="readme_ko.md">한국어</a> ·
  <a href="readme_de.md">Deutsch</a> · <a href="readme_fr.md">Français</a> ·
  <a href="readme_es.md">Español</a> · <a href="readme_ru.md">Русский</a>
</p>

# Monogatari

Monogatari es un entorno de autoría y motor de ejecución de código abierto para novelas visuales nativas de IA. Combina la interpretación libre dentro de una escena con reglas narrativas deterministas, para que el diálogo generado se sienta vivo sin entregar al modelo el control de rutas, puntuaciones o finales.

> **Estado del proyecto:** v0.9.5 está en desarrollo activo. Los flujos principales de autoría, ejecución, validación y empaquetado Web/PWA y Windows están implementados; las interfaces y los contratos de datos aún pueden cambiar antes de 1.0.

## Qué lo hace diferente

Cada turno en vivo separa tres responsabilidades:

1. El **generador de PNJ** escribe únicamente la respuesta visible y dentro del personaje.
2. Un **evaluador** independiente propone cambios estructurados de puntuación y evidencias.
3. La **máquina de estados determinista** valida las propuestas y elige por sí sola la transición o el final.

Este límite hace auditable la lógica narrativa, protege el canon escrito y permite repetir las rutas importantes sin depender de un proveedor de modelos.

## Funciones principales

- Nodos de rol en tiempo real con objetivos, Knowledge, puntuaciones, evidencias, presupuestos, transiciones, tiempos límite y finales.
- Editores visuales de personajes, Knowledge, escenas, diálogos, flujos, eventos, finales, audio y ajustes.
- Contención de inyección de prompts, contexto acotado, comprobación de anclaje y recuperación escrita por el autor.
- Quality Suites reproducibles para cobertura de rutas, estabilidad de personajes, límites y entradas hostiles.
- Distribución Web/PWA con inferencia WebGPU local y recursos de proyecto sin conexión.
- Paquetes para Windows con motores locales compatibles o servicios compatibles con OpenAI cuya generación se ha verificado.
- Live2D, sprites 2D, 3D GLB/GLTF y rutas graduales de respaldo de renderizado.
- Paquetes `.monogatari` verificados mediante rutas portables e inventarios SHA-256.
- MCP y Skill del repositorio para autoría asistida por Agent de forma controlada.

## Flujo de un turno

```text
Entrada libre del jugador
          │
          ├─ límite de seguridad y anclaje
          ▼
 generador del PNJ ──► diálogo visible
          │
          ▼
 evaluador independiente ──► propuestas de puntuación/evidencia
          │
          ▼
 máquina de estados determinista ──► siguiente nodo / final
```

## Inicio rápido

Requisitos: Node.js 20+, Rust stable y los requisitos de plataforma de Tauri 2. WebGPU solo es necesario para probar la inferencia local en el navegador.

```bash
git clone https://github.com/SakalioLabs/Monogatari.git
cd Monogatari/frontend
npm install
npm run dev
```

Aplicación de Windows:

```bash
cd Monogatari/rust-engine/crates/tauri-app
cargo tauri dev
```

Verificación:

```bash
node scripts/verify-modules.mjs --list
node scripts/verify-modules.mjs
node scripts/verify-release.mjs
```

## Mapa del repositorio

| Ruta | Función |
|---|---|
| `frontend/` | Interfaz Vue 3 + TypeScript, Playtest y Web/PWA |
| `rust-engine/` | Motor Rust, límite de autoría, MCP y Tauri |
| `data/` | Proyecto integrado canónico y fixtures de calidad ejecutables |
| `docs/` | Arquitectura, formato de datos, paquetes, MCP y entregas |
| `.agents/skills/` | Flujo de autoría con Agent para Monogatari |
| `scripts/` | Verificación de módulos, recursos, espejos, paquetes y versiones |

Empieza por [`docs/DATA_FORMAT.md`](docs/DATA_FORMAT.md). Las integraciones con Agent deben usar el [servidor MCP](docs/MCP_SERVER.md) documentado y transacciones con precondiciones exactas. Las credenciales deben permanecer fuera de los datos del proyecto.

## Contribución y licencia

Se aceptan Issues y Pull Requests. Consulta [CONTRIBUTING.md](CONTRIBUTING.md). Copyright © 2026 SakalioLabs. Publicado bajo la permisiva [Licencia MIT](LICENSE).
