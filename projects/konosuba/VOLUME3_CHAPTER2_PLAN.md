# Volume 3 Chapter 2: A Friend for the Crimson Demon Girl

Status: live Roleplay implemented, browser-played, and release-verified;
provider-backed generation evidence awaits runtime credential injection.

## Product Contract

This chapter is an eight-node scene-bound real-time NPC Roleplay. Free-form
player input drives generated replies from the selected scene participant's
own personality, relationships, Knowledge, and bounded transcript. A separate
evaluator may propose score/evidence observations. Only the deterministic
Roleplay state machine may advance nodes or select endings.

The adaptation does not turn the source's bath sequence, body comparisons, or
rumours about minors into playable sexual content. It keeps only the relevant
privacy rule: private household information cannot be used to embarrass a
young person or manipulate a rivalry.

## Chapter Boundary

The chapter starts while Darkness remains absent after the trial stay. Megumin
asks to keep Chomusuke in the mansion; the cat briefly emits fire, but no
character may infer its future identity. Sena then links repeated Explosion
practice to giant toads leaving hibernation. The party must contain the public
consequence, meet Yunyun in the snowfield, and negotiate rivalry without
humiliation or coerced stakes.

The second half covers Yunyun's difficulty joining ordinary town activities,
the adamantite challenge stall, an invitation she may accept or decline, and a
bounded welfare-check plan when Darkness still has not returned.

## Dynamic Graph

| Order | Node | Primary NPC | Scene |
|---|---|---|---|
| 1 | `chomusuke_household_boundary` | Megumin | `axel_mansion_winter_hearth` |
| 2 | `explosion_consequence_review` | Sena | `axel_mansion_winter_hearth` |
| 3 | `snowfield_rescue_coordination` | Aqua | `axel_snowy_toad_field` |
| 4 | `yunyun_identity_and_rivalry` | Yunyun | `axel_snowy_toad_field` |
| 5 | `challenge_consent_and_stakes` | Megumin | `axel_snowy_toad_field` |
| 6 | `winter_market_safety` | Megumin | `axel_winter_market` |
| 7 | `open_invitation_without_pressure` | Yunyun | `axel_winter_market` |
| 8 | `privacy_and_darkness_welfare` | Aqua | `axel_mansion_winter_hearth` |

## State Model

Score dimensions:

- `accountable_magic`
- `rivalry_consent`
- `social_attunement`
- `privacy_and_welfare`

Evidence must separate observation from hidden identity, correlation from
proved cause, rescue from humiliation, challenge consent from coerced stakes,
and an open invitation from forced friendship. Attacked turns receive zero
score and evidence. Model failures use bounded in-world recovery plus
conservative fallback evaluation.

Planned endings:

- `volume3_open_invitation_and_welfare_plan`
- `volume3_rivalry_with_boundaries`
- `volume3_frog_fallout_and_isolation`

Every ending preserves Darkness's unresolved absence as the next chapter's
starting fact while carrying different public-responsibility and relationship
state.
