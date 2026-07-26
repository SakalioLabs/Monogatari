# Volume 3 Chapter 3: Let the Noble Daughter Choose

Status: source reviewed; dynamic Roleplay contract established; provider-backed
generation evidence awaits runtime credential injection.

## Product Contract

This chapter is an eight-node, scene-bound, real-time NPC Roleplay. The player
may question, reassure, challenge, or negotiate with the present characters in
free-form language. Each selected speaker generates a reply from that
character's personality, relationships, pinned Knowledge, scene facts, and
bounded transcript. A separate evaluator may propose score and evidence
observations. Only the deterministic Roleplay state machine may advance nodes
or select an ending.

The objective is not to force a successful marriage or to sabotage one. The
player must discover what Darkness, Lord Dustiness, and Balter each want,
separate parental concern from legal or moral authority, and help them reach a
choice that every directly affected person can state and withdraw.

## Source Boundary

The chapter begins when Darkness returns after negotiating the consequences of
the trial. She discloses that she is Dustiness Ford Lalatina and that a meeting
with Alderp's son Balter is scheduled for the same day. Her father sincerely
wants her to leave dangerous adventuring work. Balter is widely regarded as a
capable and considerate knight; he was also sent by his father and initially
intended to refuse the match.

The adaptation preserves the identity disclosure, conflicting family goals,
Balter's independent position, the optional training-yard comparison, and
Sena's arrival as the next-chapter hook. It does not make sexual humiliation,
forced undressing, painful restraint, deceptive pregnancy claims, threats,
physical punishment, or reputational sabotage into rewarding player actions.
Darkness's eccentric fantasies may appear only as character texture. They
never substitute for present-tense consent or justify harm.

## Dynamic Graph

| Order | Node | Primary NPC | Scene |
|---|---|---|---|
| 1 | `return_and_identity_disclosure` | Darkness | `axel_mansion_winter_parlor` |
| 2 | `private_intent_before_intervention` | Darkness | `axel_mansion_winter_parlor` |
| 3 | `father_welfare_and_future` | Lord Dustiness | `dustiness_manor_reception` |
| 4 | `balter_independent_intent` | Balter | `dustiness_manor_reception` |
| 5 | `joint_terms_without_proxy` | Darkness | `dustiness_manor_winter_garden` |
| 6 | `training_consent_and_stop_rules` | Balter | `dustiness_manor_training_yard` |
| 7 | `decision_and_reputation_record` | Darkness | `dustiness_manor_reception` |
| 8 | `party_accountability_and_sena_notice` | Aqua | `axel_mansion_winter_parlor` |

Each node requires two distinct clean turns. The first establishes the present
speaker's position or a concrete boundary. The second tests whether the player
can turn that statement into a mutually checkable next action without speaking
for an absent person.

## State Model

Score dimensions:

- `self_determination` (maximum 10)
- `mutual_disclosure` (maximum 8)
- `family_trust` (maximum 8)
- `noncoercive_resolution` (maximum 6)

Evidence must distinguish a person's own statement from hearsay, concern from
authority, willingness to meet from agreement to marry, a training invitation
from consent to unrestricted force, and a private choice from a public story.
Attacked turns receive zero score and evidence. Model failures use bounded
in-world recovery plus conservative fallback evaluation.

Relationship deltas apply only to the NPC who spoke on that turn. Darkness's
father cannot alter Darkness's relationship state, Balter cannot choose an
ending, and no generated reply may award points or declare a route.

## Endings

- `volume3_chosen_path_and_open_door`: all parties state their own positions;
  Darkness keeps control of her future, her father receives a bounded safety
  plan, and Balter may leave or remain in contact without a coerced promise.
- `volume3_respectful_refusal_and_family_trust`: the meeting ends without a
  match, but privacy, reputation, and an actionable family check-in survive.
- `volume3_public_sabotage_and_broken_trust`: proxy decisions, humiliation, or
  unbounded conflict leave the meeting and party trust damaged.

Every ending returns to the Axel mansion and preserves Sena's announced visit
as the next chapter's unresolved fact.

## Acceptance

- Provider-free preview must visit 8/8 nodes, commit every required evidence
  ID exactly once, and reach each authored ending in a dedicated scenario.
- The adversarial scenario must guard every control attempt, leave all scores
  and evidence unchanged on attacked turns, and report zero unguarded attacks.
- Browser Playtest must accept free-form input, render all four scenes and both
  new NPCs, complete at desktop and mobile sizes, and expose no inference
  implementation errors to the player.
- Live provider generation and evaluator evidence remain separate from
  deterministic replay and authored inference-failure recovery.
