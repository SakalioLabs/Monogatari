# Volume 5 Chapter 5 Roleplay Plan

## Source Boundary

- Local source: `F:\下载\美好世界`, Volume 5 Chapter 5.
- Reviewed extraction: `.tmp/volume5-complete/OEBPS/Text/chapter05.xhtml`.
- Source paragraph count: 738.
- Source SHA-256:
  `bc394a9b31282b83d5e0aeff2198c4b3a1c967f0539a2bc72f32144b4a3b1075`.
- The project stores derived chronology, scene facts, and newly authored
  interactive contracts. It does not copy source prose into runtime content.

## Dramatic Question

Can the player contain a dangerous inherited weapon system by separating
observation from legend, disclosing their own contribution to the breach,
coordinating evacuation and bounded power use, and preserving each character's
agency instead of relying on a scripted victory?

## Runtime Contract

`volume5_chapter5_roleplay` is the playable chapter. Every clean turn uses the
active scene, selected NPC identity and current motive, pinned Knowledge,
bounded transcript, live NPC generation, a separate structured evaluator, and
deterministic score/evidence/route application. NPC prose and evaluator output
cannot select a node or ending.

Provider failure leaves the turn uncommitted and retryable. Detected control
attacks skip both model calls, receive zero score/evidence, and become bounded
in-world uncertainty. Provider-free Quality replay proves authored rules, not
live model availability.

## Ten Nodes

1. `sealed_arsenal_password_boundary`: identify the non-magical access
   mechanism and the claimed weapon objective without treating coercion,
   familiarity, or a readable label as authorization to open the arsenal.
2. `containment_before_confinement`: after Sylvia enters the arsenal, reject
   the assumption that an unknown interior is a harmless prison; establish
   evacuation, observation, and reopening contingencies.
3. `magician_killer_emergence`: record Sylvia's fusion, strong magic
   resistance, fire projection, changing mobility, and remaining uncertainty
   before choosing a response.
4. `demon_hill_evacuation_accountability`: disclose the player's role in the
   breach, account for civilians and homes, and compare abandonment with a
   bounded recovery attempt.
5. `decoy_teleport_and_mana_rotation`: assign attack, teleport, rescue,
   distance, and mana-stop roles without treating any resident as expendable.
6. `arsenal_log_and_countermeasure_provenance`: distinguish the researcher's
   notes, observed machine behavior, and unresolved claims; identify the
   countermeasure's charge, durability, and misuse risks.
7. `clothing_shop_countermeasure_recovery`: recover the long device with
   shared lifting, muzzle discipline, a clear route, and no blind trigger test.
8. `yunyun_decoy_and_charge_diagnosis`: preserve Yunyun's voluntary role and
   concealed teleport handoff, then interpret the failed first charge as
   insufficient evidence rather than proof of a broken or safe weapon.
9. `megumin_charge_and_controlled_fire`: clear the blast and backstop area,
   protect Komekko from the trigger, obtain explicit role agreement, charge
   with Explosion, and fire only after the device reports readiness.
10. `reconstruction_record_and_magic_agency`: audit casualties, damage,
    salvage, and rebuilding; separate Arue's fiction from threat evidence; and
    let Megumin decide her future magic development without debt, romance, or
    family expectation substituting for current choice.

Each node has two quoted evidence gates. The independent evaluator proposes
bounded changes to:

- `artifact_verification`: provenance, observed behavior, uncertainty, and
  safe interpretation of inherited technology.
- `civilian_coordination`: evacuation, rescue, mana rotation, blast boundaries,
  and reconstruction records.
- `accountable_power`: disclosure, proportional force, charge/fire controls,
  backstop protection, and damage ownership.
- `character_agency`: voluntary roles, stop conditions, child safety, and
  Megumin's current control over her own magic path.

## Endings

- `volume5_legacy_contained_with_shared_agency`: all twenty evidence gates and
  exact `9/9/9/9` scores.
- `volume5_legacy_contained_with_open_damage`: the immediate weapon crisis is
  contained, but accountability, reconstruction, or agency evidence remains
  incomplete.
- `volume5_legacy_decision_overrun`: the final decision window expires without
  inventing safety, consent, provenance, or victory.

All Chapter 4 endings route into Chapter 5. All three Chapter 5 endings
complete the five-entry Campaign until the epilogue is authored.

## Visual Assets

- sealed arsenal entrance at night;
- burning village observed from Demon Hill;
- underground legacy-device warehouse;
- clothing-shop yard and long countermeasure at dawn;
- controlled final firing lane at dawn;
- rapid village reconstruction;
- reviewed existing Sylvia, Yunyun, Megumin, and Komekko presentation states.

The six generated backgrounds were fingerprint-imported and remain separate
from public appearance references. Their redistribution rights require an
explicit decision outside the engine license.

## Required Acceptance

- MCP transaction planning and exact fingerprint-bound application;
- core-runtime and delivery validation;
- complete, partial, exhaustion, and structural-attack Quality scenarios;
- provider-free Chapter 1 through Chapter 5 Campaign replay;
- full node, evidence, and ending coverage;
- desktop and mobile free-form main-stage Playtest;
- separate live NPC/evaluator evidence when the configured provider is ready.
