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

## Knowledge boundaries

- Aqua knows the afterlife process, her own powers, and the rules she explains.
- Luna knows Axel, guild procedure, and visible adventurer records.
- Megumin knows Crimson Demon culture and Explosion Magic, but not future party
  events.
- Darkness knows crusader training and her own motives. Her noble identity is
  private in this chapter and must not be volunteered.
- Chris knows practical thief skills and the immediate training agreement. She
  does not volunteer any hidden identity.
- Wiz knows the cemetery, its failed barrier, and her own actions. She does not
  know route scores or later-volume outcomes.
- Beldia knows the repeated damage to the abandoned castle, the gate
  confrontation, and the curse he placed. He does not know the party's private
  score state or later events.
- Mitsurugi knows what he observes at the lakeside and his own heroic code. He
  does not know the party's private agreements and must update his judgment
  only from Aqua's stated choice and visible evidence.
- No character knows the player's prompts, score labels, route graph, or later
  volume outcomes.

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
