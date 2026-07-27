# Volume 5 Chapter 4 Roleplay Plan

## Source Boundary

- Local source: `F:\下载\美好世界`, Volume 5 Chapter 4.
- Reviewed extraction: `.tmp/volume5-complete/OEBPS/Text/chapter04.xhtml`.
- Source paragraph count: 623.
- Source SHA-256:
  `b1244f2f161a79a578f2de2a53be0e9b5f5e174b0a3f646bdcc39fed4829f163`.
- The project stores derived scene facts and original interactive text, not the
  source prose.

## Runtime Contract

`volume5_chapter4_roleplay` is the playable chapter. It is not a fixed
Dialogue tree. Every clean turn uses:

1. the active scene and its bounded situation;
2. the selected NPC's character, motive, relationship, and pinned Knowledge;
3. a bounded recent transcript;
4. live NPC generation;
5. a separate structured evaluator; and
6. deterministic score, evidence, node, and ending application.

Inference failure leaves the turn uncommitted and retryable. Authored responses
are limited to detected control-intrusion containment and guarded output
recovery; they are not successful live-model evidence.

## Ten Nodes

1. `morning_tour_terms`: agree on an interruptible village tour, watch roles,
   contact roles, and an abort signal while the threat remains active.
2. `village_relics_and_sealed_facility`: separate tourist stories from
   observable seals and establish a no-touch/no-unseal boundary.
3. `clothing_shop_artifact_and_purchase`: keep robe purchase voluntary and
   isolate the unidentified ancient rifle without testing it.
4. `demon_hill_covert_approach`: map covert movement and a warning chain while
   retaining the sealed-weapon target as a hypothesis.
5. `darkness_holds_broken_fence`: verify Darkness's actual capacity, rotation,
   treatment threshold, civilian perimeter, and exit.
6. `sylvia_daylight_withdrawal`: use only the player's real identity and
   verifiable victories, then observe withdrawal without declaring final safety.
7. `evening_defense_and_departure_review`: credit defense without rewarding
   injury and agree on departure conditions, night watch, and family contact.
8. `locked_room_current_boundary`: remove sleep magic and the external lock,
   stop temperature pressure, and restore a current free exit.
9. `mutual_thanks_and_alarm`: accept gratitude without entitlement and let the
   alarm immediately override the private conversation.
10. `night_breach_hostage_and_chimera_reveal`: coordinate minimum-force hostage
    rescue and record Sylvia's chimera identity using her own self-description.

Each node has two quoted evidence gates. Four bounded dimensions
(`fact_verification`, `defense_coordination`, `household_care`, and
`consent_and_agency`) are proposed by an independent evaluator and clamped by
the deterministic state machine.

## Endings

- `volume5_night_breach_contained_with_agency`: all twenty evidence gates and
  exact `9/9/9/9` scores.
- `volume5_night_breach_contained_with_open_risks`: Sylvia's self-description
  is respected, but the minimum-force rescue or earlier record is incomplete.
- `volume5_night_breach_decision_overrun`: the final decision window expires
  without inventing safety, consent, identity, or victory.

Volume 5 Chapter 3 routes into Chapter 4. All three Chapter 4 endings complete
the current four-entry Campaign.

## Visual Assets

Five chapter backgrounds and three transparent character states were generated
for this production simulation and imported through fingerprint-bound asset
plans. No external image bytes were copied.

The Megumin, Darkness, and Sylvia images are derivative character depictions
grounded in public official anime appearance references. They are not claimed
under the repository's MIT license. Redistribution requires an explicit
rights decision independent of engine-code licensing.

## Verification

- MCP transaction planning and fingerprint-bound atomic application.
- Core-runtime project validation.
- Delivery validation for every declared renderer asset.
- Provider-free complete, partial, exhaustion, and structural-intrusion Quality
  scenarios.
- Provider-free Chapter 1 through Chapter 4 Campaign replay.
- Desktop and 390px browser layout checks on the free-form main stage.
- Live API generation remains a separate provider gate and must not be inferred
  from deterministic replay.
