---
name: author-visual-novel
description: Create, revise, validate, and package Monogatari visual novel projects from story briefs or existing project data. Use for visual novel, galgame, VN, character, dialogue, scene, event, ending, workflow, quality-suite, or automated story-authoring tasks, including requests to generate a complete playable project for people or agents.
---

# Author Visual Novels

Build project data that the real Monogatari runtime can load. Treat a project as a connected content graph, not a collection of unrelated JSON examples.

## Establish The Contract

1. Resolve the repository root with `git rev-parse --show-toplevel`.
2. Resolve the target project root. It must contain `settings.json`; the checked-in example root is `data/`.
3. Read [references/project-contract.md](references/project-contract.md). Read `docs/DATA_FORMAT.md` only for the content types being changed.
4. Inspect nearby checked-in examples before choosing field names or schema versions.
5. Keep runtime credentials outside project files. Never place API keys, tokens, passwords, or authorization headers in content or reports.

## Author In Dependency Order

1. Define a stable ID map and story bible before writing files.
2. Create knowledge and character records.
3. Add renderer assets and scenes, using project-relative portable paths.
4. Add dialogue graphs and verify every node, speaker, choice, and terminal path.
5. Add event unlock rules and endings only after their referenced content exists.
6. Add workflows after scene, dialogue, character, and event IDs are stable.
7. Add or update a Quality Suite that proves critical branches, character identity, knowledge boundaries, and prompt-safety behavior.
8. Keep mirrored `data/` and `rust-engine/data/` roots byte-equivalent when editing the built-in project.

Use structured JSON editing and preserve unrelated author changes. Do not invent a parallel schema or bypass runtime validation with a custom parser.

## Apply Agent Transactions Safely

When an Agent transport offers transaction planning or application, read [references/agent-transaction.md](references/agent-transaction.md). Use `missing` only for new files and an exact current SHA-256 for updates or deletions. Plan first, review every resolved path and resulting hash, then apply with the authoritative candidate-project validator. Do not use the transaction API as a substitute for graph, runtime, package, or experience validation.

## Validate The Result

Run narrow gates while iterating:

```powershell
node scripts/verify-modules.mjs --list
node scripts/verify-modules.mjs --module rust-authoring --module rust-game --module rust-tauri
```

Run the complete release gate before declaring a project deliverable:

```powershell
node scripts/verify-release.mjs
```

For UI or renderer changes, also exercise the relevant route at desktop and mobile sizes. A JSON parse, successful build, or generated image alone does not prove a playable story flow.

## Report Evidence

Summarize changed content IDs, reachable story paths, Quality Suite coverage, commands run, and any hardware-only or provider-only checks that remain unverified. Keep blockers explicit instead of silently substituting mock behavior.
