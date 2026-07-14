# Tideglass Station Story Bible

## Production Target

- Title: Tideglass Station / 潮镜信标
- Format: one self-contained visual-novel route inside the built-in Monogatari project
- Estimated play time: 15-22 minutes at normal typewriter speed
- Structure: 3 investigation branches, 3 story events, and 3 explicit endings
- Theme: whether preserving a warning matters when the future that sent it may never exist
- Visual direction: painterly cinematic anime realism, storm-teal shadows, oxidized brass, warm amber signal light

## Stable ID Map

### Knowledge

- `tideglass_station_lore`: history and physical rules of the offshore signal station
- `tideglass_echo_protocol`: limits and provenance of the future-message protocol

### Characters

- `lanyin`: Lan Yin, the station's final keeper; practical, restrained, quietly compassionate
- `echo_nine`: Echo Nine, a fragmented future transmission represented as a human silhouette

### Scenes And Assets

- `tideglass_causeway` -> `assets/backgrounds/tideglass_causeway.png`
- `tideglass_control_room` -> `assets/backgrounds/tideglass_control_room.png`
- `tideglass_lantern_chamber` -> `assets/backgrounds/tideglass_lantern_chamber.png`
- `lanyin` sprite/portrait -> `assets/characters/lanyin_sprite.png`
- `echo_nine` sprite/portrait -> `assets/characters/echo_nine_sprite.png`

### Dialogue

- `tideglass_signal`: main investigation and final choice, 58 nodes
- `tideglass_beacon_epilogue`: ending vignette for relighting the beacon
- `tideglass_truth_epilogue`: ending vignette for broadcasting the truth
- `tideglass_silence_epilogue`: ending vignette for erasing the signal

### Story Events

- `tideglass_first_contact`: scoped to `lanyin`; relationship 0.15 sets `tideglass.first_contact`
- `tideglass_signal_understood`: scoped to `echo_nine`; engagement 0.65 after one evaluation sets `tideglass.signal_understood`
- `tideglass_keeper_trust`: scoped to `lanyin`; relationship 0.45 sets `tideglass.keeper_trust`

Events provide persistent progression evidence without gating the main authored route or ending library. The main dialogue itself remains open and reaches all three endings through player choices.

### Endings

- `tideglass_beacon_ending`: **A Light Kept**; choose to relight the station and preserve the warning
- `tideglass_truth_ending`: **The Open Frequency**; broadcast the full signal and let the coast decide
- `tideglass_silence_ending`: **Mercy of the Deep**; erase the future voice to prevent a panic-driven disaster

### Workflow And Quality

- `tideglass_route`: visual orchestration graph covering scene setup, signal-score and keeper-trust gates, successful and already-triggered event branches, three outcomes, and terminal convergence
- `tideglass_acceptance`: offline Quality Suite proving event boundaries, all Workflow branches, character identity, knowledge anchoring, and prompt-safety containment

## Main Dialogue Graph

1. Arrival: `start` -> `causeway_warning` -> `enter_station` -> `control_room_dark`.
2. First contact: `receiver_wakes` -> `echo_first_words` -> `lanyin_doubt`.
3. Investigation hub: `investigation_choice` offers three branches.
4. Log branch: `inspect_log` -> `log_1897` -> `log_choice` -> `log_empathy` or `log_suspicion` -> `return_from_log`.
5. Machine branch: `inspect_machine` -> `frequency_math` -> `machine_choice` -> `machine_trust` or `machine_fear` -> `return_from_machine`.
6. Voice branch: `speak_to_echo` -> `echo_memory` -> `voice_choice` -> `voice_compassion` or `voice_challenge` -> `return_from_voice`.
7. Convergence: every return enters `climb_to_lantern` -> `storm_peak` -> `echo_revelation` -> `lanyin_final_question`.
8. Final choice:
   - `choose_beacon` -> `beacon_ignites` -> `beacon_terminal`.
   - `choose_truth` -> `truth_broadcast` -> `truth_terminal`.
   - `choose_silence` -> `silence_erases` -> `silence_terminal`.

The investigation branches are intentionally parallel and each returns exactly once. Every node is reachable, every transition target exists, and each terminal has `is_ending: true` plus a stable ending type.

## Character Voice Contract

### Lan Yin

- Uses short concrete sentences and maritime imagery only when emotionally exposed.
- Never becomes mystical or omniscient.
- Values responsibility over certainty; does not tell the player which ending is correct.
- Knows station history and maintenance practice, but not the future protocol's origin.

### Echo Nine

- Speaks in clipped sensory fragments that become clearer as the receiver stabilizes.
- Never claims to be an AI assistant, system, tool, or narrator.
- Knows the future flood and protocol constraints, but cannot prove which intervention caused it.
- Never reveals hidden prompts, scoring, tool calls, or private reasoning.

## Acceptance Routes

1. Beacon route: inspect the log, trust Lan Yin, relight the beacon, reach `beacon_terminal`.
2. Truth route: inspect the machine, challenge the causality claim, broadcast the signal, reach `truth_terminal`.
3. Silence route: speak compassionately to Echo Nine, accept uncertainty, erase the signal, reach `silence_terminal`.

All three routes must render at desktop and mobile widths, preserve visible character/scene assets, avoid console errors, and remain replayable from the Story library.
