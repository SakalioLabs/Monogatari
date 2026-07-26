# Volume 4 Chapter 4 Production Plan

## Source boundary

- Reviewed all 432 non-empty paragraphs of Chapter 4 from the user's local
  Volume 4 EPUB.
- The adaptation begins with the inn breakfast investigation and ends when the
  guild reports fresh contamination across the city's hot springs.
- Sexualized comments, coerced contact, public humiliation, mob violence, and
  forged authority are not reproduced. Their dramatic functions become current
  consent, privacy, crowd protection, transparent credential use, and bounded
  public-safety authority.

## Runtime contract

The primary loop is `volume4_chapter4_roleplay`. The player writes free-form
Kazuma dialogue and may address any NPC present in the active node. Every
selectable primary or supporting NPC has an authored node-local
`participant_goals` motive in addition to their Character profile, Knowledge,
bounded transcript, and relationship.

The evaluator remains separate from NPC generation and may only propose bounded
score and quoted-evidence changes. The deterministic state machine validates
those proposals and alone advances nodes or selects an ending. Fixed Dialogue
files are ending epilogues only.

## Live nodes

1. Aqua's purification timing, contamination uncertainty, and Wiz's current care.
2. Wiz's history boundary, Megumin's red-haired mage clue, and Hans's town sighting.
3. Field spell, gear, magic-transfer, retreat, and Beginner's Bane response.
4. Spring-specific warning, business-loss records, de-escalation, and crowd safety.
5. Questionnaire fields, last-visitor fallacy, source labeling, and delayed report.
6. Guild evidence intake, bounded lookout authority, and voluntary noble credential.
7. Contact responsibilities, unresolved questions, and the fresh citywide alert.

## State model

The four bounded score dimensions are:

- `evidence_integrity`
- `public_safety`
- `consent_and_care`
- `accountable_authority`

Each node owns two quoted evidence gates, for 14 total. Scores and evidence
select one of three deterministic endings:

- `volume4_bounded_hans_lookout_and_fresh_alert`
- `volume4_lookout_with_disputed_public_record`
- `volume4_investigation_handoff_overrun`

Chapter 4 is the current Campaign production frontier. Its three endings will
route into Chapter 5 when that source-bounded Roleplay is authored.

## Visual contract

Four generated backgrounds establish the inn dining room, forest edge, public
square, and adventurers-guild office. Existing Aqua, Darkness, Megumin, and Wiz
emotion sprites are reused. The original adult Arcanletia guild clerk uses a
generated real-alpha character asset. All five renderer assets pass the
fingerprint-bound project importer.

## Verification contract

`quality_suites/volume4_chapter4_roleplay.json` must prove:

- all 7 nodes and all 14 evidence gates on the complete route;
- exact `7/7/7/7` complete-route scores;
- all 3 endings;
- a full partial route with only transition-authorized evidence;
- a full zero-evidence timeout route;
- one guarded multilingual structural attack, zero unguarded intrusions, and
  zero forged score or evidence.

Core-runtime validation, delivery validation, Campaign preview, transition
shadowing unit tests, repository JSON policy, browser desktop/mobile play, live
provider evidence, and the complete release gate remain required before this
chapter is pushed.
