# Volume 6 Chapter 1 Live Roleplay Plan

## Product Contract

`volume6_chapter1_roleplay` is the playable chapter. It is a nine-node
real-time roleplay, not a fixed Dialogue. For each clean turn the runtime:

1. builds the selected NPC prompt from the active scene, that NPC's profile,
   node-local motive, bounded transcript, relationship, and pinned Knowledge;
2. generates only the NPC's visible in-character response;
3. calls a separate evaluator for bounded score and quoted-evidence proposals;
4. validates and clamps the proposal; and
5. lets the deterministic state machine choose the next node or ending.

NPC prose cannot set scores, forge evidence, select routes, or grant itself
authority. Fixed prose is limited to opening narration, containment responses,
generation/output recovery, and ending descriptions.

## Chapter Graph

1. Mansion roles: voluntary dinner and etiquette preparation.
2. Wiz shop credit: record contributions and agree promotion scope.
3. Street crowd: keep access, stop signals, and goods intact.
4. Estate preparation: disclose clothing and prop risks.
5. Royal greeting: let Iris speak directly while guards retain bounded duties.
6. Adventure account: distinguish fact, inference, and embellishment.
7. Card privacy: use redaction or a safe minimum-disclosure demonstration.
8. Conflict repair: stop insults and violence; require specific apologies.
9. Teleport handoff: disclose destination and obtain explicit revocable consent.

## State Model

- Scores: `mutual_respect`, `truth_and_provenance`,
  `consent_and_privacy`, and `public_safety`.
- Evidence: two node-local observations per node, eighteen total.
- Full ending: all eighteen evidence items and score `9` in every dimension.
- Deferred ending: all nine nodes visited, core conflict and teleport boundaries
  acknowledged, but one or more strict full-route requirements remain unmet.
- Exhaustion ending: bounded turns expire without a safe handoff.

## Acceptance

- Provider-free complete, partial, exhaustion, and structural-attack scenarios.
- 100% node coverage for the complete and deferred routes.
- Every attacked turn guarded, zero unguarded intrusions, zero score/evidence
  mutation on attacked turns.
- Live provider evidence is reported separately from deterministic replay.
- Desktop and mobile playtests must show the free-form composer as the primary
  story interaction and no traditional Dialogue line as the main loop.
