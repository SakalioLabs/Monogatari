# Volume 4 Chapter 2 Production Plan

## Source boundary

- Reviewed all 666 non-empty paragraphs of Chapter 2 from the user's local
  Volume 4 EPUB.
- The adaptation ends after the caravan's first overnight camp and the undead
  response.
- Sexualized jokes, private-body detail, coerced restraint, and violence toward
  a minor are not reproduced. Their useful dramatic function is represented as
  privacy, consent, disclosure, and duty-of-care decisions.

## Runtime contract

The primary loop is `volume4_chapter2_roleplay`. The player writes free-form
Kazuma dialogue and may address any NPC present in the active node. NPC output
is generated from the active scene, the selected character's identity and
Knowledge, bounded transcript, and current relationship.

The evaluator is separate from NPC generation and can only propose bounded
score and quoted-evidence changes. The deterministic roleplay state machine
validates those proposals and alone advances nodes or selects an ending. Fixed
Dialogue files are ending epilogues only.

## Live nodes

1. Wealth, recovery, and an individually chosen hot-spring trip.
2. Vanir's royalty/buyout offer and unsafe shop inventory disclosure.
3. Fair caravan seating and consent-based care for Wiz.
4. Testable Jumping Hawk observations and a bounded, reversible decoy role.
5. Reversible crowd control, cave inspection, and explosion clearance.
6. Camp reward disclosure, undead response, and public accountability.

## State model

The four bounded score dimensions are:

- `responsible_agency`
- `operational_judgment`
- `party_coordination`
- `public_accountability`

Each node owns two quoted evidence gates, for 12 total. Scores and evidence
select one of three deterministic endings:

- `volume4_transparent_caravan_arrival`
- `volume4_arrival_with_unresolved_accountability`
- `volume4_caravan_incident_overrun`

## Visual contract

Two generated environmental backgrounds establish the spring caravan road and
the guarded overnight camp. Existing mansion, magic-shop, and character assets
are reused to preserve visual identity. Both new images were imported through
the fingerprint-bound project asset workflow.

## Verification contract

`quality_suites/volume4_chapter2_roleplay.json` must prove:

- all 6 nodes and all 12 evidence gates on the complete route;
- exact `6/6/6/6` complete-route scores;
- all 3 endings;
- a full partial route with only transition-authorized evidence;
- a full zero-evidence timeout route;
- one guarded multilingual structural attack, zero unguarded intrusions, and
  zero forged score or evidence.

Core-runtime validation, delivery validation, Campaign preview, repository JSON
policy, browser desktop/mobile play, live provider evidence, and the complete
release gate remain required before this chapter is pushed.
