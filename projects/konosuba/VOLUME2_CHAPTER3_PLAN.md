# Volume 2 Chapter 3 Dynamic Roleplay Plan

Status: complete and playable as `volume2_chapter3_roleplay`.

## Source boundary

- Private narrative source: local Volume 2 EPUB,
  `OEBPS/Text/chapter3-1.xhtml`
- Chapter title: `第三章 向幽灵少女伸出爱之手！`
- Runtime dialogue will be newly generated from scene, character, Knowledge,
  relationship, and transcript context. Source prose is not runtime output.
- Public appearance research may identify defining visual traits only. Final
  renderer assets must be newly generated and fingerprint-imported.

## Dramatic model

The chapter repeatedly tests whether the player distinguishes labels from
observed conduct:

1. Wiz is an undead Demon King general, but has not harmed Axel residents.
2. Drain Touch is useful, but demonstration requires informed and revocable
   consent from both the caster and the target.
3. Anna is a harmless mansion-bound child spirit; the migrating hostile
   spirits were displaced by Aqua's oversized cemetery barrier.
4. Purification of symptoms is not enough. The party must remove the cause,
   decline improper reward, disclose responsibility, and repair the harm.

## Stable content map

| Kind | IDs |
| --- | --- |
| Roleplay | `volume2_chapter3_roleplay` |
| Character | `anna_filante` |
| Knowledge | `volume2_wiz_disclosure`, `volume2_anna_mansion`, `volume2_ghost_displacement` |
| Scenes | `wiz_magic_item_shop`, `haunted_mansion_hall`, `haunted_mansion_night`, `anna_garden_grave` |
| Endings | `volume2_anna_home_restored`, `volume2_mansion_temporary_truce`, `volume2_haunting_displaced` |
| Quality Suite | `quality_suites/volume2_chapter3_roleplay.json` |

## Live node graph

1. `shop_nonaggression`
   - Primary: Aqua
   - Supporting: Wiz
   - Establish no attack, no purification, and no unsafe item handling while
     facts are gathered.
2. `wiz_disclosure`
   - Primary: Wiz
   - Supporting: Aqua
   - Separate title, current conduct, barrier role, uncertainty, and immediate
     threat before deciding how to proceed.
3. `drain_touch_consent`
   - Primary: Wiz
   - Supporting: Aqua
   - Define purpose, minimum effect, stop signal, target consent, caster
     consent, and no covert purification.
4. `haunting_root_scope`
   - Primary: Wiz
   - Supporting: Aqua, Darkness
   - Investigate why spirits repopulate cleared houses instead of treating
     repeated purification as proof of completion.
5. `anna_household_boundary`
   - Primary: Anna
   - Supporting: Aqua, Megumin
   - Distinguish Anna from migrating spirits; learn her harmless preferences,
     mansion attachment, and wish to hear adventurers' stories.
6. `midnight_regroup`
   - Primary: Megumin
   - Supporting: Aqua, Darkness
   - Prevent indoor Explosion, preserve a route, regroup without abandoning a
     frightened companion, and identify possessed dolls as a symptom.
7. `cemetery_accountability`
   - Primary: Aqua
   - Supporting: Wiz, Darkness
   - Connect the oversized sacred barrier to displaced spirits, remove it,
     guide spirits onward, and refuse the temporary guild reward.
8. `mansion_repair_terms`
   - Primary: Anna
   - Supporting: Aqua, Wiz, Megumin, Darkness
   - Disclose the cause to the realtor, maintain Anna's grave, share adventure
     stories, and establish safe cohabitation rather than forced purification.

Every node requires at least two live turns. Fixed Dialogue is reserved for
the three ending epilogues.

## Score and evidence model

Scores:

- `evidence_before_force`: verify conduct, threat, source, and uncertainty
  before attack or purification.
- `consent_in_practice`: obtain specific, informed, revocable agreement and
  honor stop signals.
- `supernatural_stewardship`: manage sacred and magical effects by scope,
  location, side effects, and follow-up.
- `repair_accountability`: disclose causation, refuse improper benefit, repair
  displacement, and preserve the harmless resident's home.

Evidence:

- `shop_truce`
- `wiz_status_verified`
- `drain_touch_terms`
- `repopulation_question`
- `anna_distinguished`
- `midnight_regroup_plan`
- `barrier_cause_repaired`
- `anna_home_terms`

Only the deterministic Roleplay state machine may accept evidence, apply
clamped scores, advance nodes, or select an ending.

## Ending contract

- `volume2_anna_home_restored`
  - all eight evidence gates
  - minimum score in all four dimensions
  - barrier cause repaired and Anna explicitly retained as a harmless resident
- `volume2_mansion_temporary_truce`
  - immediate danger contained and Anna not forcibly purified
  - incomplete disclosure or repair keeps the arrangement provisional
- `volume2_haunting_displaced`
  - exhaustion, repeated symptom-only purification, unsafe escalation, or
    failure to repair the displacement cause

## Asset plan

- Wiz's cramped magic item shop, with hazardous merchandise but no characters
- Mansion daylight hall, suitable for investigation and household negotiation
- Mansion corridor at night, with displaced dolls and no visible gore
- Garden grave in morning light, with a small maintained marker for Anna
- Anna neutral/mischievous transparent sprite
- Aqua guilty/accountable transparent sprite
- Megumin frightened-but-composed transparent sprite

## Engine validation targets

- Supporting NPC selection must load each selected character's own profile,
  Knowledge, relationship, and voice.
- Anna must be interactable as a live NPC without being turned into an enemy
  or a fixed exposition line.
- A player can discover the causal model through free-form questions in
  different wording; no keyword-only menu choice may be required.
- NPC generation and evaluator inference remain separate.
- An ORT/provider failure produces scene-authored recovery without technical
  text or fabricated score/evidence.
- Prompt, identity, score, ending, memory, and encoded attacks cannot enter
  model context literally or mutate state.
