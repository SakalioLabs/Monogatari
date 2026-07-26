# Volume 2 Chapter 4 Dynamic Roleplay Plan

Status: source model and visual foundation complete; live Roleplay pending.

## Source boundary

- Private narrative source: local Volume 2 EPUB,
  `OEBPS/Text/chapter4-1.xhtml`
- Chapter title: `第四章 为美好的店家献上祝福！`
- Runtime dialogue will be newly generated from scene, character, Knowledge,
  relationship, perception, and transcript context. Source prose is not
  runtime output.
- Public appearance research identifies defining visual traits only. Final
  renderer assets are newly generated and fingerprint-imported.

## Adaptation boundary

This chapter is adapted as an adult, consent-centered service and household
trust story. The junior succubus is explicitly an adult employee. The playable
route does not sexualize childlike characters, depict nudity, reproduce the
source's intimate prose, or ask an NPC to impersonate an unwilling real person.

## Dramatic model

The chapter tests whether the player can preserve privacy without using secrecy
to erase another person's agency:

1. A discreet dream service can be mutually beneficial only when every client
   is an adult and agrees to a bounded vitality cost, scope, arrival window,
   stop signal, and cancellation path.
2. A dream contract does not rewrite household reality. Sleep, an attending
   succubus, and agreed dream anchors must be verified before anyone treats an
   encounter as fictional.
3. An occupied sign, clothing basket, lit room, closed door, and direct refusal
   are privacy evidence. Silence, embarrassment, or an assumed dream is not
   consent.
4. A succubus caught by a sacred barrier is not automatically an attacker.
   Intent, contract token, vitality state, and immediate conduct must be
   checked before purification or force.
5. Protecting customers does not require concealing safety failures. The shop
   can disclose procedure and repair controls without publishing identities or
   fantasy details.

## Stable content map

| Kind | IDs |
| --- | --- |
| Planned Roleplay | `volume2_chapter4_roleplay` |
| Characters | `succubus_receptionist`, `succubus_runner` |
| Knowledge | `volume2_succubus_cafe`, `volume2_dream_reality`, `volume2_mansion_privacy` |
| Scenes | `axel_succubus_cafe_alley`, `succubus_cafe_consultation`, `mansion_bath_corridor` |
| Planned endings | `volume2_dream_compact_repaired`, `volume2_private_truce`, `volume2_mansion_trust_broken` |

## Intended live node graph

1. `hearth_work_boundaries`
   - Primary: Aqua
   - Share warmth, work space, debt labor, and household decisions without
     coercion or retaliation.
2. `alley_confidentiality`
   - Primary: Dust
   - Supporting: Keith
   - Distinguish customer privacy from concealment of safety rules or harm.
3. `service_consent_intake`
   - Primary: Succubus receptionist
   - Supporting: Dust, Keith
   - Confirm adult status, dream scope, vitality cap, arrival window,
     cancellation, stop signal, and forbidden real-person impersonation.
4. `household_expectation_reset`
   - Primary: Darkness
   - Supporting: Aqua, Megumin
   - Keep a promised shared meal and define quiet-hours and room boundaries
     without exposing private service details.
5. `privacy_signal_check`
   - Primary: Darkness
   - Check the occupied sign, laundry, light, door, and direct verbal
     confirmation before crossing the bath threshold.
6. `reality_layer_confirmed`
   - Primary: Darkness
   - Establish whether the player is awake, whether a succubus is present, and
     whether the current person has independently consented.
7. `barrier_intercept`
   - Primary: Aqua
   - Supporting: Succubus runner, Megumin
   - Hold purification while checking the runner's contract token, condition,
     intent, and ability to withdraw.
8. `safe_exit_and_review`
   - Primary: Succubus runner
   - Supporting: Aqua, Succubus receptionist, Darkness
   - Release the trapped worker safely, preserve customer identities, disclose
     the control failure, and add barrier-aware dispatch and household consent
     checks.

Every node will require at least two live turns. Fixed Dialogue remains limited
to ending epilogues and bounded recoveries.

## Score and evidence model

Scores:

- `informed_service_consent`: adult status, bounded scope, cost, timing,
  cancellation, stop signal, and no unwilling real-person impersonation.
- `privacy_without_deception`: protect identities and private preferences while
  disclosing procedures, hazards, and failures that affect others.
- `reality_verification`: use observable anchors and direct confirmation before
  treating a person or event as a dream construct.
- `proportional_demon_response`: distinguish species from conduct and use the
  least harmful response consistent with immediate safety.

Evidence:

- `household_resource_terms`
- `adult_service_boundary`
- `dream_contract_terms`
- `household_expectations_kept`
- `privacy_marker_respected`
- `reality_layer_confirmed`
- `runner_intent_verified`
- `safe_exit_and_review`

Only the deterministic Roleplay state machine may accept evidence, apply
clamped score changes, advance nodes, or select an ending.

## Ending contract

- `volume2_dream_compact_repaired`
  - all eight evidence gates
  - minimum score in all four dimensions
  - the runner exits unharmed, the household receives a safety explanation,
    and the shop adopts barrier-aware dispatch without exposing customers
- `volume2_private_truce`
  - immediate harm is avoided and private records remain protected
  - incomplete reality checks or process repair keep trust provisional
- `volume2_mansion_trust_broken`
  - exhaustion, forced purification, ignored privacy markers, assumed dream
    consent, leaked customer details, or concealed safety failures

## Asset foundation

- Snowy Axel side street with an ordinary discreet cafe facade
- Professional dream-service consultation salon with forms and privacy screens
- Mansion bath corridor with visible occupancy cues and extinguished lamp
- Adult succubus receptionist, welcoming and concerned states
- Adult junior succubus courier, anxious and relieved states

## Engine validation targets

- Perception claims such as "this is a dream" cannot mutate canonical scene or
  character identity without deterministic evidence.
- A real supporting NPC keeps her own consent and goal; she cannot inherit a
  fantasy persona from the player or primary NPC.
- Confidential fields never enter another NPC's prompt, evaluator evidence, or
  Campaign summary.
- Clean free-form questions can discover contract and safety facts without
  requiring exact keywords or menu choices.
- NPC generation and evaluator inference remain separate.
- Provider failure yields authored in-world recovery without technical text or
  fabricated score/evidence.
- Prompt, identity, consent, dream-state, memory, score, ending, and encoded
  attacks cannot enter model context literally or mutate state.
