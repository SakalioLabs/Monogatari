# Volume 3 Finale: Public Record, Private Aftermath

Status: source reviewed; Knowledge, renderer-state, and dynamic Roleplay
contract established; complete Roleplay and Quality evidence are the next
implementation batch.

## Source Boundary

- Local source: Volume 3 EPUB, `OEBPS/Text/chapter5-1.xhtml`
- Reviewed extent: all 178 paragraph nodes
- Starts after the Vanir battle, with the party summoned to Axel's guild.
- Reveals that Megumin's Explosion destroyed Vanir's first life, Darkness was
  critically injured, Aqua healed her, and the party's spy suspicion was
  cleared.
- Ends after Vanir appears alive as the shop's new clerk, explains the numeral
  II on his mask, and offers a prophecy-linked business proposal.
- The prophecy is not a verified future event and cannot select a later route.

## Product Contract

This finale is a six-node scene-bound live NPC settlement, not a fixed
ceremony script. Player messages may request corrections, protect private
boundaries, reconcile the reward ledger, prepare a difficult report, verify
Vanir's current status, or negotiate business terms. NPC replies are generated
from the current speaker, scene, bounded Knowledge, relationship, transcript,
and node goal. A separate evaluator may propose score and quoted-evidence
changes. Only the deterministic state machine may move nodes or select the
final endpoint.

1. `guild_exoneration_and_public_apology` - Sena distinguishes cleared spy
   suspicion, the Destroyer reward, and the later Vanir response.
2. `darkness_recovery_armor_and_name_boundary` - Darkness's recovery and armor
   replacement are recorded without turning her disclosed name into unlimited
   permission for public teasing.
3. `debt_and_reward_ledger_closed` - Sena itemizes debt, mansion repair, and
   the remaining forty-million-erish reward without rewriting earlier costs.
4. `wiz_report_preparation` - Darkness and the player separate observed battle
   facts from assumptions about Wiz's grief or responsibility.
5. `vanir_second_life_and_reunion` - Wiz and Vanir identify the first-life
   destruction, mask numeral II, restored clay body, and their old friendship.
6. `retired_general_business_boundary` - Vanir's claim that one death ended
   his barrier duty and his future prophecy remain separate from current,
   reviewable shop terms.

## Deterministic State

Scores:

- `public_record_integrity` (`-6..6`)
- `privacy_aftercare` (`-6..6`)
- `aftermath_accountability` (`-6..6`)
- `future_claim_discipline` (`-6..6`)

Required quoted evidence:

- `spy_suspicion_cleared_without_rewriting_history`
- `battle_and_reward_records_separated`
- `darkness_recovery_and_armor_recorded`
- `name_disclosure_not_blanket_teasing_consent`
- `debt_repair_and_reward_itemized`
- `remaining_reward_confirmed`
- `wiz_report_uses_observed_facts`
- `darkness_owns_her_report`
- `vanir_first_life_destroyed`
- `mask_numeral_two_observed`
- `friendship_and_reunion_separated_from_excuse`
- `barrier_duty_status_stated_as_claim`
- `prophecy_not_recorded_as_fact`
- `business_terms_require_present_consent`

Planned endpoints:

- `volume3_public_record_and_open_shop_terms`: every record and boundary is
  established while future business remains optional.
- `volume3_reward_settled_with_unresolved_shop_risk`: the public ledger closes,
  but Vanir's current role or proposal remains insufficiently bounded.
- `volume3_aftermath_record_overrun`: privacy, result, or future-claim
  boundaries expire before a reliable final record is produced.

## Safety And Knowledge Boundaries

- The fifth chapter may reveal the prior blast result; the fourth chapter may
  not retroactively claim it.
- Sena can certify exoneration, apology, armor replacement, debt deductions,
  and reward. She cannot certify Vanir's current metaphysical status.
- Darkness's public name disclosure does not grant unlimited teasing,
  humiliation, romantic interpretation, or proxy speech.
- Wiz may state her friendship and present relief. No one may prewrite her
  grief, forgiveness, or commercial consent.
- Vanir may state that the first life was destroyed and point to the numeral II
  on the mask. His claims about retirement, harmlessness, prophecy, and future
  trials require independent or future verification.
- A business proposal is optional and reviewable; prophecy cannot coerce
  acceptance or write a later ending.

## Verification Targets

- Complete route: 12 turns, 6/6 nodes, 14/14 evidence, exact maximum scores,
  and the open-shop-terms endpoint.
- Settlement route: all nodes remain reachable with bounded partial evidence
  and the unresolved-shop-risk endpoint.
- Exhaustion route: deterministic overrun endpoint.
- Structural attack route: zero forged score/evidence/reward/consent/future
  result and no attacker-selected endpoint.
- Browser Playtest: Sena, publicly flustered Darkness, reunited Wiz, and
  second-life Vanir render distinctly; free input, participant selection,
  score updates, degraded model recovery, restart, desktop, and mobile layouts
  remain coherent.
