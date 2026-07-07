# Monogatari Release Checklist

## Pre-Release Verification

### Frontend
- [ ] `cd frontend && npm run build` passes with zero errors
- [ ] `npm audit` shows zero vulnerabilities
- [ ] All 13 views render correctly (Home, Chat, Game, Workflow, Assets, Characters, Group Chat, Settings, Analytics, Marketplace, Plugins, Audio, Character Editor)
- [ ] Sidebar navigation works for all 14 items
- [ ] Responsive layout verified on mobile viewport (375px) and tablet (768px)

### Rust Backend
- [ ] `cargo check --locked -p llm-galgame-app` passes
- [ ] All 22 command modules register correctly in main.rs
- [ ] Chat streaming works with API backend
- [ ] Character personality/knowledge injection verified

### Content
- [ ] Example characters load correctly (Sakura, Luna, Kenji)
- [ ] Example dialogues play through with choices
- [ ] Knowledge base search returns relevant results
- [ ] Scene assets validate without missing file warnings

### AI Integration
- [ ] API mode: streaming chat with OpenAI-compatible endpoint
- [ ] ONNX mode: local model inference (if applicable)
- [ ] Evaluation triggers fire at correct intervals
- [ ] Relationship milestones unlock events correctly

### Workflow Editor
- [ ] All 21 node types render in palette
- [ ] Drag-and-drop creates nodes on canvas
- [ ] Validation catches missing fields and broken links
- [ ] Export produces valid JSON

### Audio
- [ ] BGM tracks list and volume controls work
- [ ] SFX preview plays correctly
- [ ] Master mixer channels respond to input

### i18n
- [ ] Locale switching works (en, zh-CN, ja-JP, ko-KR)
- [ ] Nested key resolution works for all locale files

### Cloud Sync
- [ ] Push/pull commands execute without error
- [ ] Sync status displays correctly in Settings

## Distribution
- [ ] Version bumped in tauri.conf.json
- [ ] Version bumped in package.json
- [ ] CHANGELOG.md updated with release notes
- [ ] README.md version and features updated
- [ ] Git tag created: `git tag v0.6.3`
- [ ] Windows MSI installer built: `cargo tauri build`
- [ ] macOS DMG installer built (if applicable)
- [ ] Linux AppImage built (if applicable)
- [ ] Code signing applied to installers
- [ ] GitHub Release created with installers attached