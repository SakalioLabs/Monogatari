# Volume 3 Chapter 1: Improper Trial

Status: foundation in progress.

## Product Contract

This chapter is a scene-bound real-time NPC roleplay. The player writes free-form
responses. NPC dialogue is generated from the active scene, the speaking
character's personality and relationships, pinned Knowledge, and the bounded
recent transcript. A separate evaluator may propose score and evidence changes;
only the deterministic Roleplay state machine may accept those changes, advance
nodes, or select an ending.

Authored lines are limited to opening narration, intrusion containment, and
generation/evaluation recovery. They are not the normal conversation path.

## Chapter Boundary

The chapter starts when Sena states that the randomly teleported coronatite
destroyed Lord Alderp's mansion without casualties. It covers lawful custody,
the truth-bell interrogation, the trial, the corrupted sentence, Darkness's
voluntary request for a stay, the release conditions, and the restitution
business proposed in Wiz's shop.

Characters know only facts available at the current node. Aqua may report sensing
an evil force and observers may record abrupt inconsistencies, but nobody may
identify a hidden source or mechanism. Darkness alone decides whether and when to
reveal her family crest or accept a future obligation.

## Dynamic Graph

| Order | Node | Primary NPC | Scene |
|---|---|---|---|
| 1 | `guild_charge_response` | Sena | `axel_guild_arrest` |
| 2 | `custody_and_escape_boundary` | Sena | `axel_police_cell` |
| 3 | `truth_bell_interrogation` | Sena | `axel_interrogation_room` |
| 4 | `demon_association_boundary` | Sena | `axel_interrogation_room` |
| 5 | `trial_evidence_separation` | Sena | `axel_courtroom` |
| 6 | `corruption_signal_review` | Aqua | `axel_courtroom` |
| 7 | `darkness_voluntary_stay` | Darkness | `axel_courtroom` |
| 8 | `release_terms` | Sena | `axel_courtroom` |
| 9 | `wiz_restitution_business` | Wiz | `wiz_magic_item_shop` |

## State Model

Score dimensions:

- `evidence_integrity`
- `lawful_self_defense`
- `coercion_detection`
- `restitution_accountability`

Evidence records must distinguish allegation from verdict, association from
allegiance, sensed anomaly from proved cause, and a stay from acquittal.
Attacked turns receive zero score and zero evidence. Model failures use bounded
in-world recovery and conservative deterministic fallback evaluation.

Planned endings:

- `volume3_evidence_bound_stay`
- `volume3_stay_without_restitution`
- `volume3_corrupted_sentence`

All endings preserve the same historical chapter boundary while carrying
different evidence, relationship, and restitution state into the next chapter.

