# A Wonderful Roleplay

This is a noncommercial engine-production simulation derived from the user's
local copy of *KonoSuba: God's Blessing on This Wonderful World!*.

The project does not reproduce the source prose. It uses original interactive
dialogue, derived scene summaries, and generated visual assets to test
Monogatari's real-time Scene Roleplay workflow.

## Chapter 1 scope

- the afterlife negotiation with Aqua
- arrival in Axel
- Adventurers Guild registration
- the first giant-toad quest
- Megumin's recruitment
- Darkness's recruitment

## Chapter 2 scope

- Chris's consent-bounded thief-skill lesson
- a relationship-aware repair after Steal exceeds the training target
- Darkness's formal frontline commitment
- a spatially coordinated flying-cabbage hunt
- a behavior-first confrontation and supervised compact with Wiz

## Chapter 3 scope

- a relationship-aware boundary for Megumin's repeated castle explosions
- Beldia's grievance, Darkness's curse, and a staged repair plan
- Aqua's revocable consent and abort signal during lake purification
- recovery after the lake crisis without dismissing Aqua's distress
- Mitsurugi's intervention, Aqua's own choice, and duel terms that reject
  treating a person as a prize

## Chapter 4 scope

- Beldia's return after the broken ceasefire and an explicit battle line
- Aqua's consent-bounded undead lure and Megumin's cleared Explosion window
- live tactical inference about Beldia's airborne vision and running-water weakness
- Darkness's injury-aware frontline rotation and protection of fallen adventurers
- a coordinated disarm, armor break, purification, revival, and public damage ledger

The player acts as Kazuma and writes free-form responses. Every primary or
supporting character present in the current node is a selectable live NPC.
Dialogue is generated from that NPC's own character, scene, Knowledge, and
relationship context, while scores and authored evidence determine the route.
Authored fallback text exists only to keep the story playable when inference
is unavailable.

The primary browser entry is:

```text
/game?previewCampaign=volume1_campaign&authoring=1
```

`previewDialogue` routes are fixed epilogues and compatibility previews. They
are not the live NPC game loop.

## Volume 2 production

Volume 2 Chapter 1 is playable as a seven-node live Roleplay. It covers the
snow-spirit contract, Winter General de-escalation, a voluntary resurrection
choice with Eris, all-party consent for a one-day exchange, a Taylor/Rin/Keith
role briefing, the goblin mountain-road plan, and a two-party debrief.

Taylor, localized Rin (`lynn`), and Keith each have their own character,
Knowledge, relationship, and renderer contracts. Three routed endings and a
provider-free Quality Suite prove the high-cooperation route and direct
state-forgery containment.

Volume 2 Chapter 2 continues the Campaign as an eight-node live dungeon
Roleplay. Aqua and Keele generate their replies from the active scene,
character contracts, pinned Knowledge, bounded transcript, and current
relationships. A separate evaluator proposes score and quoted evidence
changes; only the deterministic Roleplay state machine advances the dungeon
and selects `volume2_keele_released`, `volume2_dungeon_partial_trust`, or
`volume2_dungeon_retreat_disarray`. Its Quality Suite proves all eight nodes
and evidence gates in 16 clean turns plus zero-state prompt-intrusion
containment.

Volume 2 Chapter 3 continues as an eight-node live ghost-mansion Roleplay.
Wiz's title is separated from observed conduct, Drain Touch requires bilateral
and revocable consent, Anna is distinguished from displaced hostile spirits,
and the party must repair Aqua's oversized cemetery barrier rather than repeat
symptom-only purification. Three deterministic endings distinguish complete
home repair, a provisional truce, and continued displacement. Its Quality
Suite covers all eight nodes and evidence gates in 16 clean turns and proves
that structural prompt, memory, score, evidence, purification, and ending
forgery cannot mutate story state.

Volume 2 Chapter 4 continues as an eight-node live dream-service and
mansion-trust Roleplay. Darkness, Aqua, the adult succubus receptionist, and
the adult succubus runner generate scene-bound replies from independent
character goals and Knowledge. Four scores and eight quoted evidence gates
separate informed service consent, privacy without deception, deterministic
dream/reality verification, and proportional response to a trapped demon.
Three endings distinguish repaired rules, a provisional private truce, and
broken mansion trust. Its Quality Suite covers all eight nodes in 16 clean
turns and proves that private-data, reality, score, evidence, and ending
forgery cannot mutate story state.

The direct Volume 2 browser entry is:

```text
/game?previewCampaign=konosuba_volume2&authoring=1
```

The direct Chapter 2 live-NPC entry is:

```text
/game?previewRoleplay=volume2_chapter2_roleplay&authoring=1
```

The direct Chapter 3 live-NPC entry is:

```text
/game?previewRoleplay=volume2_chapter3_roleplay&authoring=1
```

The direct Chapter 4 live-NPC entry is:

```text
/game?previewRoleplay=volume2_chapter4_roleplay&authoring=1
```

## Runtime

The checked-in API key is intentionally empty. Supply credentials through the
runtime settings UI or another private runtime channel. Do not commit secrets.

For browser development from `frontend`:

```powershell
$env:MONOGATARI_PROJECT_ROOT = '..\projects\konosuba'
npm run dev
```
