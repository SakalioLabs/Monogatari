# Volume 4 Chapter 3 Production Plan

## Source boundary

- Reviewed all 618 non-empty paragraphs of Chapter 3 from the user's local
  Volume 4 EPUB.
- The adaptation begins at the caravan's arrival in Arcanletia and ends after
  the party regroups with the first bounded hot-spring sabotage warning.
- Sexualized bath comedy, coerced contact, religious humiliation, and violence
  toward children are not reproduced. Their dramatic functions become privacy,
  recruitment consent, faith-neutral visitor protection, and public-safety
  evidence handling.

## Runtime contract

The primary loop is `volume4_chapter3_roleplay`. The player writes free-form
Kazuma dialogue and may address any NPC present in the active node. Each
selectable NPC now receives an authored node-local `participant_goals` motive
in addition to their own Character profile, Knowledge, bounded transcript, and
relationship.

The evaluator remains separate from NPC generation and may only propose bounded
score and quoted-evidence changes. The deterministic roleplay state machine
validates those proposals and alone advances nodes or selects an ending. Fixed
Dialogue files are ending epilogues only.

## Live nodes

1. Arrival disclosure, lodging voucher, Wiz care, and party regroup plan.
2. Market performance boundaries and source-preserving health reports.
3. Explicit recruitment consent and faith-neutral public conduct.
4. Aqua's identity claim, current authority, confession, and church governance.
5. Hans and Wolbach's independently motivated sabotage conversation.
6. Source-labeled evidence handoff and reversible public-spring response.

## State model

The four bounded score dimensions are:

- `informed_agency`
- `civic_respect`
- `evidence_judgment`
- `hazard_response`

Each node owns two quoted evidence gates, for 12 total. Scores and evidence
select one of three deterministic endings:

- `volume4_documented_spring_warning`
- `volume4_warning_with_unresolved_authority`
- `volume4_tourism_record_overrun`

## Visual contract

Four generated environmental backgrounds establish Arcanletia's arrival plaza,
canal market, Axis church, and inn hot spring. Public anime references informed
the generated Hans and Wolbach designs. Both character assets use real alpha,
and all six renderer assets were imported through the fingerprint-bound project
asset workflow.

## Verification contract

`quality_suites/volume4_chapter3_roleplay.json` must prove:

- all 6 nodes and all 12 evidence gates on the complete route;
- exact `6/6/6/6` complete-route scores;
- all 3 endings;
- a full partial route with only transition-authorized evidence;
- a full zero-evidence timeout route;
- one guarded multilingual structural attack, zero unguarded intrusions, and
  zero forged score or evidence.

Core-runtime validation, delivery validation, Campaign preview, participant-goal
unit tests, repository JSON policy, browser desktop/mobile play, live provider
evidence, and the complete release gate remain required before this chapter is
pushed.
