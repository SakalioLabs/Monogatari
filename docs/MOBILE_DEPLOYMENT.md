# Mobile Deployment Preflight

Monogatari targets mobile through Tauri 2's Android and iOS command families. This page is a preflight checklist for turning the current Web/PWA and desktop shell into generated mobile projects without weakening the commercial release gate.

## Current Scope

- Android and iOS project generation is intentionally not checked in yet.
- `scripts/verify-tauri-mobile-preflight.mjs` verifies SDK-free readiness from the repository root.
- Generated mobile projects should be created from `rust-engine/crates/tauri-app` after Android Studio or Xcode prerequisites are installed.
- Production mobile builds must still pass `node scripts/verify-release.mjs` before publishing.

## Shared Configuration

- Tauri app root: `rust-engine/crates/tauri-app`
- Bundle identifier: `com.sakaliolabs.monogatari`
- Frontend dist: `../../../frontend/dist`
- Development URL: `http://localhost:5173`
- Frontend dev command: `cd ../../frontend && npm run dev`
- Frontend production command: `cd ../../frontend && npm run build`
- Vite mobile dev host: `process.env.TAURI_DEV_HOST`

## Android Path

Install Android Studio and configure:

- `JAVA_HOME`
- `ANDROID_HOME`
- `NDK_HOME`
- Android SDK Platform
- Android SDK Platform-Tools
- NDK (Side by side)
- Android SDK Build-Tools
- Android SDK Command-line Tools

From `rust-engine/crates/tauri-app`:

```bash
cargo tauri android init
cargo tauri android dev
cargo tauri android build
cargo tauri android run
```

Use Android Studio for device debugging when needed:

```bash
cargo tauri android dev --open
```

## iOS Path

iOS commands require a macOS host with Xcode installed.

From `rust-engine/crates/tauri-app`:

```bash
cargo tauri ios init
cargo tauri ios dev
cargo tauri ios build
cargo tauri ios run
```

For physical iOS device development, keep the Tauri CLI process alive and let Vite bind the host selected by Tauri:

```bash
cargo tauri ios dev --open --host
cargo tauri ios dev --force-ip-prompt
```

## Release Evidence

Before cutting a mobile release, attach:

- `node scripts/verify-tauri-mobile-preflight.mjs`
- `npm run verify:mobile-readiness`
- `npm run verify:responsive-shell`
- `npm run build:web`
- `node scripts/verify-release.mjs`
- Android APK/AAB build output and signing evidence
- iOS IPA/archive output and signing evidence

The preflight script does not replace SDK-specific Android/iOS builds. It proves that the checked-in cross-platform shell, Tauri configuration, Vite development server, and documentation are ready for mobile project generation.
