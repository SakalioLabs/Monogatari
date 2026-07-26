# Volume 4 Chapter 1 Production Plan

## Source boundary

- Reviewed the 19-paragraph prologue and all 470 paragraphs of Chapter 1 from
  the user's local Volume 4 EPUB.
- The prologue is the first live mansion node, not a fixed introduction.
- The chapter ends after Kazuma's revival, the two-week recovery order, and a
  privacy-preserving record of the unauthorized body prank.
- No private-body detail is reproduced or visualized.

## Runtime contract

The primary loop is `volume4_chapter1_roleplay`. The player writes free-form
Kazuma dialogue and selects any NPC present in the active node. The selected
NPC response is generated from that character's profile, current scene,
bounded transcript, relationship state, and pinned Knowledge.

NPC prose has no route authority. A separate evaluator may propose bounded
score changes and player-quote evidence. Only the deterministic Scene Roleplay
state machine validates those proposals, changes state, advances nodes, and
selects an ending. Fixed Dialogue files are ending epilogues only.

## Live nodes

1. Wealth, rest, and a non-coercive mission choice with Aqua.
2. Sena's Running Lizard briefing without level-based shame.
3. Armor fit testing and owner-confirmed weapon naming with Megumin.
4. Aqua's participation choice without food or friendship exclusion.
5. Target identification, mana checks, and retreat contingencies with Darkness.
6. Force Fire deviation and the revised Running Princess priority with Aqua.
7. Mana failure, injury, death, and mission-result separation with Megumin.
8. An informed return-or-reincarnation choice with Eris.
9. Post-revival privacy, recovery, and future-conduct terms with Aqua.

## State model

The four bounded score dimensions are:

- `autonomous_commitment`
- `operational_readiness`
- `adaptive_tactics`
- `revival_body_boundary`

Each node owns two quoted evidence gates, for 18 total. Scores and evidence
select one of three deterministic endings:

- `volume4_return_with_recovery_and_party_terms`
- `volume4_return_with_unresolved_body_boundary`
- `volume4_revival_decision_overrun`

## Visual contract

Three generated 1536x864 backgrounds establish the spring mansion, Axel
blacksmith, and residual-snow Running Lizard field. Aqua's new transparent
`hearth_stubborn` state preserves her existing identity. The afterlife hall
and existing Aqua, Megumin, Darkness, Sena, and Eris states are reused where
they already match the source setting.

All four new assets were imported through the fingerprint-bound project asset
workflow and are declared through Scene or Character JSON.

## Verification contract

`quality_suites/volume4_chapter1_roleplay.json` must prove:

- all 9 nodes and all 18 evidence gates on the complete route;
- exact `9/9/9/9` complete-route scores;
- all 3 endings;
- a full partial route with only node-authorized score dimensions;
- a full zero-evidence timeout route;
- one guarded structural attack, zero unguarded intrusions, and zero forged
  score or evidence.

Core-runtime validation, delivery validation, Campaign preview, repository JSON
policy, browser desktop/mobile play, and the complete release gate remain
required before this chapter is pushed.
