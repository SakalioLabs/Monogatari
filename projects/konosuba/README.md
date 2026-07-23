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

The player acts as Kazuma and writes free-form responses. NPC dialogue is
generated from character, scene, and knowledge context. Authored fallback text
exists only to keep the story playable when inference is unavailable.

## Runtime

The checked-in API key is intentionally empty. Supply credentials through the
runtime settings UI or another private runtime channel. Do not commit secrets.

For browser development from `frontend`:

```powershell
$env:MONOGATARI_PROJECT_ROOT = '..\projects\konosuba'
npm run dev
```
