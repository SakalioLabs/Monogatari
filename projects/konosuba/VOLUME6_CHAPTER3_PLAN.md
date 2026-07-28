# Volume 6 Chapter 3 Roleplay Plan

`volume6_chapter3_roleplay` is the playable chapter. It is a seven-node
real-time interaction graph, not a fixed Dialogue.

## Source and adaptation boundary

The chapter design was grounded in all 574 heading and paragraph nodes from
the user's local Volume 6 EPUB, `OEBPS/Text/chapter03.xhtml`. The adaptation
preserves chronology, scene facts, and character knowledge boundaries without
copying source prose. Runtime NPC wording is generated from the active scene,
selected character, node-local motive, bounded transcript, relationship, and
pinned Knowledge.

## Runtime loop

1. The party accepts a bounded watch assignment at Alderp's villa.
2. The player must stop and remedy a hidden-observation privacy violation.
3. A night intruder is identified with proportionate, nonsexual restraint.
4. Chris can disclose an artifact lead without forced identity exposure.
5. The royal report separates success, failure, sources, and unknowns.
6. Mitsurugi's warning is ranked by source while Aqua retains her own voice.
7. Artifact reconnaissance requires limited authority, supervision, exit
   conditions, and a reporting duty.

The NPC model generates only the in-scene reply. A separate evaluator may
propose clamped score deltas and exact player-quote evidence. Only the
deterministic Roleplay state machine advances the node graph or selects an
ending.

## Route authority

Four scores govern the route: `lawful_investigation`,
`truth_and_provenance`, `privacy_and_dignity`, and `team_trust`. Fourteen
independent evidence gates prevent a model from converting suspicion,
reputation, hidden identity, or claimed artifact danger into route authority.

The reachable endings are:

- `volume6_artifact_recovery_with_bounded_authority`
- `volume6_supervised_artifact_inquiry`
- `volume6_unauthorized_heist_refused`

Fixed Dialogue is limited to the three short post-ending epilogues.

## Visual production

Six original 16:9 backgrounds were generated and fingerprint-imported:

- Alderp villa reception:
  `0f2f8602ddd066888ddb1e7bbd4a4b0a9bd35c3a874d813ef68977743d33ec64`
- hidden-mirror room:
  `eebec238b3ed0c1defb890791e898351654e57add2e05587ecf5faa5ce9f2f2e`
- villa kitchen at night:
  `957b38dff514a4797ee28c80b3ec06ce3fc289201a1d566d0541de7a65c54046`
- royal audience hall:
  `dd69ef22369f230bb87ec7ea4c455dc68fb55b69395be3e555c8e444e73f67f6`
- royal-capital cafe:
  `1a6146af0468a0d91fc5778a9c62c66445baf67ca0d9461bd9dd0be7618e7e46`
- royal-capital inn room:
  `08d4c66a37f3f1d905b2088c8908c31d8c30a99620ae47d181fc3a5525b8ad5c`

Existing reviewed character sprites are reused. Public distribution requires
a separate rights review; this remains a noncommercial engine simulation.

## Acceptance

- strict, supervised, and exhaustion endings are reachable;
- every clean route covers all seven nodes;
- all fourteen evidence IDs are independently required on the strict route;
- the structural intrusion remains on `alderp_villa_watch_charter`;
- the attacked turn commits zero story turns, score, evidence, or route state;
- the provider-free Quality Suite passes all four scenarios;
- the three-entry Volume 6 Campaign completes through Chapter 3;
- a live `grok-4.5` browser turn makes separate successful NPC and evaluator
  requests before deterministic commit;
- desktop and mobile Playwright views render the generated scenes without
  overlapping the free-form interaction controls.
