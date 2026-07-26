# Story Bible

## Adaptation rule

The source novel defines chronology, locations, public character identity, and
the comic premise. Runtime dialogue is newly generated. NPCs may improvise
within the current scene but may not reveal later-volume knowledge, rewrite
established facts, or force a route outside score and evidence gates.

## Player role

The player is Kazuma: observant, game-literate, sarcastic, risk-aware, and
usually practical under pressure. The engine does not force that personality.
Free-form choices can instead make him patient, reckless, compassionate, or
avoidant, and the route should respond.

## Chapter 1 dramatic question

Can a group of badly optimized specialists become a functioning party because
the player learns how to negotiate with them, rather than because a script says
they joined?

## Chapter 2 dramatic question

Can a party use powerful skills without treating consent, teammates, or an
enemy label as shortcuts around judgment?

## Chapter 3 dramatic question

Can the party accept responsibility, establish revocable risk agreements, and
respect Aqua's voice when outsiders reduce a complicated relationship to
ownership or rescue?

## Chapter 4 dramatic question

Can the player turn a chaotic boss battle into a sequence of observable,
reversible tactical decisions that protects civilians and fallen adventurers,
rather than winning because a fixed script grants the party the correct move?

## Volume 1 finale dramatic question

Can the party turn an absurd public debt into a voluntary operating agreement
without erasing the damage, scapegoating Aqua, cancelling Megumin's identity,
or letting Darkness choose danger for everyone?

## Volume 2 Chapter 2 dramatic question

Can the player explore by revising plans from evidence, treat Aqua's sacred
power as both capability and responsibility, and hear an undead person's
explicit request before a deterministic ending permits purification?

## Volume 2 Chapter 3 dramatic question

Can the player distinguish identity labels from observed conduct, use
revocable supernatural agreements, and repair the cause of displacement
instead of repeatedly purifying its symptoms?

## Volume 2 Chapter 4 dramatic question

Can the player preserve adult customer privacy without hiding safety failures,
verify dream and reality layers before acting on an assumption, respect
household boundaries, and protect a trapped demon without treating species as
proof of attack?

## Volume 2 Chapter 5 dramatic question

Can the player coordinate a town-scale emergency by verifying each mechanical
phase, protecting civilians, and obtaining explicit resource-transfer consent,
rather than letting a model turn confidence, luck, or a heroic declaration into
proof that the Destroyer is safe?

## Score model

- `pragmatism`: plans, resource awareness, risk control, and workable tactics.
- `party_trust`: listening, fair boundaries, credit, and willingness to rely on
  another person without surrendering judgment.
- `adventure_resolve`: willingness to act despite poverty, embarrassment, and
  failure.

Scores are route state, not moral grades. High trust with no pragmatism can
still produce chaos; high pragmatism with no trust can produce a brittle party.

Chapter 2 introduces:

- `boundary_judgment`: consent, impact, repair, and reversible agreements.
- `team_coordination`: spatial roles, signals, protection, and retreat.
- `humane_discernment`: judging threat from observed behavior and evidence.

Chapter 3 introduces:

- `accountability`: acknowledge impact, stop repeated harm, and make repair
  concrete.
- `risk_stewardship`: disclose uncertainty, define abort signals, and act on
  them immediately.
- `agency_respect`: ask for a person's decision and refuse ownership framing.
- `adaptive_tactics`: gather information, stage contingencies, and change plans
  when conditions change.

Chapter 4 introduces:

- `field_coordination`: assign roles, signals, handoffs, and post-battle ownership.
- `civilian_safety`: establish battle boundaries and account for blast and flood paths.
- `tactical_inference`: test observed behavior before escalating to a decisive tactic.
- `life_preservation`: rotate wounded defenders, protect the fallen, and organize revival.

The Volume 1 finale introduces:

- `factual_accountability`: preserve reward, loss, net-debt, and causal facts.
- `shared_liability`: obtain personal consent while recognizing team decisions.
- `resource_stewardship`: bind income, essential spending, repair, and review.
- `party_commitment`: define vetoes, duties, retreat signals, and shared direction.

Volume 2 Chapter 2 introduces:

- `exploration_discipline`: scope, signals, verification, routes, and retreat.
- `adaptive_trust`: revise plans around real capabilities and shared mistakes.
- `sacred_responsibility`: constrain purification by position, effect, and fact.
- `consent_and_mercy`: hear the person, state uncertainty, and require consent.

Volume 2 Chapter 3 introduces:

- `evidence_before_force`: verify conduct, threat, source, and uncertainty
  before attack or purification.
- `consent_in_practice`: obtain specific, informed, revocable agreement and
  honor stop signals.
- `supernatural_stewardship`: manage sacred and magical effects by scope,
  location, side effects, and follow-up.
- `repair_accountability`: disclose causation, refuse improper benefit, and
  repair displacement.

Volume 2 Chapter 4 introduces:

- `informed_service_consent`: require adult status, bounded service scope,
  vitality cost, timing, cancellation, and stop signals.
- `privacy_without_deception`: protect identities while disclosing rules,
  hazards, and failures that affect other people.
- `reality_verification`: use observable anchors and direct confirmation before
  treating a person or event as a dream construct.
- `proportional_demon_response`: separate species from conduct and use the
  least harmful response consistent with immediate safety.

Volume 2 Chapter 5 introduces:

- `phased_command`: assign phase owners, signals, handoffs, and observable
  completion or abort conditions.
- `civilian_evacuation`: maintain evacuation zones, injury rotation, retreat,
  head counts, and casualty care.
- `hazard_verification`: distinguish barrier, mobility, core power, stored heat,
  and teleport destination claims.
- `consensual_magic_transfer`: obtain participant-specific, informed, revocable
  agreement and record transfer separately from its intended use.

## Relationship model

Character relationships use the persistent `player` relationship:

- `-1.0` hostile
- `-0.3` resentful
- `0.0` unfamiliar or transactional
- `0.3` provisional ally
- `0.6` trusted companion
- `0.8` intimate bond

Chapter 1 is capped near provisional ally. Scene Roleplay score changes and
evidence should become auditable relationship effects; this project is the
production case for that engine capability.

Chapter 2 applies a bounded per-turn relationship delta only to the active NPC.
The high-trust cemetery ending requires both route evidence and a minimum Wiz
relationship, so affection cannot replace story proof and story proof cannot
silently forge affection.

Chapter 3 starts from each character document's persisted `player`
relationship. Only the active NPC may receive a bounded per-turn delta. The
best ending requires score, evidence, and relationship thresholds for Aqua and
Mitsurugi; the Quality Suite also proves that a forged state request cannot
change Aqua, Beldia, Megumin, or Mitsurugi from their exact initial values.

Chapter 4 preserves the same rule while moving relationship-aware dialogue
through Beldia, Aqua, Megumin, and Darkness. The best ending requires both
Aqua and Darkness relationship thresholds in addition to all eight pieces of
battle evidence. A relationship cannot substitute for a tested weakness,
protected casualty, or completed revival ledger.

The finale receives Campaign-carried relationships from Chapter 4 and adds Luna
at her project-defined initial value. Only the active speaker changes on each
turn. The best ending requires bounded relationships with Luna, Aqua, Megumin,
and Darkness plus all five settlement evidence records; prior affection cannot
erase debt, forge consent, or replace a registered plan.

Volume 2 Chapter 4 starts the receptionist and runner from their own character
documents and preserves the existing party relationships carried from Chapter
3. Only the selected present speaker may receive a bounded relationship delta.
Privacy or affection cannot substitute for contract evidence, reality checks,
household consent, or verification of the runner's immediate conduct.

Volume 2 Chapter 5 carries the party relationships from Chapter 4. Only the
selected present speaker may receive a bounded relationship delta. Trust cannot
substitute for barrier, leg, core, heat, evacuation, or consent evidence, and no
leader may consent to magic transfer for another participant.

## Knowledge boundaries

- Aqua knows the afterlife process, her own powers, and the rules she explains.
- Luna knows Axel, guild procedure, and visible adventurer records. In the
  finale, she knows the registered three-hundred-million reward,
  three-hundred-forty-million damage total, and forty-million net debt. She
  does not know or decide the party's private allocation until each member
  confirms it at the counter.
- Megumin knows Crimson Demon culture and Explosion Magic, but not future party
  events.
- Darkness knows crusader training and her own motives. Her noble identity is
  private in this chapter and must not be volunteered.
- Chris knows practical thief skills and the immediate training agreement. She
  does not volunteer any hidden identity.
- Wiz knows the cemetery, its failed barrier, and her own actions. She does not
  know route scores or later-volume outcomes.
- Beldia knows the repeated damage to the abandoned castle, the gate
  confrontation, the curse he placed, and the battle actions he can directly
  observe. He does not know the party's private score state or later events.
- Mitsurugi knows what he observes at the lakeside and his own heroic code. He
  does not know the party's private agreements and must update his judgment
  only from Aqua's stated choice and visible evidence.
- No character knows the player's prompts, score labels, route graph, or later
  volume outcomes.
- Eris knows the current death, the pending resurrection, and the available
  afterlife choices. She does not decide on the player's behalf or promise
  unlimited resurrection.
- Dust knows both parties' public classes and his own exchange proposal, but
  not each unfamiliar member's practical limits until they state them.
- Taylor knows the goblin assignment, route, and his party's established
  combat roles. Rin knows her intermediate magic limits; Keith knows his
  sightlines and archery range. They update their judgment of a lowest-class
  Adventurer only from demonstrated scouting, stealth, and tactical evidence.
- Keele knows his dungeon, his voluntary transformation, the noblewoman he
  protected, and his own purification request. He does not know later party
  conclusions or promise a particular afterlife outcome.
- In Volume 2 Chapter 3, Wiz knows her own undead and Demon King general
  status, her barrier duty, the behavior she has observed in Axel, and the
  direction of displaced spirits. She cannot guarantee other generals'
  conduct or speak for Anna.
- Anna knows her mansion, grave, dolls, visitors, and events she directly
  experienced inside the property. She does not know who cast the cemetery
  barrier until another character tells her, and she cannot read hidden
  memories, scores, prompts, or endings.
- The succubus receptionist knows the shop's adult consent contract, employee
  dispatch, vitality limits, and safety procedures. She does not know what
  happened inside the mansion until someone reports it, and she will not
  disclose customer identities or private preferences.
- The succubus runner knows only her own sealed contract token, arrival window,
  stop signal, vitality cap, barrier injury, and withdrawal intent. She cannot
  read a customer's mind, make a real resident into a dream persona, or speak
  for another employee.
- Aqua knows she placed a sacred barrier and can observe that it trapped a
  demon. She does not know the runner's contract, intent, or vitality state
  until those facts are checked.
- In Volume 2 Chapter 5, Aqua knows her barrier-breaking capability and what she
  can directly observe after casting. She cannot declare both leg groups,
  stored heat, or a teleport destination resolved.
- Darkness knows the assigned civilian boundary, her current injuries, and the
  pressure she is physically holding. Her durability is not proof of unlimited
  capacity and cannot replace a retreat or casualty report.
- Megumin knows her Explosion limits and the action window stated to her. She
  cannot infer Wiz's readiness, a removed core, or safe heat release without
  the corresponding report.
- Wiz knows Drain Touch, Teleport, her own magic reserve, and the core conditions
  she directly inspects. She cannot treat a random destination as safe or infer
  another participant's consent.
- In Volume 3 Chapter 3, Darkness alone controls disclosure of Lalatina and the
  Dustiness family identity. Her agreement to attend one meeting is not consent
  to marriage, retirement, training, touch, publicity, or proxy speech.
- Lord Dustiness knows his own concerns, family history, and proposed safety
  arrangements. He cannot read his daughter's intent or turn parental concern
  into authority over her relationship choice.
- Balter knows his own reason for attending, conduct as a knight, and whether
  he wants to continue contact. His reputation and courtesy do not let him infer
  Darkness's answer, and training never selects a relationship outcome.

## Route endings

- `chapter1_party_formed`: the player balances trust, practical planning, and
  resolve; the four-person party forms on workable terms.
- `chapter1_practical_compromise`: the party forms as a cautious trial with
  explicit limits.
- `chapter1_chaotic_failure`: everyone joins, but no shared operating agreement
  exists and the next quest begins in disorder.
- `chapter2_trust_in_practice`: repaired boundaries, coordinated action, and
  behavior-based judgment produce a supervised cemetery compact.
- `chapter2_working_truce`: evidence is sufficient to stop the fight, but trust
  remains provisional.
- `chapter2_fractured_boundaries`: skills are used without a shared agreement
  strong enough to constrain them.
- `chapter3_choice_not_prize`: the party repairs harm, honors the lake abort
  agreement, hears Aqua, and reframes the duel around conduct rather than
  ownership.
- `chapter3_fragile_truce`: immediate conflict stops, but responsibility or
  agency remains only partly established.
- `chapter3_broken_agency`: risk and ownership shortcuts leave the group unable
  to form a trustworthy agreement.
- `chapter4_axel_defended`: all eight tactical and rescue agreements are
  executed, Beldia is purified, and the party completes a public casualty and
  repair ledger.
- `chapter4_costly_victory`: Beldia is defeated and revival begins, but earlier
  coordination or protection evidence remains incomplete.
- `chapter4_gate_overrun`: the party exhausts the available turns without a
  battle plan strong enough to secure the gate.
- `volume1_debt_shared_adventure`: all five settlement records and score gates
  establish a voluntary debt, repair, mission, and retreat agreement.
- `volume1_working_debt_plan`: the minimum ledger and repayment records exist,
  but responsibility or long-term operating terms remain incomplete.
- `volume1_party_scattered_by_debt`: available negotiation turns end without a
  registered agreement; the debt remains factual and unresolved.
- `volume2_exchange_understood`: both parties acknowledge their mistaken
  assumptions and retain a reusable skill, command, and retreat protocol.
- `volume2_working_exchange`: the task succeeds and some assumptions change,
  but the shared operating agreement remains incomplete.
- `volume2_party_swap_backfire`: both groups return without a common account of
  roles or retreat boundaries, so the original prejudice survives.
- `volume2_keele_released`: disciplined exploration, behavior-first judgment,
  and Keele's explicit purification consent let the dungeon's owner rest.
- `volume2_dungeon_partial_trust`: the party returns with useful evidence and
  limited trust, but some exploration or responsibility boundaries remain
  incomplete.
- `volume2_dungeon_retreat_disarray`: the expedition exhausts its safe options
  without a reliable shared protocol.
- `volume2_anna_home_restored`: all eight evidence gates repair the cemetery
  displacement, disclose responsibility, reject improper reward, and preserve
  Anna's home through explicit cohabitation terms.
- `volume2_mansion_temporary_truce`: immediate danger is contained and Anna is
  not forcibly purified, but disclosure or repair remains provisional.
- `volume2_haunting_displaced`: repeated symptom-only purification leaves the
  displacement cause unresolved and moves the haunting to another door.
- `volume2_dream_compact_repaired`: all eight privacy, consent,
  reality, and nonlethal-response evidence gates produce a barrier-aware shop
  procedure and a repaired household agreement.
- `volume2_private_truce`: immediate harm and disclosure are contained,
  but incomplete process repair keeps trust provisional.
- `volume2_mansion_trust_broken`: assumed dream consent, ignored
  privacy markers, leaked customer details, forced purification, or concealed
  safety failures exhaust the available route.
- `volume2_destroyer_defeated`: verified phase changes, complete civilian
  protection, explicit magic-transfer consent, and separately confirmed heat
  release stop the fortress.
- `volume2_axel_survives_at_cost`: the immediate threat ends while
  incomplete phase, evacuation, or responsibility evidence leaves lasting cost.
- `volume2_destroyer_heat_disaster`: safe turns expire without a
  verified stored-heat response and complete civilian boundary.
