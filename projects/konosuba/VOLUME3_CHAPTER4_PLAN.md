# Volume 3 Chapter 4: Separate the Mask Without Erasing the Host

Status: complete; source, Character, Knowledge, renderer states, eight-node
Roleplay, three endings, four-scenario Quality evidence, browser Playtest, and
the fourth Volume 3 Campaign entry are implemented.

## Source Boundary

- Local source: Volume 3 EPUB, `OEBPS/Text/chapter4-1.xhtml`
- Reviewed extent: all 688 paragraph nodes
- Starts with Sena reporting masked dolls emerging from Keele's dungeon.
- Ends with Kazuma ordering Megumin to act and an explosion sounding outside
  the dungeon.
- The blast's effect on Darkness, Vanir, the mask, and nearby people belongs to
  the next source chapter and must not be asserted here.

## Product Contract

This chapter is an eight-node scene-bound live NPC incident, not a fixed combat
script. Player messages may investigate, negotiate, coordinate, or propose a
response. NPC replies are generated from the current speaker, scene, bounded
Knowledge, relationship, transcript, and node goal. A separate evaluator may
propose score and quoted-evidence changes. Only the deterministic state machine
may move nodes or select the chapter endpoint.

1. `sena_inquiry_without_presumed_guilt` - Sena separates the observed doll
   emergency from suspicion about the party.
2. `mask_doll_behavior_and_circle_disclosure` - Aqua reports the lingering
   purification circle while the group records the dolls' observable behavior.
3. `expedition_roles_and_retreat_signal` - Darkness, Aqua, Megumin, and the
   player define entry, support, evacuation, and retreat responsibilities.
4. `vanir_identity_motive_and_stop_terms` - Vanir states identity, body model,
   admitted doll production, and whether production will stop.
5. `magic_circle_access_without_fact_erasure` - the circle is handled as
   evidence and an access barrier, not secretly erased to manufacture innocence.
6. `possession_dual_voice_and_host_check` - Darkness and Vanir remain separate
   speakers while control, pain, response, and authorization are checked.
7. `surface_layered_exorcism` - the group sequences distance, restraint,
   host-safe exorcism, seal handling, and a mask-removal window.
8. `witnessed_last_resort_order` - Darkness's current instruction, Megumin's
   understood target, Kazuma's responsibility, and Sena's witness record are
   separated before the chapter stops at the blast.

## Deterministic State

Scores:

- `investigation_integrity` (`-8..8`)
- `party_role_clarity` (`-6..6`)
- `host_control_evidence` (`-8..8`)
- `proportional_response` (`-8..8`)

Required quoted evidence:

- `incident_not_presumed_guilt`
- `doll_behavior_observed`
- `circle_disclosed_without_false_causation`
- `entry_and_surface_roles_agreed`
- `retreat_signal_agreed`
- `entry_damage_boundary_agreed`
- `vanir_identity_and_body_stated`
- `doll_production_admitted`
- `stop_and_exit_terms_requested`
- `circle_preserved_as_evidence`
- `host_and_mask_voices_separated`
- `current_body_control_checked`
- `host_authorization_checked`
- `exorcism_resistance_observed`
- `layered_separation_sequence`
- `responsibility_and_witness_recorded`

Endpoints:

- `volume3_witnessed_response_at_dungeon_gate`: all sixteen evidence gates and
  full scores are present; the chapter records the authorized command and blast
  without inventing its result.
- `volume3_containment_without_separation`: the site remains bounded and the
  host is still heard, but a complete separation sequence is not established.
- `volume3_possession_breach_and_unverified_blast`: control, authorization, or
  witness boundaries fail before the turn budget expires.

## Safety And Knowledge Boundaries

- The player cannot declare the party guilty or innocent, invent a summoner, or
  erase the purification circle from history.
- Vanir cannot use prediction to write score, evidence, control, consent,
  injury, separation, or ending state.
- Possession never merges Darkness and Vanir into one authority. Every material
  instruction must preserve who spoke and who controlled the body.
- Darkness's enjoyment of danger is not consent to injury, abandonment,
  humiliation, indefinite possession, or explosive force.
- Aqua's exorcism, Sena's seal, and Megumin's Explosion each have different
  effects and constraints; one cannot be substituted for another.
- No endpoint claims that the final blast killed, separated, purified, or
  spared anyone.

## Verification Targets

- Complete route: 16 turns, 8/8 nodes, 16/16 evidence, exact maximum scores,
  and the witnessed-response endpoint.
- Containment route: all nodes remain reachable with partial evidence and the
  unresolved-containment endpoint.
- Exhaustion route: deterministic breach endpoint.
- Structural attack route: zero forged score/evidence/control/authorization
  and no attacker-selected endpoint.
- Browser Playtest: Vanir and possessed Darkness render distinctly; free input,
  speaker selection, score updates, degraded model recovery, restart, desktop,
  and mobile layouts remain coherent.

## Verification Result

- Provider-free Quality execution passes 4/4 scenarios and reaches every
  endpoint.
- The complete route visits 8/8 nodes, observes 16/16 evidence gates, and
  finishes at exact scores `8/6/8/8`.
- The four-entry Volume 3 Campaign completes in order with every entry visited.
- Browser Playtest renders Vanir in the hidden chamber and the distinct
  possessed-Darkness state in the dual-voice node. Model unavailability is
  exposed only as an authored in-scene recovery, never as an ORT allocation
  error.
