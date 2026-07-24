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

The player acts as Kazuma and writes free-form responses. NPC dialogue is
generated from character, scene, and knowledge context. Authored fallback text
exists only to keep the story playable when inference is unavailable.

The primary browser entry is:

```text
/game?previewRoleplay=chapter4_roleplay&authoring=1
```

`previewDialogue` routes are fixed epilogues and compatibility previews. They
are not the live NPC game loop.

## Runtime

The checked-in API key is intentionally empty. Supply credentials through the
runtime settings UI or another private runtime channel. Do not commit secrets.

For browser development from `frontend`:

```powershell
$env:MONOGATARI_PROJECT_ROOT = '..\projects\konosuba'
npm run dev
```
