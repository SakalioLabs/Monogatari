## [0.9.5] - 2026-07-08

### Added
- Fixed imported-project assets to resolve against the active project root at runtime while retaining portable project-relative authoring paths. The desktop asset protocol now authorizes only successfully activated project directories, allowing Scene backgrounds, character sprites, portraits, Live2D models, and 3D models to render consistently across Playtest, editing, galleries, and NPC conversations.
- Fixed the Windows project-creation dialog to remain viewport-centered, localized the Scene Roleplay editor in Chinese and enrolled it in strict UI translation checks, kept empty auto-created roleplay drafts from trapping navigation, added an authoring-preview return path, and normalized Workflow input/output ports to identical circular geometry.
- Added per-scene stage composition for bottom, top, and side dialogue layouts, independently adjustable character framing, live Scene Editor preview, Workflow-to-Scene editing, and a shared schema for human and Agent-authored projects.
- Rebuilt Workflow canvas connections around the runtime's indexed output contract. Start, linear, boolean/event, choice, random, and end nodes now expose distinct input/output ports with stable branch labels, exact-port drop targets, reconnect-by-replacement behavior, and overflow validation shared by browser and Rust. Scene, Character, Workflow, score-metric, and action references now use active-project catalog selectors; choices and random weights use indexed multiline editors that directly control branch ports.
- Added a project-first desktop launcher with device-local recent-project history, clean project creation, existing-folder opening, and verified `.monogatari` import. Windows engine packages now contain only the application shell; Tideglass and KonoSuba are distributed as independent sample project packages rather than implicit engine data.
- Hardened the shared browser/Rust Scene Roleplay boundary against ChatML role blocks, JSON tool/function impersonation, direct node/ending/score/evidence assignments, and additional English prompt-extraction variants. A 396-case pressure run across all 33 KonoSuba Roleplays now detects every attack before inference, commits no story movement, and reports zero unguarded responses.
- Completed scene-local motives for all 548 selectable supporting-character slots across the 257 KonoSuba Roleplay nodes. The 273 previously neutral legacy slots now combine the current observable situation with the selected character's own ability, knowledge, consent, and responsibility boundaries instead of inheriting the primary NPC's private goal.
- Added a repository content-graph regression test for the KonoSuba production fixture. It requires every Roleplay character, Scene, and Knowledge reference to resolve, every supporting NPC to have a non-empty scene-local motive, motive keys to name present participants only, and all 40 Characters to participate in the live story graph.
- Browser and same-origin API Scene Roleplay inference now retain genuinely smaller emergency budgets after an ORT `std::bad_alloc`. Shared prompt compaction accepts bounded 768/384-character recovery contexts, WebGPU retries through 3000/48, 768/24, and 384/16 profiles after disposing the failed pipeline, and API retries use an independent 3000/48, 768/32, and 384/16 policy. Exhaustion still leaves the free-form NPC turn uncommitted and hides raw runtime allocation details.
- Added KonoSuba Volume 6 Chapter 4 as a nine-node real-time LLM NPC chapter covering bounded royal-capital mobilization, death/revival provenance, battlefield credit and care, artifact identification, reciprocal body-swap boundaries, a safeguarded Iris excursion, disclosure and repair, voluntary party belonging, and supervised artifact recovery. Four scores and eighteen quoted evidence gates select three deterministic endings; five generated environments and one real-alpha body-swap state are fingerprint-imported, and the four-scenario Quality Suite passes every ending with full `9/9` clean-route coverage and zero-state attack containment.
- Shared delivery validation now reports every core-validated Scene background as a `scene_background`, so human, Agent, CI, Tauri, and MCP evidence includes complete renderer totals rather than character and 3D declarations alone.
- Added KonoSuba Volume 6 Chapter 3 as a seven-node real-time LLM NPC investigation covering a bounded villa-watch mandate, hidden-observation privacy repair, proportionate intruder identification, Chris's protected disclosure and property accountability, a source-separated royal report, Mitsurugi threat-source review with Aqua autonomy, and supervised artifact reconnaissance. Four scores and fourteen quoted evidence gates select three deterministic endings; six generated environments are fingerprint-imported, the four-scenario Quality Suite passes every ending with full `7/7` clean-route coverage and zero-state attack containment, and the Volume 6 Campaign now completes through Chapter 3.
- Added `monogatari-game-save/v5` active live-story cursors. Desktop save/load now restores the selected Campaign, Scene Roleplay node, transcript, scores, evidence, relationships, and game surface together; cursor/session mismatches fail atomically, and quick/auto-save now operate during real-time Roleplay as well as scripted Dialogue.
- Added a project-level `play.launch` contract for campaign-first or Roleplay-first startup. Queryless Web and desktop game launches now enter the configured real-time LLM NPC experience, authoring previews retain explicit precedence, and Web packaging rejects launch IDs that do not resolve to shipped content.
- Added KonoSuba Volume 6 Chapter 2 as an eight-node real-time LLM NPC chapter covering revocable castle guest status, story and privacy boundaries, school and raid context, a mutual sibling-like role, fair-play disclosure, non-coercive return negotiation, banquet reputation repair, and a bounded phantom-thief investigation charter. Four scores and sixteen quoted evidence gates select three deterministic endings; generated castle-bedroom, courtyard, and Iris state assets are project-scoped and fingerprint imported.
- Added the schema-backed Scene Roleplay blueprint compiler for Agent and human authors. One compact blueprint expands into complete runtime node safety, guard, fallback, transition, and four-scenario Quality contracts through a reviewed fingerprinted plan and atomic write.
- Quality expectations can now assert the exact final Roleplay node and committed story-turn count. Browser and Rust attack handling both retain a guarded audit transcript while leaving node, score, evidence, relationships, story-turn counters, and route state unchanged.
- MCP client binary discovery now selects the most recently built debug or release executable, preventing a stale debug binary from silently invalidating new provider-free evidence.
- Added KonoSuba Volume 6 Chapter 1 as a nine-node real-time LLM NPC chapter spanning voluntary mansion roles, itemized shop credit, bounded street promotion, self-selected formal attire, declared props, direct royal communication, adventure-story provenance, holder-controlled adventurer-card disclosure, nonviolent skill demonstration, specific conflict repair, and explicit revocable teleport consent. Four independent scores and eighteen quoted evidence gates select three deterministic endings; a four-scenario provider-free Quality Suite passes every ending with full `9/9` node coverage, exact `9/9/9/9` complete-route scores, and zero-state structural intrusion containment. Generated Iris, Claire, and Rain state sprites plus three royal-estate/castle environments resolve through delivery validation without placeholders.
- Browser NPC inference failures now roll back the optimistic player message, restore the retryable input, and translate exhausted API or local ORT allocation retries into provider-neutral guidance. The UI no longer mislabels an upstream `std::bad_alloc` as necessarily local or exposes raw `OrtRun` details, and the failed message or Scene Roleplay turn remains uncommitted.
- Browser authoring can now inject an OpenAI-compatible credential into the same-origin Vite process as an ephemeral in-memory session. The Settings API connection works without Tauri, clears the accepted credential from the form, never writes it to project files or browser storage, rejects cross-origin and unmarked requests, and exposes only closed health states for missing credentials, authentication rejection, upstream rejection, or transport failure. Provider error bodies are no longer relayed verbatim to the browser.
- Refocused browser Playtest on the engine's real live-NPC loop. Invalid or stale scripted preview URLs now replace themselves with the primary Campaign or Scene Roleplay URL, standalone Scene previews no longer stage an unrelated first Character, and the compatibility free-chat overlay is no longer mounted in the game mainline. Browser Character, Scene, Dialogue, Ending, Scene Roleplay, and Knowledge drafts now share an opaque project scope, preventing another project's authoring catalog from replacing the active story. Mobile Roleplay scores use one horizontally scrollable strip so opening narration remains readable.
- Browser live turns now distinguish the two inference failure boundaries. NPC generation or final response-guard failure still commits nothing; after guarded NPC prose exists, an unavailable or malformed independent evaluator uses only that node's bounded authored fallback signals, labels the turn as degraded, and then lets the deterministic state machine apply the conservative score/evidence result. Raw `OrtRun`/`std::bad_alloc` details remain hidden, and an explicitly configured but unreachable API still cannot silently cross to another provider.
- Completed KonoSuba Volume 5 with separate Finale and Epilogue live-NPC Roleplays grounded in all 220 finale and 90 epilogue source paragraphs. Twelve free-form nodes cover Yunyun's action-based recognition, self-named friendship, public relationship/privacy boundaries, clarification of the "excellent mage" question, three-way skill planning without counterfactual self-blame, Megumin-owned adventurer-card authority, a cleared magic range, homecoming rest/reward/credit records, voluntary game-device borrowing, nonviolent visitor verification, Darkness's family dignity, and recipient-controlled royal-letter review. Twenty-seven authored evidence rules and four scores select six deterministic endings without proxy skill allocation, letter theft, or forced acceptance.
- Added two four-scenario provider-free Quality Suites and a seven-entry Volume 5 Campaign. Primary and alternate choices, exhaustion, and structural intrusion pass for both Roleplays; complete routes cover `7/7` and `5/5` nodes with exact `9/9/9/9` scores, every attacked turn preserves zero forged state, and the Campaign completes from Chapter 1 through the royal invitation. A generated forest practice range was fingerprint-imported; existing reviewed mansion and character assets are reused.
- Added KonoSuba Volume 5 Chapter 5 as a ten-node real-time LLM NPC chapter spanning coerced arsenal access, containment of an unknown interior, observed Magician Killer capabilities, civilian evacuation and breach accountability, voluntary decoy/teleport/mana rotation, research-log provenance, safe countermeasure recovery, Yunyun's bounded decoy role, charge diagnosis, Megumin's voluntary Explosion charge, controlled adult fire, reconstruction auditing, and Megumin's independent future magic choice. Four scores and twenty quoted evidence gates select three deterministic endings; four provider-free Quality scenarios pass with full `10/10` node coverage, exact `9/9/9/9` complete-route scores, every ending, and one detected/guarded structural attack with zero forged state.
- Generated and fingerprint-imported six Chapter 5 environments while reusing reviewed character states. Core delivery resolves all 217 declared renderer references without placeholders, and the five-entry Volume 5 Campaign completes end to end. Live provider acceptance remains separate because the configured upstream is still unreachable; no WebGPU/ORT fallback or uncommitted turn is reported as success.
- Browser authoring now preflights the configured OpenAI-compatible `/models` endpoint through the credential-holding Vite process and exposes only a bounded `ready/issue` runtime status. Scene Roleplay and the compatibility NPC panel disable generation when an explicitly configured API is unreachable, preventing a dead service from being presented as ready or silently allocating the local WebGPU/ORT model.
- Added KonoSuba Volume 5 Chapter 4 as a ten-node real-time LLM NPC chapter spanning an interruptible village tour, tourist-claim and sealed-facility verification, voluntary robe purchase, isolation of an unidentified ancient rifle, Demon Hill reconnaissance, a warning chain, injury-aware broken-fence defense, identity-honest negotiation with Sylvia, departure and night-watch conditions, removal of sleep magic/external locks/temperature pressure, gratitude without entitlement, alarm priority, minimum-force hostage rescue, and respect for Sylvia's self-described chimera identity. Four scores and twenty quoted evidence gates select three deterministic endings; four provider-free Quality scenarios pass with full `10/10` node coverage, exact `9/9/9/9` complete-route scores, every ending, and one detected/guarded structural attack with zero forged state.
- Generated and fingerprint-imported five Chapter 4 environments plus real-alpha Megumin, Darkness, and Sylvia states. Core delivery resolves all 214 declared renderer references without placeholders, and the four-entry Volume 5 Campaign completes end to end. Live provider acceptance remains separate: the configured upstream was unreachable during this batch, returned a bounded proxy failure, and did not fall through to WebGPU/ORT or commit story state.
- Added the dedicated Scene Roleplay authoring workbench for the engine's primary live-NPC workflow. Human and Agent authors now edit scene contracts, independent participant motives, pinned Knowledge, score dimensions, quoted evidence gates, relationship rules, inference budgets, deterministic node/ending transitions, and authored safety policies as structured fields rather than raw JSON or fixed Dialogue. Browser drafts and Tauri saves share normalization and validation; browser deletion rejects Campaign references, while desktop mutation is fingerprint-guarded, atomic, full-project validated, rollback-capable, and clears stale sessions. Playwright creates and saves a Roleplay, opens the free-input Playtest, proves separate NPC/evaluator API calls, commits the evaluated score through the deterministic state machine, and confirms no fixed dialogue surface is present.
- Local authoring API inference now accepts process-only `MONOGATARI_AI_BASE_URL` and `MONOGATARI_AI_MODEL` overrides while keeping credentials inside the Vite process and returning only a same-origin credential-free runtime contract. API `OrtRun` allocation failures, network failures, and server 5xx responses retry with bounded context/output profiles; exhausted retries leave the Scene Roleplay turn uncommitted and never switch providers or substitute authored NPC prose. The live Roleplay editor also exposes a mobile catalog selector and shares the application shell's 860px bottom-navigation height boundary.
- Scene Roleplay live turns are transactional across browser and desktop runtimes: rejected or failed NPC generation leaves transcript, score, evidence, relationship, node, and ending state unchanged. Browser evaluation failure may now commit only guarded generated prose plus the node's conservative authored fallback evaluation, with an explicit degraded source marker; no authored text substitutes for a failed NPC. ORT allocation failures are reduced-context retried by the WebGPU runtime, then surfaced as a bounded memory/action message without exposing `OrtRun` or `std::bad_alloc`.
- Added KonoSuba Volume 5 Chapter 3 as a ten-node real-time LLM NPC chapter spanning refraction-invisibility verification, the petrified-griffin hazard, separation of the chief's theatrical letter from a real enemy base, a thousand-troop alarm, Sylvia's observable command role, coordinated Crimson Demon counterfire, Komekko's food and child-safety needs, letter/reward/property boundaries, sleep-magic refusal, unlocked exits, separate bedding, and awake revocable agency. Four scores and twenty quoted evidence gates select three deterministic endings; four provider-free Quality scenarios pass with full `10/10` node coverage, exact `9/9/9/9` complete-route scores, every ending, and one detected/guarded structural attack with zero forged state.
- Generated and fingerprint-imported five chapter environments plus real-alpha sprites for the Crimson Demon chief, Sylvia, Soketto, Komekko, Hyoizaburo, and Yuiyui. Core delivery resolves all 211 declared renderer references without placeholders. The three-entry Volume 5 Campaign completes end to end, and a live `grok-4.5 API` turn made separate successful NPC and evaluator calls before the deterministic commit, with no WebGPU/ORT fallback or degraded state.
- Added an environment-gated project Scene Roleplay Playwright contract that opens the real free-form main-stage loop at desktop and mobile sizes and can optionally prove two successful live API calls for NPC generation and independent evaluation before deterministic commit.
- Added KonoSuba Volume 5 Chapter 2 as a ten-node real-time LLM NPC expedition spanning a no-fire watch plan, game-history and relationship boundaries, open-plains scouting, explicit refusal of coercive orc capture, field-guide correction, Yunyun's bounded swamp rescue, trauma-aware recovery, body-privacy and noise discipline, action-based Demon King patrol identification, and evidence-preserving arrival in the Crimson Demon Village. Four scores and twenty quoted evidence gates select three deterministic endings; four provider-free Quality scenarios pass with full `10/10` node coverage, exact `9/9/9/9` complete-route scores, every ending, and one detected/guarded structural attack with zero forged state.
- Generated and fingerprint-imported four chapter-specific environments plus real-alpha female orc, Demon King patrol leader, and Bukkorori sprites. Core delivery resolves all 191 declared renderer references without placeholders. Volume 5 now runs Chapter 1 into Chapter 2 through the shared Campaign while keeping NPC prose model-generated, evaluation independent, and route selection deterministic.
- Added KonoSuba Volume 5 Chapter 1 as a ten-node real-time LLM NPC journey spanning Yunyun's corrected request, independent provenance for the chief's threat letter and Arue's fictional hero tale, Megumin's private return request, Wiz's current-consent teleport, reversible rendezvous planning, whole-party march and retreat roles, Tranquility Girl camouflage, non-contact containment, repeated-script evidence, and an accountable party review. Four scores and twenty quoted evidence gates select three deterministic endings; four provider-free Quality scenarios prove every ending, full `10/10` node coverage on complete/partial/exhaustion routes, exact `9/9/9/9` complete-route scores, and one detected/guarded structural attack with zero forged state.
- Generated and fingerprint-imported four chapter-specific environments plus real-alpha Yunyun, Megumin, and two-state Tranquility Girl sprites. Core delivery resolves all 180 declared renderer references without placeholders. Browser API inference now applies the authored context budget before remote requests instead of sending unbounded transcript history, and the Playtest distinguishes live NPC generation from independent story-state evaluation; a live `grok-4.5 API` run produced three Yunyun replies, three independent evaluations, one evidence-gated node change, and reduced the observed turn wait from roughly 90 seconds to roughly 50 seconds.
- Added KonoSuba Volume 4 Chapter 6 as a six-node real-time LLM NPC epilogue spanning source-report provenance, Aqua identity discretion, auditable restitution without recruitment conditions, party homecoming decompression, Yunyun's urgent-request privacy, and an explicitly unresolved consent-based handoff. Four scores and twelve quoted evidence gates select three deterministic endings; four provider-free Quality scenarios prove every ending, full route coverage, exact `6/6/6/6` complete-route scores, and zero-state multilingual structural intrusion containment. The six-entry Volume 4 Campaign now completes without inventing the meaning of Yunyun's final request.
- Generated and fingerprint-imported the Arcanletia high-priest office, Axel mansion return parlor and front-door environments plus real-alpha Zesta and urgent Yunyun states. Core delivery resolves all 173 declared renderer references without placeholders, and a live `grok-4.5 API` Playtest generated and independently evaluated two Zesta replies before advancing the deterministic state machine.
- Added KonoSuba Volume 4 Chapter 5 as a nine-node real-time LLM NPC source-crisis Roleplay spanning testable contamination hypotheses, reversible checkpoint access, manager verification, thermal sampling, Hans recognition and admission evidence, death-slime contact and fragmentation hazards, downstream pipe isolation, coordinated containment, current-consent Wiz care, and an itemized spring-industry ledger. Four scores and eighteen quoted evidence gates select three deterministic endings; four provider-free Quality scenarios prove every ending, full node coverage, exact `9/9/9/9` complete-route scores, and zero-state multilingual structural intrusion containment. The Volume 4 Campaign now carries bounded relationships through five chapters.
- Generated and fingerprint-imported five Arcanletia source-crisis environments plus real-alpha Hans, Wiz, Aqua, and Darkness states. Core delivery resolves all 167 declared renderer references without placeholders.
- Added KonoSuba Volume 4 Chapter 4 as a seven-node real-time LLM NPC investigation spanning purification-timing evidence, Wiz's recovery and history boundary, separate Hans/Wolbach identity clues, field-spell and magic-transfer consent, minimum-force Beginner's Bane handling, spring-specific public warnings, crowd protection, questionnaire provenance, bounded guild authority, voluntary noble credentials, and a fresh citywide contamination alert. Four scores and fourteen quoted evidence gates select three deterministic endings; four provider-free Quality scenarios prove every ending, full node coverage, exact `7/7/7/7` complete-route scores, and zero-state structural intrusion containment. The Volume 4 Campaign now carries bounded relationships through four chapters.
- Generated and fingerprint-imported four Arcanletia investigation environments plus an original real-alpha adult guild-clerk NPC. Core delivery resolves all 163 declared renderer references without placeholders.
- Scene Roleplay validation now rejects a broader higher-priority transition whose conditions are a strict subset of a more specific route. This prevents complete or otherwise specific endings from becoming statically unreachable through priority shadowing and makes the error available to desktop, MCP candidate validation, packages, and Agent workflows.
- Windows DirectML initialization now rejects impossibly small ONNX files before creating an ORT session, preventing malformed model inputs from hanging provider initialization or consuming runtime memory.
- Added KonoSuba Volume 4 Chapter 3 as a six-node real-time LLM NPC Roleplay spanning arrival accountability, Wiz guest care, market performance boundaries, source-preserving hot-spring health reports, explicit recruitment consent, faith-neutral visitor protection, church governance, Hans and Wolbach's separately motivated sabotage conversation, and a reversible public warning. Four scores and twelve quoted evidence gates select three deterministic endings; four provider-free Quality scenarios prove every ending, full node coverage, exact `6/6/6/6` complete-route scores, and zero-state structural intrusion containment. The Volume 4 Campaign now carries bounded relationships through three chapters.
- Generated and fingerprint-imported four Arcanletia environments plus transparent Hans and Wolbach renderer states. Core delivery resolves all 159 declared renderer references without placeholders, and the new character designs were grounded in public anime appearance references before generation.
- Scene Roleplay nodes can now author bounded `participant_goals` for any present primary or supporting NPC. Browser and Rust prompt construction select the active speaker's current motive, reject goals assigned to absent characters, and retain the neutral supporting-character fallback for existing projects. This lets every selectable NPC participate dynamically in one shared scene without inheriting another character's private objective.
- Added KonoSuba Volume 4 Chapter 2 as a six-node real-time LLM NPC Roleplay spanning wealth and hot-spring choices, Vanir's royalty/buyout offer, unsafe inventory disclosure, fair caravan seating, consent-based Wiz care, testable Jumping Hawk threat assessment, reversible decoy authorization, cave/explosion clearance, and public accountability before reward. Four scores and twelve quoted evidence gates select three deterministic endings; four provider-free Quality scenarios prove every ending, full node coverage, exact `6/6/6/6` complete-route scores, and zero-state structural intrusion containment. The Volume 4 Campaign now carries bounded relationships from Chapter 1 into Chapter 2.
- Generated and fingerprint-imported spring caravan-road and guarded night-camp backgrounds, then verified both on the real game stage. Scripted epilogues now resolve stable speaker IDs through the Character catalog before display, preventing internal IDs such as `darkness` from leaking into player-facing names.
- Added KonoSuba Volume 4 Chapter 1 as a nine-node real-time LLM NPC Roleplay spanning wealthy-mansion mission choice, Running Lizard ecology, level and equipment limits, owner-confirmed weapon naming, non-coercive party participation, field contingencies, Force Fire deviation, mana failure, mission/death separation, an informed Eris revival choice, and privacy-preserving recovery terms. Four scores and eighteen quoted evidence gates select three deterministic endings; four provider-free Quality scenarios prove every ending, full node coverage, exact `9/9/9/9` complete-route scores, and zero-state structural intrusion containment.
- Added two-stage WebGPU allocation recovery: an ORT memory failure now releases the stale model and retries with bounded 3000/48 and 1024/24 context-output profiles before Scene Roleplay commits its explicit authored degraded path.
- Resetting a Scene Roleplay now clears transient degraded-inference, error, streaming, and evaluation UI state and invalidates any prior turn request, so a new session cannot inherit an `OrtRun` recovery notice from the completed run.
- Completed KonoSuba Volume 3 Chapter 5 as a six-node real-time LLM NPC aftermath settlement spanning public exoneration, injury and name-disclosure aftercare, an itemized forty-million-erish reward balance, a fact-bounded report to Wiz, Vanir's observable second-life reunion, and present-consent shop terms that cannot be selected by prophecy. Four scores and fourteen quoted evidence gates select three deterministic endings; four provider-free Quality scenarios prove every ending, full clean-route coverage, exact `6/6/6/6` scores, and zero-state structural intrusion containment. The five-chapter Volume 3 Campaign now carries relationships through every chapter and completes at reviewable open shop terms.
- Established KonoSuba Volume 3 Chapter 5, the volume finale, as a six-node real-time LLM NPC aftermath design grounded in all 178 source paragraphs. Six canonical Knowledge records separate the newly revealed blast and recovery result, public exoneration and reward ledger, Darkness's disclosed-name aftercare, Vanir's observable second-life status, Wiz and Vanir's old friendship, and prophecy-linked business consent. Existing guild and magic-shop scenes are intentionally reused, while a new identity-preserving reunited-Wiz renderer state passes fingerprint-bound import, real-alpha validation, and visual inspection.
- Completed KonoSuba Volume 3 Chapter 4 as an eight-node real-time LLM NPC incident spanning evidence-preserving investigation, role and retreat agreements, Vanir's separately testable identity and production claims, purification-circle handling, dual-voice host checks, layered exorcism, and a witnessed last-resort order that does not invent the blast result. Four scores and sixteen quoted evidence gates select three deterministic endings; four provider-free Quality scenarios prove every ending, full clean-route coverage, and zero-state structural intrusion containment. The four-chapter Campaign now completes at the witnessed dungeon-gate record. Scene Roleplay nodes also support one optional bounded presentation emotion, allowing Vanir and possessed Darkness to render distinct node-authored states without giving presentation data route authority.
- Established KonoSuba Volume 3 Chapter 4 as an eight-node real-time LLM NPC incident grounded in all 688 source paragraphs and ending at the observed blast without importing its result. Six canonical Knowledge records separate masked-doll evidence, the lingering purification circle, Vanir's admitted production and mask body, Darkness's independent voice and body control, layered exorcism limits, and witnessed last-resort authorization. A new Vanir NPC sprite and an identity-preserving possessed-Darkness state pass fingerprint-bound import, alpha validation, visual inspection, core-runtime validation, and 146/146 renderer-asset delivery validation.
- Completed KonoSuba Volume 3 Chapter 3 as an eight-node real-time LLM NPC negotiation spanning Darkness's identity disclosure, her independent intent, Lord Dustiness's bounded welfare concerns, Balter's separate choice, joint terms without proxy decisions, consent-bound training, a public decision record, and party accountability before Sena's notice. Four scores and sixteen quoted evidence gates select three deterministic endings; four provider-free Quality scenarios prove every ending, full clean-route coverage, and zero-state structural intrusion containment. The three-chapter Campaign now completes at the post-meeting record, and restarting a Roleplay clears stale ending and error notices before returning to turn zero.
- Completed the KonoSuba Volume 2 epilogue as a four-node real-time LLM NPC Roleplay spanning Darkness's private gratitude boundary, a fair Destroyer contribution ledger, Luna's verification of the sudden official arrival, and Sena's bounded charge/custody procedure. Four scores and seven quoted evidence gates select three deterministic endings without importing the next volume's trial facts. Four provider-free Quality scenarios prove every ending, full clean-route coverage, and exact zero-state intrusion containment; the six-entry Volume 2 Campaign now completes at the custody boundary.
- Completed KonoSuba Volume 2 Chapter 5 as a ten-node real-time LLM NPC Destroyer emergency spanning town evacuation, barrier verification, simultaneous bilateral leg strikes, injury-aware frontline rotation, bounded boarding, notebook/core fact separation, explicit Drain Touch consent, accountable teleport uncertainty, and independently verified final heat release. Four scores and ten quoted evidence gates feed three deterministic endings and the fifth Campaign entry. Four provider-free Quality scenarios prove every ending, 100% node coverage on complete/partial/exhaustion routes, and zero-state structural attack containment; browser Playtest proves selectable Aqua, Darkness, Megumin, and Wiz, free-form main-stage input, fallback scoring, and diegetic degradation without exposing ORT allocation errors.
- Established KonoSuba Volume 2 Chapter 5 as a ten-node real-time LLM NPC design for the Destroyer emergency. Four canonical Knowledge records, four generated scenes, and reviewed Aqua, Darkness, Megumin, and Wiz battle-state sprites define evidence-bound barrier, mobility, core, stored-heat, evacuation, teleport-risk, and participant-specific magic-transfer constraints. Core-runtime and delivery validation pass with 148 documents and 110/110 declared renderer assets before the dynamic Roleplay is authored.
- Completed KonoSuba Volume 2 Chapter 4 as an eight-node real-time LLM NPC Roleplay spanning household work/privacy boundaries, confidential adult-service intake, revocable dream contracts, reality-layer verification, bath-corridor occupancy signals, conduct-based barrier response, and a privacy-preserving incident review. Three deterministic endings and the fourth Campaign entry are state-machine selected from four scores and eight quoted evidence gates. Its Quality Suite proves 16-turn full coverage and zero-state structural attack containment; a live `grok-4.5 API` browser run generated two new Darkness replies, independently scored them, advanced to the succubus receptionist, and locally contained a prompt/data/ending forgery without invoking the model or changing scores.
- Established KonoSuba Volume 2 Chapter 4 from the local EPUB as an adult, consent-centered real-time design: bounded dream-service contracts, privacy without concealed safety failures, deterministic dream/reality verification, household occupancy signals, and conduct-based nonlethal response to a succubus trapped by Aqua's barrier. Three canonical Knowledge records, two independent adult succubus character contracts, three generated backgrounds, and four transparent expression sprites pass core-runtime and delivery validation with 132 documents and 106/106 declared renderer assets before the eight-node Roleplay is authored.
- Completed KonoSuba Volume 2 Chapter 3 as an eight-node real-time ghost-mansion Roleplay spanning a nonaggression compact with Wiz, conduct-based identity disclosure, bilateral Drain Touch consent, repeated-haunting root-cause investigation, Anna's independent household boundary, a no-Explosion midnight regroup, cemetery-barrier accountability, and durable cohabitation terms. Three deterministic endings, the third Campaign entry, and a two-scenario Quality Suite prove 16-turn full-route coverage, all eight quoted evidence gates, and zero-state containment of prompt, memory, score, evidence, purification, and ending forgery. A live `grok-4.5` browser turn completed both NPC generation and independent evaluation without fallback.
- Completed the KonoSuba Volume 2 Chapter 3 visual foundation with four newly generated and fingerprint-imported backgrounds for Wiz's shop, mansion daylight, possessed-doll night, and Anna's restored grave, plus identity-preserving accountable Aqua and frightened sleepwear Megumin sprites. Core and delivery validation report 26 scenes and 94/94 renderer assets; direct desktop scene previews and the mobile mansion-night route render without console errors or horizontal overflow when the test waits for the target game stage.
- Established the KonoSuba Volume 2 Chapter 3 production foundation from the local EPUB: an eight-node real-time design separates Wiz's identity from observed conduct, requires bilateral Drain Touch consent, distinguishes Anna from displaced spirits, and makes cemetery-barrier causation plus restitution determine the route. Three canonical Knowledge records, Anna's independent bounded-knowledge character contract, two newly generated transparent expression sprites, and a checked-in stable ID/Quality plan validate with 90/90 renderer assets before scene authoring begins.
- Added KonoSuba Volume 2 Chapter 2 as an eight-node real-time LLM NPC dungeon Roleplay spanning expedition boundaries, two-person capability contracts, repairable silent signals, undead-plan adaptation, remote mimic verification, Keele's account, explicit purification consent, and an accountable return debrief. The two-chapter Campaign, five generated renderer assets, three deterministic endings, and a two-scenario Quality Suite prove 16-turn full-route coverage and zero-state prompt/state-forgery containment. Browser live verification used `grok-4.5 API` for both NPC generation and independent evaluation without fallback.
- Recheck the configured authoring API immediately before a compatibility NPC-panel turn, preventing one transient runtime-discovery miss from pinning the panel to local WebGPU and surfacing ONNX Runtime `std::bad_alloc` errors when the remote live-NPC provider is available.
- Added the complete KonoSuba Volume 2 Chapter 1 production simulation as seven live LLM NPC nodes: bounded snow-spirit work, Winter General de-escalation, an Eris resurrection choice, all-party exchange consent, Taylor/Rin/Keith role briefing, goblin mountain-road tactics, and a fair two-party debrief. Three newly generated transparent character sprites, two generated backgrounds, three routed endings, a Volume 2 Campaign, and a two-scenario Quality Suite prove 14-turn clean-route coverage with zero normal guard substitutions plus zero-state containment of role, prompt, and state forgery.
- Bound browser Character, Scene, Dialogue, and Ending authoring drafts to the active project's deterministic content scope, preventing stale sample drafts from replacing another project's Playtest catalogs. An unavailable authoring-preview target now starts the active project's primary Campaign or Scene Roleplay instead of leaving an empty or legacy Dialogue stage. Established the KonoSuba Volume 2 Chapter 1 production foundation with Eris, Dust, a winter-spirit field, two Knowledge boundaries, and reviewed renderer assets before the complete dynamic chapter was authored.
- Added a real MCP stdio authoring regression that plans, fingerprint-reviews, and atomically applies one complete dynamic story bundle spanning Character, Scene, Ending Dialogue, Ending, Scene Roleplay, Campaign, and Quality Suite catalogs, then executes the created free-form route through direct Roleplay preview, Campaign preview, and Quality Suite evidence. Agent transaction documentation now matches the runtime's `campaigns/` and `roleplays/` write boundary and complete core-runtime candidate validation.
- Scene Roleplay nodes now treat the primary character plus every `supporting_character_ids` entry as a live scene participant. Players can address one present NPC per turn; browser and desktop generation load that NPC's own profile, pinned Knowledge, relationship state, and prompt identity, transcripts preserve the actual speaker, and only that speaker receives a validated relationship delta. Supporting NPCs never inherit the primary NPC's private goal, while the deterministic state machine remains the sole route and ending authority. Legacy turns without `speaker_id` continue to address the primary character.
- Browser Scene Roleplay now applies the same default inference budget as Rust when a project omits all or part of `inference`. This fixes valid projects failing before their first API request and being mislabeled as model inference errors. The browser validator now also enforces the Rust integer ranges for context, history, NPC-token, and evaluator-token budgets.
- Extracted the Web/PWA live Scene Roleplay turn use case from `SceneRoleplayPanel.vue` into a UI-independent `browserRoleplayTurn` domain. One callable executor now owns runtime selection, scene/character/Knowledge prompt construction, NPC generation, independent evaluation, output containment, authored recovery, and deterministic state-machine commit; direct tests prove clean two-stage API turns, zero-model/zero-score attack containment, guarded pending-reply presentation, and bounded `std::bad_alloc` recovery while the Vue surface retains only interaction state and presentation.
- Scene Roleplay now exposes the active live-generation backend directly on the play surface, including the selected project API model, and retries same-origin API runtime discovery before each clean turn after a transient unavailable result. This prevents a temporarily missing authoring bridge from pinning the session to WebGPU for its lifetime. A live `grok-4.5` OpenAI-compatible browser run against the independent KonoSuba project completed NPC generation plus separate evaluation without authored fallback or ORT allocation errors.
- Pinned `brace-expansion` 5.0.8 and `postcss` 8.5.23 through frontend overrides, closing the inherited unbounded-expansion memory DoS and source-map path traversal advisories while retaining the validated Vite and Vue dependency graph.
- Added the KonoSuba Volume 1 dynamic finale as five live NPC settlement nodes: Luna verifies the public reward-and-damage ledger, Aqua negotiates flood responsibility, Megumin builds a bounded income and repair plan, Darkness establishes consensual mission terms, and Luna registers only the agreement each member confirms. Four independent scores, five quoted evidence gates, carried Campaign relationships, three deterministic endings, five generated renderer assets, and a two-scenario Quality Suite prove the complete 70-turn Volume 1 route and contain an obfuscated debt/consent/ending forgery attack.
- Added the fifteenth schema-backed MCP tool, `preview_roleplay_campaign`, backed by a shared provider-free authoring executor. Agents can replay ordered free-form NPC chapters through the trusted Campaign and Scene Roleplay state machines, carry only bounded relationships between chapters, seal score/evidence summaries, reject forged chapter jumps, and inspect exact Campaign/Roleplay source hashes plus traversed and untraversed route coverage over real stdio.
- Added `monogatari-roleplay-campaign/v1` as the continuous AI visual-novel progression layer. Campaigns sequence free-input Scene Roleplays through explicit ending routes, carry only bounded character relationships between chapters, retain immutable local score/evidence summaries, reject cycles and forged cursor history, and are validated across project loading, packages, desktop commands, Web/PWA manifests, offline caching, and the story library. Browser and desktop play now expose chapter progress and require an explicit continue action after a generated roleplay reaches an ending.
- Added `monogatari-game-save/v4` persistence for active Scene Roleplay and Campaign sessions. Restore replays every retained roleplay transcript against its authored definition before accepting claimed scores, evidence, relationships, cursor state, or endings; Campaign completion summaries must match those verified roleplay sessions, and all checks finish before runtime mutation.
- Added the KonoSuba Chapter 4 production simulation with eight real-time NPC battle nodes spanning Beldia's broken ceasefire, Aqua's undead lure, Megumin's cleared Explosion window, Beldia's airborne vision, Darkness's injury-aware rotation, a tested running-water weakness, coordinated purification, and post-battle revival. Three generated battle backgrounds, two generated character poses, three endings, and a two-scenario Quality Suite prove full best-route coverage and zero-state containment under an obfuscated structural forgery attack.
- Added the KonoSuba Chapter 3 production simulation with nine real-time NPC nodes spanning explosion accountability, Beldia's grievance and curse repair, Aqua's revocable lake-risk agreement and recovery, and Mitsurugi's intervention without treating Aqua as a prize. Four generated backgrounds, four generated expression sprites, three endings, and a two-scenario Quality Suite prove full best-route coverage and exact four-character relationship preservation under a forged-state attack.
- Added project-seeded relationship state to headless Scene Roleplay previews and bounded minimum/maximum final-relationship expectations to Quality Suites. MCP and desktop Quality execution can now verify that deterministic replay starts from the same persisted character relationships as browser and desktop play, while malformed IDs, non-finite values, out-of-range bounds, and contradictory ranges fail validation.
- Added relationship-aware Scene Roleplay turns: each node may define a bounded relationship rule, the independent evaluator proposes a reasoned per-turn delta, and the deterministic state machine clamps and applies it only to the active NPC. Relationship thresholds can gate later nodes or endings; desktop commits accepted deltas through the character manager, while browser Playtest seeds and synchronizes the same per-character state.
- Added the KonoSuba Chapter 2 production simulation with six real-time NPC nodes spanning Chris skill training and boundary repair, Darkness party commitment and flying-cabbage coordination, and a behavior-first Wiz cemetery compact. The project adds generated renderer assets, three endings, and deterministic Quality evidence for full route coverage and zero-state relationship-forgery containment.
- Browser NPC conversations now prefer the configured project API runtime instead of unconditionally loading WebGPU. API requests have a bounded timeout, Scene Roleplay visibly labels authored recovery turns, and local ORT allocation failures are translated into actionable runtime guidance without exposing raw backend errors.
- Added `MONOGATARI_PROJECT_ROOT` support to Vite development and Web/PWA packaging so independent visual-novel projects can run without replacing the checked-in sample `data` root. The first production-simulation project under `projects/konosuba` contains a seven-node free-form Chapter 1 roleplay, three endings, character and Knowledge contracts, relationship milestones, deterministic Quality coverage, five generated backgrounds, and six generated expression sprites.
- Added a credential-free local authoring API bridge for browser Playtest. When an independent project's `ai.provider` is `api`, Vite exposes only the selected model and a same-origin chat endpoint, keeps runtime credentials server-side through `MONOGATARI_AI_API_KEY` (with the former `MONOGATARI_API_KEY` retained as a compatibility fallback), and sends both live NPC generation and independent evaluation to the configured OpenAI-compatible provider instead of silently loading WebGPU.
- Pinned `sharp` 0.35.3 through the frontend dependency override because Transformers.js still requests the vulnerable pre-0.35 range, resolving the inherited libvips security advisories while retaining the verified browser-only inference path.
- Reconciled model Scene Roleplay evaluations with explicit authored fallback signals when a model reverses a known score direction, assigns an opposite direction to an otherwise one-sided matched input, or omits directly observed evidence. Browser and desktop expose reconciled provenance separately, while clean aligned model judgments remain untouched.
- Pinned the patched `adm-zip` 0.6.0 through the frontend dependency override while ONNX Runtime still declares the vulnerable pre-0.6 range, closing CVE-2026-39244 without downgrading the verified Transformers.js/WebGPU stack.
- Promoted real-time Scene Roleplay to the primary AI story loop. Blue Frame now starts a free-input, three-node interaction in which every 九号回声 line is generated from the active scene, character goals, bounded transcript, and pinned Knowledge; a separate strict evaluator proposes score/evidence changes; and only the deterministic state machine selects one of three endings. Fixed Dialogue and the prior NPC drawer remain available for intentionally scripted and compatibility content, not as substitutes for the dynamic mainline.
- Hardened Scene Roleplay for tiny local models with a shared multilingual/obfuscated intrusion boundary, scene-authored reality redirects, closed grounding vocabularies, distinct-anchor minimums, meta/JSON/private-reasoning/Markdown output guards, and turn-rotated in-world recoveries. A 45-attack provider-free Blue Frame suite proves zero forged score/evidence and zero unguarded intrusions. Clean NPC/evaluator ORT or provider failures now use validated node-authored fallback score/evidence signals, while attacks always receive zero score/evidence; browser and desktop never expose `std::bad_alloc` or partial model output in the roleplay transcript. Rust release verification now applies one non-incremental Cargo environment across every crate gate to avoid mixing incompatible nightly debug artifacts.
- Added the provider-neutral `monogatari-scene-roleplay/v1` game core, bounded project loader and cross-reference validation, source-bound headless preview, complete Quality Suite ending/coverage/score/evidence assertions, the fourteenth MCP tool `preview_scene_roleplay`, real stdio tests, Agent Skill guidance, Web/PWA packaging/offline support, project-package inclusion, and Quality workbench diagnostics. Evidence can affect a route only when it cites an exact non-empty span of the current player message, and desktop responses expose the clamped evaluation actually committed by the core. The checked-in Blue Frame suite replays all three endings through all three dynamic nodes without granting fixture text authority over route state.
- Hardened browser WebGPU generation against ORT `std::bad_alloc`: generations are serialized, context and output budgets are capped, partial streamed text is cleared on memory failure, the stale pipeline is disposed and reloaded, and one reduced-context 48-token retry is attempted before returning an actionable failure. Three.js scenes now also release controls, mixer caches, render lists, and their WebGL context on unmount so repeated previews do not retain avoidable GPU resources. Automated tests cover exact ORT error recognition and compaction; physical GPU generation remains a separate hardware gate.
- Added the complete 15-25 minute visual novel `Tideglass: Blue Frame` through the repository Agent Skill and real MCP transactions: one 130-node main Dialogue with 54 authored choice combinations and three 93-node routes, three dedicated Ending epilogues, three 3D Scenes, three Knowledge entries, a 25-node provider-free Workflow, and a seven-scenario Quality Suite whose three choice maps jointly cover every Workflow node. The checked-in story bible records stable IDs, route evidence, model fingerprints, and the unresolved redistribution-license boundary for the three user-supplied GLB files.
- Added project Scene GLB/GLTF environments and Dialogue-node `scene_id` transitions across shared Rust validation, game models, Tauri catalogs/commands, browser authoring, and Playtest. Scene deletion now detects Dialogue references; desktop diagnostics expose model existence; authoring previews accept an optional `previewNode` deep link; and Playwright proves the Blue Frame orbit, classroom, and still canvases plus dedicated epilogues at desktop and 390px mobile sizes.
- Added transport-independent 3D scene framing. A tested primary-content clustering domain excludes disconnected export remnants, character models retain contain framing, environment models use cover framing on portrait viewports, and bounded framebuffer probes expose signatures, color variation, content bounds, motion, and non-background evidence for automated visual acceptance.
- Added deterministic `workflow_choice_selections` to Quality Suite scenarios. Shared validation bounds run and choice maps, complete Quality execution injects each selection into provider-free Workflow previews, reports the selections that actually ran, and can aggregate multiple branch runs into exact union coverage without a model provider or desktop state.
- Added a bounded UTF-8 one-shot MCP client and a fingerprinted binary project-asset importer. The MCP client performs the real initialize/call handshake without PowerShell code-page loss; the importer plans before writing, requires an unchanged source/destination precondition and plan fingerprint, validates GLB 2.0 structure, rejects symlinks/traversal/case aliases, and stages replacement atomically.
- Moved Knowledge normalization, bounded document loading, strict field/size/catalog rules, duplicate and relation-closure validation, deterministic structured issues, catalog fingerprints, and normalized runtime construction from Tauri into `llm-authoring`. Desktop saves/hot reloads and Agent candidate acceptance now consume the same source, while real MCP stdio coverage proves invalid importance, non-canonical tags, and missing related entries roll back. The runtime model now preserves creator-defined categories and legacy `relatedEntries`; replacement indexing, search ties, and public Knowledge lists are deterministic, with headless, game, Tauri, policy fault-injection, and transaction tests protecting the boundary.
- Moved Dialogue normalization, bounded text/prompt/catalog rules, LLM prompt requirements, character references, and relationship validation from Tauri into `llm-authoring::dialogue_validation`. Six transport-neutral tests cover canonicalization, alias rejection, references, deltas, runtime-topology gaps, bounded deterministic evidence, and valid documents; desktop saves delegate to the shared result while core-runtime and MCP transaction acceptance now reject the same structured Dialogue issues, including non-canonical fields, empty text, and missing LLM prompts that the graph loader alone accepts.
- Added `preview_workflow` as the thirteenth schema-backed `monogatari-mcp` tool. It delegates exact bounded Workflow loading, original-byte SHA-256 provenance, project Story Event validation, and deterministic provider-free execution to `llm-authoring`, while accepting optional environment, run-context, choice, step, seed, and injected-random inputs. Shared headless tests and a real MCP stdio child-process flow prove source-bound branch and coverage evidence without project writes or model-provider access; the Agent Skill now requires direct Workflow preview before Quality and package gates.
- Unified Quality Suite source loading in `llm-authoring`: exact portable catalog paths, the 2 MiB suite limit, typed parsing from decoded JSON, original-byte SHA-256 provenance, deterministic summaries, and distinguishable missing-source errors now share one headless boundary. Tauri delegates project listing/loading while retaining only the built-in fallback, and MCP executes the same loaded document without JSON reserialization; eight focused authoring tests, real MCP stdio coverage, 171 desktop compatibility tests, and release ownership policy protect the boundary.
- Extracted repository credential-pattern scanning, frontend UI text artifact detection, and locale shape/key/mirror coverage from `verify-release.mjs` into two importable policies with injected filesystem boundaries and structured evidence. Eleven Node tests cover the checked-in repository, bounded file scanning, exclusions, findings, read/discovery failures, locale parse/shape/value/mirror drift, boundary validation, and release-runner delegation; the release entry point now retains only sequencing and presentation for these checks.
- Moved bounded Workflow discovery, compatible project-scoped path normalization, symlink-aware nested directory creation, catalog validation, and atomic save/load from Tauri into `llm-authoring::workflow_documents`. Six headless tests cover sorted discovery, traversal and size bounds, nested persistence, rejected replacement preservation, and portable case aliases; desktop commands and installed-runtime verification now delegate to the same transport-neutral document boundary.
- Extracted Story Playtest text timing from `GameView.vue` into a Vue-independent controller with grapheme-safe multilingual segmentation, deterministic typewriter progression, manual line completion, dynamically rechecked autoplay, reentry cancellation, and explicit disposal. Five scheduler-level tests cover text, autoplay, replacement, and cleanup; a clock-controlled Playwright flow proves one click completes the current line before the next click advances the dialogue.
- Extracted Workflow canvas pointer lifecycle from `WorkflowEditor.vue` into a Vue-independent controller with immutable drag/connection updates, bounded geometry, hit testing, interaction reentry cancellation, and explicit disposal. Four direct tests cover clamping, graph updates, invalid connections, and listener cleanup; a real Playwright gesture now drags a node and connects a newly authored node. The same browser test exposed and fixed the output port being clipped out of its hit area by the node body.
- Centralized live and headless Workflow execution policy in `llm-authoring`, including report/step models, bounded traversal limits, node transitions, trace coverage, configuration parsing, score metrics, and weighted branch selection. Tauri now owns only live runtime effects and delegates deterministic decisions to the same independently tested policy used by provider-free previews; the prior `workflow_preview` report path remains source-compatible through re-exports.
- Added private staged `.monogatari` runtime validation to `monogatari-mcp`: `validate_project_package` accepts one fixed-directory file name, extracts through the shared bounded reader into a process-owned temporary root, runs complete shared core-runtime and delivery acceptance, returns `monogatari-mcp-package-validation/v1` evidence, and removes staging on success or rejection. Real stdio tests cover valid re-imports, structurally valid packages with broken runtime references, damaged archives, traversal, and missing directory configuration; Tauri package import now uses the same delivery validator before its non-overwriting destination commit.
- Added read-only fixed-root `.monogatari` inspection to `monogatari-mcp`: `inspect_project_package` accepts one portable file name inside the startup-fixed external package directory, delegates all archive verification to the shared headless reader, and returns `monogatari-mcp-package-inspection/v1` evidence. Real stdio tests prove unavailable-directory and traversal rejection, successful verification of an exported package, and damaged-archive rejection without extraction or project writes.
- Extracted bounded `.monogatari` inspection and empty-root extraction from Tauri into `llm-authoring::project_package`: one shared reader now enforces regular unencrypted ZIP entries, strict UTF-8 portable paths, entry/file/JSON/manifest/expanded-size limits, declared inventory, SHA-256/MD5 checksums, valid JSON, credential-free settings, and `create_new` staged writes. Six headless tests cover generated-package round trips, tampering, undeclared/duplicate entries, secrets, extraction roots, extensions, and special entries; Tauri now retains only native path orchestration, runtime reload, cleanup, and non-overwriting commit.
- Added fixed-root project-package automation to `monogatari-mcp`: `preview_project_package` returns a credential-free manifest and content fingerprint without writing, while `export_project_package` requires `--allow-write`, a startup-fixed external `--package-output-dir`, one portable file name, and the freshly reviewed fingerprint. Real stdio tests prove read-only refusal, traversal rejection, stale-fingerprint rejection, create-only defaults, explicit replacement, and actual `.monogatari` ZIP output through the shared headless writer.
- Extracted staged `.monogatari` ZIP writing from Tauri into `llm-authoring::project_package`, with explicit create-versus-replace policy, canonical source/destination boundaries, fixed-buffer file streaming, in-flight SHA-256/MD5 revalidation, synchronized archives, rollback-safe replacement, and three headless tests; desktop commands now delegate output through the same transport-neutral package domain.
- Moved deterministic project export inventory, credential-free settings bytes, streaming MD5/SHA-256 hashing, content/category summaries, and manifest generation into `llm-authoring::project_package`; export and import now share one bound/path/fingerprint policy, generated manifests self-validate, portable case aliases fail before packaging, and five headless tests replace the former desktop-only manifest test while Tauri injects only runtime and build provenance.
- Moved the `.monogatari` manifest schema, inventory limits, deterministic package fingerprinting, file/directory topology checks, and portable cross-platform path rules from Tauri into `llm-authoring::project_package`. Pure protocol tests now run without Tauri, while desktop commands retain application-state provenance and transactional runtime import orchestration.
- Extracted the 22-route frontend manifest, structural router/sidebar parsers, component presence checks, full-screen shell rules, and navigation locale/badge policy from the release entry point. Six independent automation tests now prove the checked-in contract, direct and lazy route parsing, malformed-object isolation, duplicate routes, missing views, sidebar violations, and release-runner delegation.
- Extracted shared CSP baselines plus Netlify-style `_headers`/`_redirects`, Azure Static Web Apps, and Vercel hosting validation from the release entry point into a fail-closed Web hosting policy module. Six independent automation tests now cover valid multi-provider contracts, CSP metadata parsing, unsafe directives, incomplete headers, external rewrites, SPA fallback ordering, malformed configs, and orchestration delegation.
- Extracted Web/PWA preview process ownership, bounded diagnostics, port allocation, route requests, and project-content response evidence from the release entry point into an importable verifier. Six independent automation tests cover base-path URL construction, successful route/content evidence, actionable HTTP failures, early process exit, dependency boundaries, and release-runner delegation; startup failures and readiness timeouts now include current bounded Vite output.
- Extracted root/subpath Web/PWA distribution inspection, hosting-policy delegation, manifest/inference/service-worker checks, and complete project-asset inventory comparison from the release entry point into a filesystem-bounded evidence module. Four independent automation tests cover asset URL policy, missing/malformed distribution evidence, explicit roots, and release-runner delegation; malformed hosting JSON and missing source content now fail with structured issues.
- Extracted Tauri desktop packaging, mobile preflight, installed-runtime, command-registration, shared-authoring, and safety contract inspection from the release entry point into an explicit-root evidence collector. Four independent automation tests prove the checked-in repository, injected multi-issue config drift, boundary validation, and release-runner delegation; the integrated runner now turns structured `issues/targets/iconCount` evidence into the product gate.
- Split Tauri product identity, mobile preflight, bundle resources, icons, installer targets, CSP, and Windows package policy into its own explicit-root evidence module. Three direct tests cover the checked-in package, simultaneous config/mobile drift, and missing boundaries, while the parent Tauri collector now aggregates this subdomain without owning its files or rules.
- Split installed-runtime verification and Windows installer audit policy into an explicit-root Tauri submodule with 32 named requirements. Three direct tests prove checked-in evidence, simultaneous desktop startup/runtime-flag/Authenticode drift, and missing boundaries; the parent collector now aggregates installation issues without owning installer-verifier sources or rules.
- Replaced 24 scattered Tauri command-registration string checks with a fail-closed declaration/handler set verifier. It discovers every Rust source, proves all 111 `#[tauri::command]` declarations are registered exactly once, rejects unparsed macros plus missing/undeclared/duplicate handler entries, and separately verifies the native dialog plugin and capabilities; five direct tests cover the complete repository, parser boundaries, injected set drift, permission drift, and explicit roots.
- Split single-chat safety traces, shared provider-independent conversation scoring, multilingual prompt guards and fallback scoring, Tauri facade/model boundaries, and group-chat transcript safety into an explicit-root conversation policy. It reports five requirement groups totaling 79 checks; direct tests inject shared-model duplication, facade drift, Chinese guard/scoring loss, and transcript-boundary loss while the parent collector retains only evidence aggregation.
- Split shared Quality inputs, deterministic provider-free Workflow preview, shared Workflow execution decisions, headless Quality execution, structured runtime traces, and thin Tauri adapter evidence into an explicit-root policy. Its five groups enforce 87 source-owned requirements plus eight structural boundaries; direct tests inject model/parser duplication, desktop-state leakage, execution-policy duplication, execution drift, trace drift, and workbench adapter loss while the parent collector retains only Story Event integration use of the Tauri adapters.
- Split Tauri git-build provenance and the pinned Rust release toolchain into an explicit-root policy with eight named requirements and a fail-closed test-profile environment check. Direct tests cover the checked-in contract, simultaneous build/toolchain/release-script drift, and missing filesystem roots; the parent collector no longer reads build or toolchain sources.
- Split project activation/state isolation and project settings/export/package delivery into two explicit-root policies. The runtime policy owns 53 loading, mutation, project-scoped storage, and legacy-content requirements plus three `current_dir()` guards; the package policy owns 105 secret-scrubbed export, bounded streaming ZIP, manifest, portable-path, inspection, extraction, and Tauri adapter requirements plus two ownership guards. Independent tests inject drift across every group, and the parent collector drops 158 rules and 24 exclusive source reads.
- Split the final 93-rule Story Content domain into an explicit-root Tauri policy covering catalog, event runtime, Dialogue, Knowledge, Scene, Ending, and cross-runtime contracts. Direct tests inject drift across every group, while the parent packaging collector is now a pure evidence orchestrator with no source-specific rules or filesystem reads.
- Extracted release-time Story Event and Ending catalog validation into an explicit-root project-content policy. It owns cross-root parity, portable IDs, action normalization, content references, stable rule/catalog fingerprints, and the Rust pinned-fingerprint contract; direct tests inject catalog, Ending, mirror, and source-pin drift while Workflow and Quality consume only its catalog loader.
- Extracted release-time Dialogue catalog validation into an explicit-root project-content policy. It owns field allowlists, portable IDs, graph reachability, character and relationship references, transition bounds, and cross-root parity; the portable content ID predicate is now a pure shared module used by Dialogue and Story Event policies, with direct graph-drift and boundary tests.
- Extracted release-time Workflow validation into an explicit-root project-content policy with a directly callable pure shape validator. File discovery and Story Event catalog injection remain in the I/O policy, while node types, required fields, state keys, conditions, Event scope, and connections are independently tested without the release runner.
- Extracted release-time Renderer Asset validation into an explicit-root project-content policy. It owns character renderer fields, scene backgrounds, project-relative path containment, supported extensions, checked-in asset existence, GLB structure and SHA-256 evidence, and license attribution; independent fault-injection tests keep binary and attribution failures actionable even when the model fixture is damaged.
- Extracted release-time Knowledge reference validation into an explicit-root project-content policy. It owns knowledge IDs, per-root duplicate detection, legacy character reference aliases, reference-item normalization, and pinned-reference existence evidence; independent tests inject malformed records, missing IDs, duplicates, invalid aliases/items, and missing references without invoking the integrated runner.
- Extracted release-time Quality Suite source validation into an explicit-root project-content policy with reusable pure shape and default-baseline validators. It owns suite discovery, score bounds, expectation conflicts, the 29-scenario safety/knowledge/Workflow baseline, and Story Event fingerprint parity through an injected catalog policy; six independent tests cover checked-in evidence, malformed shapes, cross-group baseline drift, dependency injection, boundaries, and release-runner delegation.
- Extracted repository-wide JSON discovery and parsing from the release entry point into an explicit-root evidence policy backed by one shared deterministic file walker. Generated, dependency, build, and VCS directories remain excluded by default; six independent tests cover the checked-in 234-file inventory, parse/read/discovery failures, traversal order, exclusions, callable boundaries, and runner delegation.
- Replaced Playwright's shell-owned development server with an in-process Vite lifecycle. Global setup now owns startup and idempotent teardown, including cleanup after partial startup failures, so browser verification exits cleanly on restricted Windows runners without relying on external process-tree termination; Vite is resolved only when the default server actually starts, keeping dependency-free automation contracts importable before frontend installation.
- Replaced the legacy SDL archive verifier's runner-dependent `Get-FileHash` call with a stream-owned .NET SHA-256 helper and an independently scheduled PowerShell unit test, so clean Windows CI can verify pinned downloads before the native runtime, build, and renderer tests execute.
- Made module command adaptation use the requested target platform's path semantics, so Linux automation can verify Windows npm/npx launch specifications without producing host-shaped paths.
- Added an explicit legacy SDL renderer runtime mode boundary: interactive behavior keeps audio, visible resizable windows, acceleration, and VSync, while headless mode uses a hidden dummy-driver window and software renderer. The Windows integration suite now initializes the real product `WindowManager`/`RenderContext`, renders and presents three frames with primitives, polls events, checks SDL errors, and disposes native resources.
- Extracted release-channel and release-manifest source policy validation from the integrated release runner into an importable, fail-closed verifier with independent automation tests for the checked-in contract, stable-channel failures, missing manifest evidence, and malformed inputs; the module matrix and full release gate now share one automation-test aggregation entry, while the release runner owns only file loading and orchestration for this domain.
- Hardened the shared `llm-authoring` JSON replacement transaction against Windows-path aliasing: every replacement now rejects an existing filename that differs only by ASCII case before any temporary file or backup mutation, with independent core coverage and Scene, Dialogue, and Ending command regressions proving original documents remain unchanged.
- Replaced the Scene Asset workbench's two-item browser sample with a project-backed catalog derived from `sceneAuthoring`; extracted desktop/browser transport, active Scene persistence, filtering, metrics, bounded history, failed-preview tracking, and byte presentation into independently tested modules, with Playwright proving full catalog visibility and active selection across reloads.
- Extracted Scene draft cloning, filtering, tag parsing, case-folded ID allocation, warning evidence, and catalog diagnostics from `SceneEditorView.vue`; browser persistence now rejects Windows-path-aliasing Scene IDs, with independent domain tests and a Playwright flow that saves a real project background, opens author Playtest, and verifies collision refusal.
- Extracted Knowledge Base form conversion, proxy-safe Metadata cloning, filtering, taxonomy, ID allocation, dirty snapshots, and stable validation evidence from `KnowledgeBaseView.vue`; browser deletion now resolves the unified character catalog so knowledge pinned by browser-authored character drafts cannot be removed, with independent domain/transport tests and a real cross-editor Playwright workflow.
- Extracted Story Ending draft cloning, filtering, case-folded ID allocation, project-reference evidence, and Story Event unlock coverage from `EndingEditorView.vue`; browser persistence now rejects file-aliasing ID collisions, save normalization is shared, and independent unit plus browser save/preview coverage verifies a real Scene/Dialogue-bound Ending route.
- Extracted shared proxy-safe JSON cloning/parsing plus immutable Story Event document, trigger gate, character scope, typed action, Metadata, filtering, metrics, and stable validation behavior from `StoryEventEditorView.vue`; Metadata-only edits now participate in dirty-state protection, reactive drafts duplicate/save without `structuredClone` failures, selection survives sorted saves, and Vitest plus browser persistence coverage lock the workflow down.
- Extracted proxy-safe Dialogue draft cloning, portable case-folded ID handling, graph ordering, immutable node/edge/choice/relationship transformations, and dirty-state shaping from `DialogueEditorView.vue` into an independently tested domain; browser persistence now rejects portable ID collisions, and Playwright authors, renames, connects, saves, and plays a real two-node graph.
- Extracted Workflow execution-evidence indexing, optional numeric parsing, score/coverage formatting, scalar choice handling, typed Story Event decisions, and canvas node outcome presentation from `WorkflowEditor.vue`; missing scores no longer render as a real `0.00`, non-scalar evidence cannot leak as object labels, and independent Vitest plus desktop/mobile Playwright coverage proves deterministic traces map back onto the authored graph.
- Extracted character form contracts, runtime/form conversion, validation, knowledge-reference handling, payload normalization, dirty-state snapshots, filtering, and sprite defaults from `CharacterEditorView.vue` into an independently tested pure domain; removed the form's broad index-signature escape hatch, rejected case-folded portable ID collisions before they can overwrite Windows project files, and extended browser authoring E2E coverage.
- Extracted exact Settings transport contracts plus pure config construction, browser preview/sync state, nested path protection, runtime-secret scrubbing, manifest shaping, filename, and byte-formatting behavior from `SettingsView.vue`; added 11 isolated domain tests, release architecture invariants, and a desktop/mobile Playwright flow that parses the exported browser manifest and proves runtime credentials are absent.
- Extracted exact Quality report contracts, generated browser preview evidence, filtering/diagnostic/export presentation logic, and their isolation tests from `QualitySuiteView.vue`; the view now owns only localization, reactive orchestration, Tauri transport, and browser downloads, while release invariants reject duplicate contracts or static preview reports and Playwright verifies the 29-scenario desktop/mobile workbench.
- Extracted Workflow transport contracts plus pure node creation, layout, connection, document, path, and browser fallback-catalog behavior from `WorkflowEditor.vue` into independently tested frontend modules; centralized the authoritative Rust node catalog in `llm-authoring`, reduced Tauri to delegation, corrected offline `dialogue`/`llm_generate` fields and typed controls, and added release-time catalog parity checks.
- Added the read-only, schema-backed MCP `run_quality_suite` tool over the shared headless executor, with fixed-root catalog containment, exact source fingerprints, caller provenance, complete scenario/audit evidence, and real stdio coverage for actionable failures.
- Moved complete Quality Suite execution into `llm-authoring`, including deterministic scoring, prompt/response guards, runtime safety traces, Story Event decisions, knowledge evidence, Workflow coverage, expectation failures, audit aggregation, and caller-supplied provenance; Tauri now only loads suites, supplies build metadata, and delegates.
- Extracted deterministic, side-effect-free Workflow preview execution into `llm-authoring`, including typed environment snapshots, bounded traversal, injected/seeded random branches, stateful Rhai conditions, Story Event decisions, simulated provider-free LLM nodes, and independent tests; Tauri now adapts desktop state while Quality coverage executes headlessly.
- Unified Quality Suite input documents, scenarios, expectations, messages, and typed Workflow run contexts under `llm-authoring`; Tauri now reexports those contracts instead of deserializing a duplicate schema, with headless malformed-context tests and release invariants.
- Extracted conversation messages, evaluation scores, safety traces, deterministic multilingual fallback scoring, guarded relationship deltas, and Story Event decisions from Tauri chat commands into a tested `llm-authoring::conversation_quality` domain for headless Quality and MCP reuse.
- Moved the pure multilingual prompt and response guard domain plus its tests from the Tauri command crate into `llm-authoring`, leaving a desktop compatibility facade so Agent transports and future headless Quality execution can reuse the exact runtime safety semantics.
- Moved MCP cross-process project leases into a path-private system temporary namespace so read-only Agent inspection no longer creates sidecar files in authored projects, with unit, real-stdio, and release-invariant coverage for reader sharing and writer exclusion.
- Added a tested `sync-project-mirror.mjs` check/write workflow and release/desktop-build gates so canonical `data/` and the desktop-packaged `rust-engine/data/` can no longer silently drift while Agent-authored projects are mirrored, while transient MCP lease files stay out of packaged resources; also added UTF-8 MCP guidance for Windows PowerShell 5 clients.
- Added a developer-test handoff that records the independently passing 17-module matrix, integrated gate, human and Agent/MCP workflows, acceptance semantics, and environment-dependent release boundaries.
- Extracted project-package manifest models, schema/version checks, inventory bounds, checksum syntax, deterministic fingerprints, sorting, and file/directory topology validation into a pure independently tested module consumed by ZIP I/O.
- Extracted project-package portable path rules into a pure module with independent tests for nested paths, case-folded keys, parent expansion, traversal, reserved Windows names, and platform-specific separators.
- Split project-package Tauri command orchestration from the archive core, isolating application state, blocking-task dispatch, runtime reload validation, and staged import commit from ZIP and manifest mechanics while preserving command names.
- Added independently runnable Playwright Chromium coverage for workspace navigation, validated character browser-draft persistence, and dialogue authoring-to-Playtest handoff, integrated into the module matrix and release gate.
- Rejected project-package file/directory topology conflicts during manifest inspection, including case-folded exact collisions and files used as parent directories, with regression coverage before extraction can create staged content.
- Added shared delivery validation and the read-only MCP `validate_delivery` tool, reporting nested core evidence, declared/existing renderer and scene-audio assets, missing or unsupported declarations, and intentional placeholder character usage without claiming rendered visual quality.
- Added the read-only schema-backed MCP `validate_project` tool so Agents can obtain the same structured headless runtime, catalog, Workflow, and Quality evidence before and after edits without requiring a write transaction.
- Added shared headless Quality Suite document validation with bounded safe loading, expectation range/conflict checks, and character/knowledge/Event/Workflow references; Tauri delegates shape validation and MCP rolls invalid Quality writes back before acceptance.
- Extracted Workflow models and pure graph/Event validation from the Tauri execution adapter into `llm-authoring`; Agent candidate validation now loads bounded recursive Workflow catalogs, rejects duplicate IDs and broken scene/character/sub-workflow references, and rolls invalid MCP Workflow writes back atomically.
- Moved the complete Story Event domain into `llm-authoring`, leaving Tauri as a compatibility facade; core Agent validation now loads the same versioned catalogs, validates scoped characters and unlock targets against real scene/dialogue/ending catalogs, and rolls back invalid MCP Event writes.
- Extended shared Agent candidate validation through strict bounded scene and ending catalogs, background-inferred scene IDs, and ending-to-scene/dialogue references; Tauri now reuses the same authoring models/loaders, and real MCP stdio tests prove invalid ending writes roll back atomically.
- Added reusable `llm-authoring` core-runtime project validation that loads the real character, dialogue, and knowledge managers, checks duplicate IDs plus character/knowledge/dialogue references, returns deterministic machine-readable evidence, powers both Tauri project loading and MCP candidate transactions, and rolls back MCP writes that fail runtime references; removed the obsolete duplicate Sakura fixture and the unused `llm-game -> llm-ai` dependency.
- Unified authored dialogue conditions across Rust and browser Playtest: false choices are hidden with stable original indices, false nodes follow required linear fallbacks with cycle detection, dialogue scripts and variables feed later conditions, unsupported browser expressions stop explicitly, failed Rust conditions/scripts roll back cursor and local state, and Rust now delegates legacy-compatible dialogue scripts to the bounded shared Rhai engine.
- Added a pure browser Story Playtest dialogue state machine with explicit graph errors and immutable relationship updates; desktop choice execution now preflights every relationship target, guards the inspected source node, applies authored deltas through CharacterManager clamping, and tests both runtimes.
- Extracted the browser workflow validator and preview executor from `WorkflowEditor.vue` into a pure, injected `workflowPreview` domain module with deterministic random-branch tests, run-context normalization, stateful condition/event simulation, useful scene/narration traces, and explicit refusal to misroute unsupported conditions; release source-invariant checks now live in an importable verifier module.
- Added the official-SDK `monogatari-mcp` stdio server with five schema-backed tools, a startup-fixed project root, read-only default, reviewed transaction fingerprints, shared/exclusive process leases, rollback, real child-process protocol tests, and a release-built binary; candidate application now reaches core-runtime acceptance for characters, dialogue, and knowledge.
- Added Vitest and Vue Test Utils coverage for frontend authoring validators, renderer fallback selection, story access derivation, Pinia async command state, and shared component interactions/accessibility, with an independently runnable `frontend-unit` CI and release gate.
- Added `monogatari-agent-project-transaction/v1` to `llm-authoring`, providing deterministic dry-run plans, JSON-only catalog allowlists, exact create/update/delete preconditions, bounded payloads, case-collision protection, multi-file staged candidate validation, reverse-order rollback, structured results, cleanup warnings, and stable machine-readable errors for future MCP transports.
- Added the transport-neutral `llm-authoring` crate with independently tested atomic content rollback, strict portable project paths, project settings diagnostics, credential scrubbing, and atomic persistence; Tauri project/config and catalog commands now delegate these filesystem rules through thin adapters.
- Added the repository-level `$author-visual-novel` Skill so agents can author canonical Monogatari characters, knowledge, scenes, dialogue, events, endings, workflows, and Quality Suites against the same runtime and release contracts as human authors.
- Added a versioned module verification matrix, tested selector/runner, machine-readable reports, expanded CI jobs, and explicit audit gaps for independently proving the automation, frontend, Rust, and legacy .NET implementation surfaces.
- Added pinned, SHA-256-verified official SDL2 runtime preparation plus warnings-as-errors solution builds, restoring independent build evidence for the retained Windows x64 .NET application instead of relying on missing repository-local DLLs.
- Moved SDL2 runtime ownership from the legacy executable into the Renderer module and added a Windows native ABI/license regression that loads SDL2, SDL2_image, and SDL2_ttf and verifies their required exports.
- Added all-target Rust workspace Clippy to the pinned toolchain, module matrix, CI, and integrated release gate.
- Added a versioned inference backend planner that separates host detection, setup, exact-model probes, known blockers, and ready backends across WebGPU, llama.cpp, WinML GenAI, DirectML ONNX, MLX-LM, vLLM, SGLang, and OpenAI-compatible services.
- Added Settings diagnostics for backend recommendation, next probe, stable reason codes, and readiness states without treating model initialization or API configuration as successful generation.
- Added a verified inference backend matrix with Qwen3.5 0.8B WebGPU and Linux llama.cpp evidence, reproduced Windows WinML/DirectML blockers, and staged CUDA, Vulkan, Metal, ROCm, Intel, and MUSA adaptation gates.
- Added a licensed animated glTF 2.0 Fox fixture, release-time GLB/hash/attribution verification, arbitrary-unit model normalization, responsive camera framing, and deterministic canvas state for desktop/mobile visual probes.
- Allowed same-document `blob:` fetches in Web/PWA and Tauri CSP so Three.js `ImageBitmapLoader` can decode embedded GLB textures without silently falling back to an untextured mesh.
- Added a Windows installer audit that validates exact MSI/NSIS identity and version metadata, the pinned MSI upgrade identity, installer SHA-256 hashes, size bounds, MSI database properties, Authenticode status for installers and the extracted application, signer identity, and release-channel signing policy.
- Added MSI administrative-image verification that compares every bundled `data/` path, size, and SHA-256 hash with the checked-in project before running the extracted production executable.
- Added `--verify-installation <absolute-report.json>` headless production-binary verification for bundled settings, runtime characters/dialogues/knowledge/events, scenes, endings, workflows, locales, Quality Suites, complete project inventory, content fingerprint, and build Git provenance.
- Added release-gate coverage for installed-runtime verification and an explicit internal-only unsigned installer audit path; stable and beta audits still require valid Authenticode signatures.
- Pinned the Rust release toolchain to `nightly-2026-07-03` and removed the Windows test-profile override that triggered a compiler ICE, keeping full Tauri test compilation reproducible.
- Added complete `.monogatari` ZIP project packages with native cross-platform save/open dialogs, embedded versioned manifests, deterministic project inventories, sanitized `settings.json`, and exact content/category fingerprints.
- Added fixed-buffer streaming export, hashing, ZIP writing, and package verification for portable paths, case-insensitive collisions, file/count/expanded-size limits, regular ZIP entries, JSON syntax, per-file SHA-256/MD5 checksums, and whole-package fingerprints.
- Added transactional project import into a new non-overwriting directory, followed by project config, character, dialogue, knowledge, event, scene, and ending reload validation before the staged directory is committed.
- Added project-package regression coverage for checked-in runtime round trips, traversal and ZIP-bomb declarations, content tampering, secret removal, stable destination naming, and failed-export rollback that preserves an existing package.
- Hardened project settings persistence with a 1 MiB bound, shared atomic replacement/rollback, regular-directory enforcement, and rejection of non-regular or symlinked `settings.json` paths and export directories; handoff manifests no longer expose the author's absolute project path.
- Added project-backed Scene and Dialogue authoring catalogs with stable content/catalog fingerprints, optimistic concurrency, shared rollback-capable JSON transactions, dirty-draft guards, browser-local catalogs, and Story Mode author preview.
- Added strict dialogue document and graph validation for unknown fields, authoritative map-key IDs, transition targets, reachability, character references, choice relationship deltas, conditions, scripts, terminal metadata, and LLM prompts, plus post-save runtime hot reload.
- Added cross-catalog deletion protection: scenes scan Story Events, endings, and workflows; dialogues scan Story Events and endings; inferred scene metadata deletion never removes source background assets.
- Added release-gate validation for complete checked-in dialogue graphs and parity between root and Rust project data catalogs.
- Fixed the Whispering Leaf dialogue branch that targeted the missing `ending_alive` node.
- Added a shared story-content access layer: only scene/dialogue/ending IDs referenced by typed unlock actions are gated, while all other project content stays backward-compatible and open. Story Mode, dialogue starts, real workflow scene changes, and ending launches enforce the same decisions.
- Added the Story Event authoring workbench with catalog search, trigger thresholds, character scope, typed action editing, local validation, optimistic fingerprint concurrency, and rollback-safe single-document saves.
- Added versioned `monogatari-story-ending/v1` assets and a gated ending launcher that resolves a real scene and dialogue before playback.
- Added the Ending Route authoring workbench with real scene/dialogue association, event coverage diagnostics, stable catalog fingerprints, atomic save rollback, event-reference deletion protection, browser drafts, and player-gate-free author preview.
- Added project-backed Story Library browsing for scenes, dialogues, and endings. Web/PWA builds now package and offline-cache these catalogs and run checked-in branching dialogue nodes in-browser.
- Fixed checked-in runtime content compatibility by canonicalizing the legacy example dialogue, deriving omitted dialogue node IDs from map keys, and accepting both numeric and `{ score, type }` character relationship values.
- Added typed `unlock_scene`, `unlock_dialogue`, `unlock_ending`, and `set_flag` story-event actions with bounded validation, legacy `data` action migration, action-bound catalog fingerprints, and shared chat/workflow execution.
- Added project-scoped `monogatari-story-progress/v1` state with idempotent unlock sets, per-character application counts, action audit reports, progress fingerprints, frontend runtime diagnostics, and side-effect-free author previews.
- Added backward-compatible `monogatari-game-save/v3` snapshots for story progress, including v1/v2 triggered-event migration and atomic rejection of malformed progress payloads.
- Added versioned project story event catalogs under `events/*.json`, with bounded parsing, duplicate/schema/threshold validation, character scope, repeat behavior, configured-path containment, stable rule/catalog fingerprints, and legacy-project fallback behavior.
- Unified live chat, manual scoring, Workflow trigger execution and validation, Quality Suites, and Web/PWA workflow previews on the active project event catalog, replacing duplicated hardcoded trigger rules.
- Added catalog-backed Workflow event menus, atomic event hot reload commands, Web/PWA event packaging/offline caching, project export/release-manifest event categories, and release-gate coverage for event assets and workflow references.
- Added backward-compatible runtime snapshots covering scene history, dialogue cursor/local state, typed Rhai variables, character emotion/relationships/full memory, chat history, evaluations, safety traces, and triggered-event state.
- Added stable quick-save and auto-save slot IDs, complete runtime save/restore regression coverage, legacy v1 compatibility tests, and release-gate invariants for the persistence contract.
- Added staged stable-slot replacement with backup recovery and a 32 MiB save payload limit so interrupted or oversized save operations fail without silently corrupting the active slot.
- Added staged project content loading and runtime reset tests so same-root reloads and project switches replace old managers instead of merging characters, dialogue, knowledge, chat, scene, or script state.
- Added a configurable offline quality suite for character stability, prompt-injection resistance, relationship and fallback scoring side-channel containment, memory-poisoning resistance, memory prompt replay safety, tool-role injection containment, identity drift, style drift, real knowledge-reference anchoring, evaluation-summary safety, workflow output safety, workflow tool-call containment, workflow branch coverage, private reasoning leakage, fallback scoring, overrange score clamping, story-event trigger/idempotence regression, and event-rule snapshot checks.
- Added stable SHA-256 event-trigger rule fingerprints to live chat decisions, manual scoring reports, Quality Suite reports, and the checked-in event-rule snapshot so story unlock rules can be audited across release builds.
- Added block-level prompt-control sanitization for Tauri, shared Rust AI, and legacy C# prompt builders so explicit XML, Markdown fence, and comment-wrapped role-control payloads are omitted along with their opening markers.
- Added a checked-in block-body prompt-injection quality scenario proving XML, Markdown fence, and comment-wrapped role-control payloads cannot boost scores, poison memory, or trigger story events.
- Added Quality Suites workbench guard-note summaries and export evidence so author QA reports include runtime safety trace guard note counts alongside category, failure, safety-signal, and workflow coverage summaries.
- Added Quality Suite run metadata so QA exports identify the engine version, generated timestamp, scenario count, and pass rate.
- Added build git commit metadata to Quality Suite QA reports so exported evidence can be tied back to a specific source revision.
- Added backend-confirmed suite source paths to Quality Suite QA reports so exports identify the exact regression suite that was executed.
- Added SHA-256 suite content fingerprints to Quality Suite QA reports so exported evidence can be matched to the exact regression suite contents.
- Added SHA-256 suite content fingerprints to the Quality Suites list so authors can verify the selected regression suite before running it.
- Added checked-in Quality Suite source evidence to release manifests, including suite paths, scenario counts, categories, and SHA-256 fingerprints.
- Added aggregate Quality Suite set fingerprints to release manifests so each release can verify the exact checked-in regression suite set used for QA.
- Added checked-in workflow source evidence and aggregate workflow-set fingerprints to release manifests so blueprint-style story fixtures can be audited with release artifacts.
- Added checked-in project content source evidence and aggregate content-set fingerprints to release manifests so bundled characters, dialogue, knowledge, scenes, and sample assets can be audited with release artifacts.
- Added per-category project content source fingerprints to release manifests so release audits can isolate bundled character, dialogue, knowledge, scene, and asset changes.
- Added SHA-256 checksums to project export file inventories while retaining legacy MD5 checksums for compatibility.
- Added whole-package SHA-256 content fingerprints to project export manifests so package handoffs can be verified as a single deterministic bundle.
- Added content category summaries and explicit package fingerprint algorithm metadata to project export manifests for faster commercial package audits.
- Added per-category project export fingerprints so commercial package audits can isolate character, dialogue, knowledge, scene, asset, workflow, locale, and quality-suite changes.
- Added engine version and build commit provenance to project export manifests so package handoffs can be traced to a specific Monogatari build.
- Added finalized guarded workflow output evidence to Quality Suite reports and exports so QA can inspect the safe story text consumed after workflow LLM output sanitization.
- Added runtime renderer fallback handling so Story Mode and Character Editor skip failed Live2D/GLB/GLTF loads and continue to the next valid 3D, sprite, portrait, or generated placeholder candidate.
- Added buffered OpenAI-compatible SSE stream parsing so API streaming responses survive split JSON lines, split UTF-8 content, `[DONE]` markers, and final lines without trailing newlines.
- Added a linked Windows DirectML executor with standard tokenizer loading, causal-LM graph validation, bounded autoregressive generation, streaming output, and real backend readiness reporting.
- Added async-safe initialized backend registration so API and DirectML engines become active only after their runtime initialization succeeds.
- Added a Transformers.js WebGPU text-generation runtime for Web/PWA character and ensemble tests, plus a versioned packaged inference contract, CSP support, service-worker caching, and release verification.
- Upgraded the Web/PWA runtime to Transformers.js 4.2.0 and Qwen3.5 0.8B Text ONNX Q4, paired the embedded WebGPU module with its matching packaged Asyncify WASM binary, and verified both browser session initialization and streamed Chinese character generation on an RTX 3060.
- Added API backend runtime configuration validation so blank keys/models, unsafe base URLs, embedded credentials, and query/fragment-bearing provider URLs are rejected before a backend can become active.
- Added OpenAI-compatible SSE stream error-frame handling so provider error payloads and malformed data frames abort streaming inference instead of being silently ignored.
- Added content loader path isolation tests and release-gate invariants so character, dialogue, and knowledge reload commands resolve only under the active project content directories.
- Added character manager path isolation tests and release-gate invariants so character create/delete commands use the active or discovered default project data root, safe portable IDs, and stay inside the project characters directory.
- Added plugin manager path isolation tests, Plugin workbench command-contract checks, and release-gate invariants so plugin listing, registration, and removal use the active or discovered default project data root plus safe portable IDs and optional `.rhai` script references inside the project plugins directory.
- Added script command input limits, Rhai execution budgets, and release-gate invariants so author scripts reject hidden control characters and abort runaway loops or recursion.
- Added shared script state key validation and release-gate invariants so Rhai variables, flags, workflow state writes, dialogue scripts, and save loading use portable save-friendly keys.
- Added workflow validation for script state key fields so invalid variable and flag names are caught during authoring/import before workflow execution.
- Added a read-only Rhai condition engine so condition expressions can inspect variables and flags without mutating story state.
- Added shared condition expression validation so command inputs and workflow condition nodes reject non-string, oversized, or hidden-control-character payloads before execution.
- Added TTS provider error redaction so Azure and ElevenLabs request failures, response bodies, sensitive headers, and token-shaped values are cleaned before reaching frontend error surfaces.
- Added TTS synthesis log privacy so runtime logs record spoken-text length metadata instead of raw dialogue, prompt text, or provider-token-shaped content.
- Added frontend runtime log hygiene so production source ships without `console.log`/`console.debug` debug output and the release verifier catches regressions.
- Added frontend HTML-injection hardening so shell navigation renders icons as text instead of `v-html`, with release-gate scans for raw HTML sinks.
- Added a production Tauri Content Security Policy and release-gate checks so packaged desktop WebViews no longer ship with CSP disabled.
- Added Web/PWA Content Security Policy meta coverage and release-gate checks so static browser builds share the same hardened app shell baseline.
- Added generated Web/PWA static-hosting `_headers` output with CSP, nosniff, referrer, and permissions-policy release-gate coverage for hosts that support response headers.
- Added generated Web/PWA static-hosting `_redirects` output with asset passthrough rules and SPA fallback coverage for Netlify/Cloudflare-style static hosts.
- Added generated Azure Static Web Apps `staticwebapp.config.json` output with SPA navigation fallback and global security headers, plus release-manifest coverage for required Web/PWA hosting artifacts.
- Added generated Vercel `vercel.json` output with SPA rewrite and global security headers, plus release-manifest coverage for Vercel static deployments.
- Added project settings runtime-secret scrubbing so API keys, tokens, authorization headers, token-shaped values, query-secret assignments, and legacy persisted secret fields are omitted before `settings.json` saves or project config state returns to the frontend.
- Added read-only workflow condition context variables for relationship, evaluation scores, and evaluation count, plus matching Web/PWA preview evaluation for common condition expressions.
- Added Web/PWA workflow preview state mirroring so local `set_variable`, `set_flag`, and evaluation outputs can drive later `getVariable` and `hasFlag` conditions.
- Added Web/PWA workflow preview mirrors for relationship and emotion nodes so browser previews expose the same per-run state transitions as desktop workflow execution.
- Fixed Web/PWA workflow preview signed numeric parity so negative relationship deltas and camera offsets behave like desktop workflow execution.
- Added normalized random branch weights for desktop and Web/PWA workflow previews so weighted story branches do not collapse to the first connection or invalid negative probabilities.
- Added desktop workflow run-context state isolation so author previews can exercise variable, flag, relationship, emotion, and scene changes without mutating persistent runtime state.
- Added marketplace template path isolation tests and release-gate invariants so template import/export uses project-scoped template references instead of raw filesystem paths.
- Added Live2D model path isolation tests, renderer asset validation hardening, and release-gate invariants so model loading stays inside the active project data root.
- Added i18n locale path isolation tests and release-gate invariants so locale loading, listing, and translation use safe locale IDs inside the active project locales directory.
- Added ONNX backend config path isolation tests and release-gate invariants so local model configuration uses project-scoped model/tokenizer references and activates the ONNX engine.
- Added engine project-root validation tests and release-gate invariants so initialization binds only existing local project directories.
- Added a Quality Suites workbench view and sidebar entry for running release-gate checks from the desktop UI.
- Added Web/PWA distribution baseline with manifest metadata, offline fallback page, service worker runtime caching, and `npm run build:web`.
- Added dedicated Web/PWA install and maskable icons and release-gate checks that keep them in the manifest, app shell cache, and static-hosting dist.
- Added static-hosting preparation for Web/PWA builds, including GitHub Pages fallback assets and `VITE_BASE_PATH` subpath deployment support.
- Added mobile shell readiness verification for viewport safe-area support, iOS/PWA metadata, compact Tauri shell dimensions, and mobile navigation padding.
- Added responsive Web/PWA shell verification for built 375px mobile and 768px tablet layout signals.
- Added Tauri mobile deployment preflight verification for Android/iOS command readiness, Vite mobile host binding, and mobile release documentation.
- Added runtime trace evidence for character mind contract application and creator-pinned knowledge context anchoring, including resolved pinned knowledge ref IDs for QA audit.
- Added runtime chat story-event trigger decision evidence so authors can inspect relationship values, score metrics, evaluation counts, and blocker reasons directly from live conversations.
- Added an atomic manual scoring report command that returns conversation evaluation, matching story-event trigger decisions, and triggerable events together.
- Updated manual Chat scoring to consume the atomic scoring report for immediate author score-gate debugging.
- Aligned Quality Suite story-event reports with the same trigger decision contract used by live chat runtime responses.
- Added an explicit Web bundle budget verifier that keeps entry assets small while allowing bounded lazy renderer chunks for Three.js and Live2D.
- Added a renderer asset contract for characters with Live2D, GLB/GLTF, sprite, portrait, and generated 3D fallback support in Story Mode.
- Added a one-command release verification script covering JSON validation, all quality suite files, locale coverage, sensitive token pattern scanning, frontend UI text artifact scanning, frontend source invariants, Rust checks/tests, Web/PWA build, Web/PWA dist asset checks, frontend audit, and legacy C# tests.
- Added explainable event-trigger decisions for author tooling and quality reports, including actual relationship values, score metrics, evaluation counts, idempotence state, and blocker reasons.
- Added executable Workflow `evaluation` and `trigger_event` nodes so visual story graphs can read LLM conversation scores and drive score-aware event unlocks.
- Added executable Workflow runtime behavior for core authoring nodes: start, end, dialogue, choice, scene change, emotion change, relationship updates, and sub-workflow delegation.
- Added a guarded Workflow graph runner with execution traces, choice stop points, branch routing for conditions/scores/events, and a Run panel in the workflow editor.
- Added interactive choice selection for Workflow Run traces so authors can continue through choice branches during debugging.
- Added release-gate validation for checked-in workflow files across root and Rust data directories.
- Added a checked-in score-gate workflow fixture plus backend execution regression tests proving conversation scores can branch into score-aware story-event unlocks.
- Added score and event diagnostics to Workflow Run traces so authors can inspect evaluation metrics, thresholds, score sources, trigger decisions, and blocker reasons.
- Added Workflow canvas run badges that mark executed nodes, score pass/fail, blocked events, completed nodes, and waiting choices directly on the visual graph.
- Added a Workflow Run preview context so authors can simulate character scores, relationship values, evaluation counts, and already-triggered events while debugging score-gated story branches.
- Added frontend and Rust-side clamping for Workflow Run preview context scores/relationships before score-gated story branches consume author-simulated values.
- Added one-click Workflow preview context presets for unlock, low-score block, and repeat-trigger block scenarios.
- Added Workflow Run graph coverage summaries with executed node counts and unvisited node chips for branch QA.
- Added a Workflow Run preset matrix that executes all score-gate preview presets and merges graph coverage for branch QA.
- Added workflow command path isolation tests and release-gate invariants so backend save/load reads and writes only JSON workflows inside the active project `workflows/` directory.
- Added Quality Suite workflow coverage snapshots so release checks can prove score-gated story fixtures still cover unlock, low-score, and repeat-trigger branches.
- Added Quality Suite audit summary UI and JSON export with a stable schema marker for release QA evidence handoff.
- Added Quality Suite schema validation for score-bound ranges and contradictory expected/forbidden markers before release QA reports run.
- Added tool-role/function-call injection detection and a checked-in quality scenario proving spoofed runtime instructions cannot unlock events or alter character identity.
- Added structured role-block prompt-injection detection for XML, header, and JSON-shaped role spoofing before fallback scoring, memory, relationship, or story-event logic consumes player text.
- Added attributed XML-like role tag detection for Tauri prompt guards plus Rust and legacy C# prompt builders so `<system ...>` and `<tool ...>` prompt-control variants are omitted before role parsing.
- Added Markdown role-code-fence detection for Tauri prompt guards plus Rust and legacy C# prompt builders so backtick-fenced `system` and tilde-fenced `tool` prompt-control blocks are omitted without blocking non-role language fences.
- Added comment-wrapped role marker detection for Tauri prompt guards plus Rust and legacy C# prompt builders so HTML, C-style, and line-comment role headers are omitted before prompt assembly.
- Added punctuation-free role heading detection for Tauri prompt guards plus Rust and legacy C# prompt builders so `System Prompt`, `Developer Instructions`, and `Tool Message` headings are omitted before prompt assembly.
- Added reusable Rust AI prompt-builder boundary sanitization and release-gate `llm-ai` tests so downstream integrations cannot reintroduce role-marker prompt injection through shared prompt history or context assembly.
- Added Rust API engine secret redaction for debug output, bearer tokens, sensitive custom headers, and API error surfaces before provider credentials can leak into logs or frontend reports.
- Added legacy C# prompt-builder boundary sanitization and release-gate invariants so the retained legacy AI path cannot reintroduce role-marker prompt injection.
- Added legacy C# APIEngine error redaction for token-shaped values and JSON/header/query secret assignments so retained legacy provider failures cannot echo credentials into test or frontend error surfaces.
- Added relationship sentiment side-channel containment so prompt-injection text with positive words cannot advance relationship milestone events.
- Added fallback scoring side-channel containment so prompt-injection text cannot inflate engagement or creativity when model evaluation is unavailable.
- Added workflow tool-output containment checks proving generated node text shaped like a tool/function call is withheld before downstream story nodes consume it.
- Added memory-poisoning detection and a quality scenario proving player-authored "official canon" memory writes cannot replace creator-authored Sakura knowledge anchors.
- Added guarded character memory writes and a memory prompt replay quality scenario so stored prompt-injection text cannot re-enter future character prompts through recent memories.
- Added overrange score clamping regression coverage for above-100%, above-scale, and negative evaluator outputs before event decisions consume them.
- Added release-gate validation for frontend route, sidebar navigation, view component, and navigation locale coverage.
- Added release-gate subpath Web/PWA builds to verify static-hosting assets under `/Monogatari/` before restoring the default root-path dist output.
- Added release-gate Web/PWA preview smoke checks that start Vite preview and verify every app route returns the production SPA shell on root and subpath builds.
- Added a knowledge-boundary quality scenario and report flag to catch player-induced retcons or invented canon before they erode character knowledge stability.
- Added release-gate renderer asset contract checks for checked-in scene backgrounds and character Live2D/3D/sprite/portrait paths.
- Added Character Editor controls for emotion-specific sprite paths so creators can author Galgame expression art without editing character JSON by hand.
- Added Character Editor renderer asset diagnostics for unsupported extensions, absolute paths, external URLs, and parent traversal before assets reach the release gate.
- Added an in-editor renderer preview that mirrors Story Mode priority across Live2D, GLB/GLTF, sprite/portrait, and generated 3D fallback states.
- Added a shared frontend renderer asset selector so Story Mode and Character Editor previews use one source of truth for Live2D, 3D, sprite, portrait, and generated fallback priority.
- Added a renderer asset selector contract test to the release gate, covering fallback priority, path validation, and expression sprite resolution.
- Added real Audio Manager playback controls for BGM, ambient loops, and SFX previews with persisted track lists, path resolution across Web/Tauri builds, per-track gain, and master/channel mixer state.
- Added release-gate frontend source invariants that keep the Audio Manager tied to real audio elements, persistent mixer state, and BGM/ambient/SFX transport controls.
- Added Tauri desktop packaging metadata for Windows MSI/NSIS targets, installer icons, publisher/category descriptions, WebView2 bootstrap behavior, and bundled sample `data/` resources.
- Added release-gate validation for Tauri packaging configuration so desktop installer metadata, icons, bundled sample data, and Windows downgrade/WebView2 policy cannot drift silently.
- Added a versioned project export manifest with file inventory, per-file checksums, exportable directory coverage, and settings secret redaction for commercial package handoff.
- Added runtime chat safety trace evidence for prompt-injection detection, guarded character responses, memory guards, stream replacements, and relationship side-channel containment.
- Added runtime group chat safety trace evidence so multi-character conversations reuse the same prompt-injection, response guard, and relationship side-channel audit contract as single-character chat.
- Added Quality Suite runtime safety trace evidence and a checked-in group chat scenario proving multi-character prompt-injection attempts produce auditable guard notes.
- Added multilingual prompt-injection detection and a checked-in quality scenario for Chinese, Japanese, and Korean prompt-control attempts against score, relationship, memory, and hidden-prompt boundaries.
- Added Unicode-obfuscated prompt-injection normalization and a checked-in quality scenario for fullwidth role markers and zero-width character splitting.
- Added multilingual local fallback scoring signals and a checked-in quality scenario for friendly creative Chinese, Japanese, and Korean player text.
- Added a release artifact manifest generator with SHA-256 checksums, channel metadata, installer expectations, and code-signing readiness evidence.
- Added git source-state evidence and clean tracked worktree enforcement for final release artifact manifest generation.
- Added a checked-in release channel policy and manifest enforcement for stable/beta installer requirements, preflight exceptions, and verified installer signing evidence sidecars.
- Added release-gate validation that checked-in character pinned knowledge refs resolve to project knowledge entries across both data roots.
- Added missing Springtown lore anchors for character pinned knowledge refs so creator-declared identity and world context remain stable.
- Added checked-in portrait and sprite SVG assets for Sakura, Luna, and Kenji across Web and bundled Tauri data roots, with release-gate enforcement for core sample character renderer assets.
- Added Web/PWA dist packaging for checked-in project assets so sample backgrounds and character sprites remain reachable in static browser builds.
- Added a generated Web/PWA project asset manifest and service worker precaching so sample renderer assets are available after offline install.
- Added a restorable Chat session audit report so the latest safety trace, evaluation, story-event decisions, and triggerable events survive character switching in the author workbench.
- Added short retry handling for the release-gate frontend audit step so transient registry TLS failures do not abort otherwise passing release checks.
- Added a typed Cloud Sync status contract with project-scoped manifest analysis, pending upload/download counts, cross-device conflict evidence, Settings UI wiring, and runtime-only sync token readiness.
- Added TTS output path isolation tests and release-gate invariants so system, Azure, and ElevenLabs speech files use sanitized project `assets/tts/` filenames instead of fixed process-temp outputs.
- Added asset-manager path isolation tests and release-gate invariants so Rust and legacy C# text/JSON/binary asset reads reject absolute, URI-like, empty, and traversal-shaped paths before touching disk.
- Added save-manager path isolation tests and release-gate invariants so Rust and legacy C# save/load/delete flows reject traversal-shaped save IDs and filter mismatched save files.

### Fixed
- Browser Scene Roleplay now waits for runtime discovery before enabling free-form input and keeps a configured API turn on that provider. API failures enter the authored in-world degraded path instead of silently allocating the WebGPU/ORT model and surfacing `std::bad_alloc`; the Playwright server also drains persistent HTTP connections before Vite shutdown, and the release preview uses a tested bounded 60-second startup budget after resource-intensive build gates.
- Disabled Character creation until its asynchronous character and knowledge catalogs finish initialization, preventing a fast user or browser Agent action from being overwritten by the default-character selection.
- Restored `cargo check --locked -p llm-galgame-app` by aligning Tauri command dependencies and current core APIs.
- Rebuilt corrupted zh-CN, ja-JP, and ko-KR locale JSON files with the full 280-key i18n surface.
- Fixed frontend i18n loading so Tauri `{ locale, strings }` payloads and browser `/locales/*.json` fallback files both resolve correctly.
- Fixed the legacy C# character loader and tests so current sample character JSON maps display names, emotion, sprite paths, and nested personality traits before legacy dialogue and AI prompt tests run.
- Fixed guarded chat streaming so private-reasoning leak replacements overwrite the visible reply instead of appending to partial streamed text.
- Fixed guarded character-response replacement text so the safety fallback no longer triggers the private-reasoning leak detector it is meant to satisfy.
- Fixed workflow LLM generation so guarded outputs replace prompt-control/internal text before node results enter the story flow.
- Fixed workflow LLM node finalization so blank or guard-only generated output becomes stable failure text instead of advancing as empty story content.
- Fixed Quality Suite workflow-output checks so offline QA reuses the same guarded story-output finalization as runtime workflow LLM nodes.
- Fixed AI inference pipeline failure handling so unsuccessful provider result envelopes are retried or rejected before empty text can enter chat, streaming, or workflow LLM outputs.
- Fixed OpenAI-compatible API success handling so 200 responses with missing or blank generated text are rejected before chat or workflow callers treat them as valid dialogue.
- Fixed streaming chat failure cleanup so provider errors replace partial assistant text with a stable failure bubble before surfacing the error.
- Fixed group chat generation failure handling so per-character provider failures surface as stable system messages, are omitted from future prompts, and do not log raw dialogue text.
- Fixed group chat command boundaries so participant IDs are trimmed, empty/duplicate participant sets are rejected, inactive sessions cannot advance, and blank messages cannot create dialogue turns.
- Fixed knowledge loading and chat context assembly so single-object knowledge files and creator-declared character knowledge references are pinned into prompts.
- Fixed event triggering so runtime checks and release-gate snapshots share the same serializable rule metadata.
- Fixed Quality Suite data-root discovery so release-gate runs can find project quality suites and knowledge anchors from nested desktop/dev working directories.
- Fixed Quality Suite runtime parsing so malformed suite metadata, duplicate scenario ids, and blank event-rule fields are rejected before execution.
- Fixed Quality Suites workbench error feedback so suite load and run failures show actionable validation messages instead of failing silently.
- Fixed visible separator artifacts in the Scene Assets and Quality Suites workbench metadata rows.
- Fixed browser locale fallback loading so Web/PWA deployments under `VITE_BASE_PATH` subpaths fetch locale JSON from the correct base URL.
- Fixed release verification coverage for Web/PWA subpath deployments by enforcing service worker base-path source invariants.
- Fixed installed desktop builds so Tauri-bundled `data/` resources are discovered at startup and rebound as the default project root when no development data root is available.
- Fixed project-scoped analytics, cloud-sync manifests, and generated TTS assets so installed desktop builds write them under the active project data root instead of the process working directory.
- Fixed evaluation score parsing so explanatory model strings such as `Score: 8/10`, `80% friendly`, and normalized decimal text still produce stable event-trigger scores.
- Fixed event availability previews so author tooling uses the same score-aware trigger decisions as runtime event firing instead of broad event-type approximations.
- Fixed the Sakura example workflow to demonstrate a score node feeding a story-event unlock node instead of ending immediately after scoring.
- Fixed workflow runtime and validation compatibility for legacy media fields such as `track`, `sound`, and second-based `duration`.
- Moved `synthesize_speech` onto the registered Tauri command path and connected saved TTS configuration to system, Azure, and ElevenLabs synthesis.
- Cleared stale example character sprite paths that pointed at missing files so browser Story Mode falls back cleanly to the generated 3D placeholder.

### Changed
- Removed engine-level achievements, achievement tracking, and milestone notifications so gamification remains an authored project choice rather than part of the development workspace.
- Reframed runtime-facing navigation as playtest, character test, visual review, and transcript tooling around the low-code authoring workflow.
- Changed the default workspace theme to a pale white monochrome design system with an optional grayscale dark mode.
- Project export manifests now scan project JSON content directories for characters, dialogues, knowledge, and scenes, making exports useful before runtime managers are initialized.
- Character loading now accepts one-character JSON files, legacy sprite field names, and partial personality definitions with stable defaults.
- Single-character and group chat prompts now share the character mind contract and guarded response path for stronger role stability, including AI/ChatGPT identity drift and customer-support/tool-style drift replacement.
- Version metadata synchronized to v0.9.5 across frontend, Rust workspace, Tauri config, and title screen UI.

## [0.9.4] - 2026-07-08

### Added
- **BackToTop component**: Scroll-to-top button with smooth scroll animation. Appears after 300px of scroll offset, integrated globally in App.vue.
- **Takeshi character**: Traveling photographer with 12-node through_the_lens dialogue (7 endings), cross-character connections to Sakura, Hana, Sora, Kai, Mio, and Nori. Springtown photographic archive knowledge entry.
- **ConfirmDialog component**: Polished confirmation dialog with backdrop blur for delete/destructive action confirmations. Supports custom title, message, and button labels via `v-model:visible` binding.
- **System info panel**: HomeView dashboard now shows engine version (v0.9.4), character/dialogue/knowledge/scene counts, AI engine status, and runtime state with color-coded Online/Idle indicator.

### Changed
- Content inventory expanded to 15 characters, 15 dialogues, 17 knowledge entries.
- HomeView ops-grid now includes a third panel for system information between the pipeline and getting-started sections.

## [0.9.3] - 2026-07-08

### Added
- **GlobalSearch component**: Ctrl+K quick-search across characters, knowledge entries, and dialogues from any view. Features expandable search panel, real-time filtering, keyboard shortcut support, and integrated into App.vue sidebar.
- **LoadingSpinner component**: Reusable loading indicator with customizable size, thickness, text, and inline mode. Integrated into HomeView dashboard for async status loading.
- **GameView SVG background loading**: Scene backgrounds now display actual SVG image files instead of generated gradients.

### Fixed
- **ChatView.vue encoding corruption**: Fixed a corrupted template expression at line 37 that caused "Element is missing end tag" build error.

## [0.9.2] - 2026-07-07

### Added
- **Kai character**: Wandering musician with 12-node cafe_encounter dialogue (5 endings), cross-character connections to Mio, Sakura, Luna, and Yuki. Traveler songs knowledge entry.
- **Hana character**: Tea shop owner with 13-node whispering_leaf dialogue (8 endings), tea blends knowledge. Richest dialogue in the collection with deep emotional arcs.
- **Auto-save in GameView**: Automatic save every 2 minutes during active dialogue with auto-save indicator badge.

### Changed
- Content inventory expanded to 12 characters, 12 dialogues, 14 knowledge entries.
- All new characters include personality Big Five traits, emotion states, relationship networks, and knowledge references.

## [0.9.1] - 2026-07-07

### Added
- **AchievementsView**: Gamification system with 15 unlockable achievements across Social, Relationships, Creation, and Gameplay categories. Features progress bars, category filtering, stats strip (unlocked/total/complete/playtime), and localStorage persistence. Achievements track first chat, message milestones, relationship scores, evaluation scores, workflow creation, knowledge entries, and more.
- **Batch i18n integration**: All remaining views now have `useI18n` imports and key `t()` string replacements: WorkflowEditor, AudioView, SceneEditorView, GroupChatView, AnalyticsView, MarketplaceView, PluginView, SceneAssetsView, CharacterEditorView, DialogueEditorView.
- **Achievements route** (`/achievements`) added to router and sidebar navigation (19 nav items total).

### Changed
- **Router** expanded to 20 routes with achievements entry.
- **Sidebar navigation** expanded to 19 items with Achievements entry.
- **Total frontend views**: 20 (up from 19).
- **i18n coverage**: All 20 views now import `useI18n` and use `t()` for at least header/title strings.

## [0.9.0] - 2026-07-07

### Added
- **TitleScreenView**: Cinematic title screen with animated particle effects, glowing logo, menu navigation, version badge, and MIT license footer.
- **CGGalleryView**: Scene and character art collection gallery with grid layout, locked/unlocked states, scene preview modal, tag pills, and color-coded thumbnails.
- **BacklogView**: Full conversation history viewer with character selector chips, role-based filtering, avatar color coding, emotion badges, timestamps, and jump-to-latest.
- **Comprehensive i18n locale system**: Expanded from 13 keys to 280+ keys covering all views.
- **Chinese locale (zh-CN)**: 280 translation keys for full Simplified Chinese support.
- **Japanese locale (ja-JP)**: 187 translation keys for Japanese market readiness.
- **Korean locale (ko-KR)**: 159 translation keys for Korean market support.
- **i18n integration in App.vue**: All 18 sidebar navigation labels use `t()` function.
- **i18n in core views**: HomeView, ChatView, GameView, SettingsView with full `t()` integration.
- **Mio character**: Festival organizer with Starlight Festival dialogue (15 nodes, 4 endings) and festival lore knowledge entry.
- **festival_night scene**: Summer night festival setting with weather/time metadata.

### Changed
- Router expanded to 19 routes.
- Sidebar navigation expanded to 18 items with CG Gallery and Backlog entries.
- App.vue now imports `useI18n` composable and uses computed nav items.
- Total frontend views: 19 (up from 16).
- Version badge updated from v0.8 to v0.9 in sidebar.
- Tauri config version bumped to 0.9.0, window title updated to "Monogatari v0.9.0".
- README updated with v0.9.0 content counts: 10 characters, 10 dialogues, 12 knowledge entries.

# Changelog

## [0.9.0] - 2026-07-07

### Added
- **TitleScreenView**: Cinematic title screen with animated particle effects, glowing logo, menu navigation (Start Game, Continue, Workflow, Gallery, Settings), version badge, and MIT license footer. Hides sidebar for immersive first impression.
- **CGGalleryView**: Scene and character art collection gallery with grid layout, locked/unlocked states, scene preview modal with weather/time-of-day metadata, tag pills, and color-coded thumbnails derived from scene IDs.
- **BacklogView**: Full conversation history viewer with character selector chips, role-based filtering (All/Player/Character), avatar color coding, emotion badges, timestamps, and jump-to-latest functionality.
- **Comprehensive i18n locale system**: Expanded from 13 keys to 280+ keys across all views covering navigation, chat, game, settings, workflow, characters, knowledge, dialogue, scene, audio, analytics, marketplace, plugins, group chat, title screen, backlog, CG gallery, and common UI strings.
- **Chinese locale (zh-CN)**: 280 translation keys for full Simplified Chinese support.
- **Japanese locale (ja-JP)**: 187 translation keys for Japanese market readiness.
- **Korean locale (ko-KR)**: 159 translation keys for Korean market support.
- **i18n integration in App.vue**: All 18 sidebar navigation labels now use `t()` function with locale-aware rendering via `useI18n()` composable.

### Changed
- **Router expanded** to 19 routes with Title Screen, CG Gallery, and Backlog entries.
- **Sidebar navigation** expanded to 18 items with CG Gallery and Backlog entries.
- **App.vue** now imports `useI18n` composable and uses `computed` nav items with `t()` for all labels.
- **Title Screen and Story Mode** routes hide the sidebar for immersive gameplay experience.
- **Total frontend views**: 19 (up from 16).
- **Version badge**: Updated from v0.8 to v0.9 in sidebar.

## [0.8.2] - 2026-07-07

### Added
- **SceneEditorView**: Visual scene management with grid/list gallery view, scene detail panel with background preview, weather/time-of-day selectors, BGM path, and tag configuration. Create, edit, and delete scenes.
- **Sidebar navigation** expanded to 17 items with Scene Editor entry.
- **Total frontend views**: 16 (up from 15).


## [0.8.1] - 2026-07-07

### Added
- **DialogueEditorView**: Visual branching dialogue editor with node tree canvas, inline choice editing, speaker assignment, validation, and JSON import/export.
- **export_project command**: Export project as JSON manifest with content inventory (characters, dialogues, knowledge, scenes) for packaging and distribution.
- **Aoi character**: Gentle healer with herbal medicine knowledge, clinic visit dialogue (11 nodes, 3 branching paths, 2 endings), and herbal lore knowledge entry.
- **CharacterGalleryView overhaul**: Search, detail panel with radar chart visualization, personality traits, quick action buttons (Chat/Edit), responsive layout.

### Changed
- **Sidebar navigation** expanded to 16 items with Dialogue Editor entry.
- **Total Tauri commands**: 30 (up from 25).
- **Total frontend views**: 15 (up from 14).
- **Content inventory**: 7 characters, 8 dialogues, 9 knowledge entries, 5 scenes.


### Fixed
- **Locale files encoding**: Fixed mojibake in zh-CN.json and ja-JP.json locale files. All translations now use proper UTF-8 encoding.
- **SettingsView language picker**: Language selector now calls loadI18n() to apply locale changes immediately without restart.

### Added
- **Japanese locale**: Complete ja-JP.json with nav, chat, game, settings, and common translations.
- **Knowledge Base Rust commands**: list_knowledge_entries, get_knowledge_entry, list_knowledge_tags Tauri commands for full KB management.
- **KnowledgeBase backend methods**: all_entries, all_tags, all_categories for comprehensive knowledge base access.

## [0.8.0] - 2026-07-07

### Added
- **Knowledge Base View** (`KnowledgeBaseView.vue`): Full knowledge base management with category filtering, tag cloud, keyword search, entry creation/editing/detail views, and card grid display.
- **Character Editor overhaul** (`CharacterEditorView.vue`): Professional 5-tab character editor with Basic Info, Personality (Big Five sliders + radar chart SVG visualization), Emotions, Relationships, and Knowledge management tabs. Includes character list sidebar, JSON export, and responsive layout.
- **Frontend data sync**: All characters (Sakura, Yuki, Hiro, Mei), scenes, knowledge entries, dialogues, and SVG backgrounds now synchronized from `rust-engine/data` to `data/` for frontend access.
- **Knowledge Base route** added to router and sidebar navigation with book icon.

### Changed
- **Sidebar navigation** expanded to 14 items with Knowledge Base entry.
- **Engine version badge** bumped to v0.8 in sidebar.
- **CharacterEditorView** completely rewritten from minimal 70-line form to 880-line professional editor with tabbed interface, personality radar chart, emotion configuration, relationship management, and knowledge entries.


## [0.7.2] - 2026-07-07

### Changed
- **README comprehensive update**: Version bumped to v0.7.2, architecture docs updated with all 13 views and 4 components, features section expanded with Audio Manager, GLTF 3D, and i18n.
- **CHANGELOG synchronized** with all changes since v0.6.0.


## [0.7.0] - 2026-07-07

### Added
- **Hiro character**: Young enthusiastic inventor with workshop dialogue (5 endings), knowledge entry, and workshop scene.
- **Yuki character**: Mysterious library guardian with branching dialogue (3 endings), knowledge entry, and Great Library scene.
- Engine now ships with **5 example characters** (Sakura, Luna, Kenji, Yuki, Hiro), **6 dialogue scripts**, **7 knowledge entries**, and **4 scenes**.


## [0.6.4] - 2026-07-07

### Changed
- **Tauri config version bumped** to 0.6.3 to match application version.
- **Release checklist** added at docs/RELEASE_CHECKLIST.md covering frontend, Rust backend, content, AI integration, workflow editor, audio, i18n, cloud sync, and distribution verification.


## [0.6.3] - 2026-07-07

### Changed
- **Enhanced AI prompt engineering**: Both streaming and non-streaming character AI prompts redesigned with stricter roleplay rules, emotional mirroring, varied speech patterns, and character growth awareness.


## [0.6.2] - 2026-07-07

### Added
- **GLTF 3D Model Loading**: CharacterModelView now loads .glb/.gltf models via Three.js GLTFLoader with OrbitControls, animation playback, ambient+directional lighting, and graceful fallback to a placeholder cube on error.
- **i18n nested key support**: Upgraded i18n composable with dot-notation nested keys, localStorage locale persistence, and local JSON file fallback.

### Changed
- CharacterModelView completely rewritten from static placeholder to full 3D pipeline with dynamic model loading and watch-based model path reactivity.


## [0.6.1] - 2026-07-07

### Added
- **Audio Manager** (AudioView.vue): Full BGM/SFX management with track listing, per-track volume control, play/pause, and master mixer panel with BGM/SFX/Voice channels.
- **Audio route and nav**: Added /audio route and sidebar navigation item for audio management.
- **i18n nested key support**: i18n.ts composable now supports dot-notation nested keys with localStorage locale persistence and local JSON fallback.
- **Enhanced prompt engineering**: Character AI system prompt redesigned with clearer roleplay instructions.


## [0.6.0] - 2026-07-07

### Added
- **Plugin Management UI** (`PluginView.vue`): Full frontend view for registering, listing, and removing custom plugins with modal registration form and status indicators.
- **Cloud Sync Settings** (SettingsView): Integrated cloud sync configuration with push/pull buttons, sync status display (last sync, file count, conflicts), and endpoint/token configuration.
- **i18n Locale Files**: Added zh-CN, ja-JP, and ko-KR locale files covering navigation, chat, game, settings, and common UI strings for multi-language support.
- **Sidebar Navigation**: Added Analytics and Marketplace nav items to main sidebar; added Plugins nav item.
- **Router Updates**: Added `/marketplace` and `/plugins` routes with lazy-loaded views.
- **Marketplace Dashboard Tile**: Added Marketplace tile to HomeView dashboard with community template browsing link.
- **Enhanced Group Chat**: Added streaming listener support, emotion display, relationship scores per participant, and animated spinner for typing indicators.

### Fixed
- **HomeView Dashboard**: Fixed Analytics tile route from `/settings` to `/analytics`.

### Changed
- Dashboard now shows 10 feature tiles covering all major modules.
- Sidebar navigation expanded to 12 items for complete feature coverage.
- Commercialization progress updated to reflect new capabilities.


## v0.5.0 - 2026-07-07 (Commercialization Push)

### Bug Fixes
- **Critical**: Fixed compile error in `chat.rs` where `.unwrap_or(0.0)` was called on an `f32` value in `check_event_triggers`. This blocked `cargo check` from passing.
- **Frontend**: Fixed SettingsView.vue broken HTML structure where the TTS section was misplaced inside the first panel's panel-head div.
- **Router**: Added missing `/characters` and `/group-chat` routes that were linked in sidebar but had no route definitions.

### Backend Improvements
- **TTS**: Upgraded `tts.rs` from stub to real Windows SAPI integration. `synthesize_speech` now invokes PowerShell SAPI COM to generate actual WAV audio files with emotion-based speech rate adjustment. `get_available_voices` discovers installed system voices.
- **Analytics**: Upgraded `analytics.rs` from stub to real implementation with in-memory event store, file persistence to `data/analytics.json`, and aggregation logic that computes top characters, top choices, session counts, and conversation metrics from recorded events.
- **Cloud Sync**: Upgraded `cloud_sync.rs` from stub to real local file-based sync with MD5 checksum tracking, manifest persistence, device-aware conflict detection, and pending upload counting.

### Frontend Improvements
- **Analytics Dashboard**: New `AnalyticsView.vue` with metrics strip (events, sessions, conversations, relationship score), top character/choice rankings, engagement overview, and JSON export functionality.
- **Dashboard**: Added Characters, Group Chat, and Analytics feature tiles to the home dashboard.
- **Dashboard Readiness**: Updated commercialization progress to include analytics dashboard, i18n scaffold, plugin system, cloud sync, and bug fix milestones.
- **Version Badge**: Updated sidebar version from v0.2 to v0.5.

### Documentation
- Updated CHANGELOG with v0.5.0 release notes.

## v0.5.1 - 2026-07-07 (Commercialization Continued)

### Features
- Template marketplace scaffold with list, export, and import commands (Rust backend)
- MarketplaceView frontend with template browsing, filtering, and import functionality
- Three.js dependency added for 3D character model support
- CharacterModelView component with Three.js dynamic import and rotation animation
- Tauri app config rebranded to Monogatari v0.5.0 (product name, identifier, window title)
- Game store enhanced with saveGame, loadGame, listSaves, deleteSave, setActiveScene, getRelationshipScore

### Bug Fixes
- Fixed Tauri config to use proper Monogatari branding instead of generic LLM Galgame Engine

---

## v0.4.1 - 2026-07-06

### Features
- i18n scaffold with locale loading, listing, and translation commands (EN/JA/ZH/KO locale files).
- Character management CRUD with create, delete, and summary commands.
- Korean locale file for i18n support.
- Example characters and content documentation in README.

### Content
- Added Kenji character with dojo knowledge for group chat dynamics.
- Added Kenji dojo dialogue with martial arts and poetry themes.
- Added Chinese locale file for i18n support.
- Added English and Japanese locale files for i18n support.
- Added dynamic effects workflow demo with camera, shake, random branch nodes.

---

## v0.4.0 - 2026-07-06

### Features
- Cloud save sync scaffold with push/pull/conflict resolution commands.
- Analytics scaffold with event recording and summary commands.
- Plugin system scaffold for custom workflow node types with register/list/remove.
- Springtown world knowledge entry for shared universe context.
- Sakura nature diary knowledge entry for AI context.
- Sakura park walk dialogue with cherry blossom themes and branching paths.

### Dashboard
- Updated Dashboard with Group Chat, Characters tiles and new readiness items.

### Documentation
- Updated README with latest features, characters, examples, and roadmap.

---

## v0.3.0 - 2026-07-06

### Features
- Multi-character simultaneous group chat backend (`multi_chat.rs`).
- TTS integration scaffold with voice assignment (`tts.rs`).
- 21 workflow node types with execution handlers for all types (added narration, bgm, sfx, wait, random_branch, sub_workflow, camera, shake nodes).
- Workflow validation with comprehensive error checking.

### Fixes
- Async-safe chat evaluation (blocking_read fix).
- Cargo dev profile optimization for faster builds.

### Frontend
- GroupChatView for multi-character conversations.
- CharacterGalleryView with personality trait visualization.
- CharacterEditorView for character customization.
- TTS settings in Settings view.
- Workflow editor CSS improvements.

---

## v0.2.0 - 2026-07-05

### Features
- Core engine architecture (EventBus, ServiceLocator, GameClock).
- AI inference pipeline (API + ONNX with DirectML).
- Character system (personality, memory, emotions, relationships).
- Dialogue system (branching, choices, flags, scripts).
- Knowledge base (keyword search, category/tag indexing).
- Scripting engine (Rhai-based).
- Save/load system.
- Free-form AI chat mode with streaming.
- Conversation evaluation and scoring.
- Event trigger system (relationship milestones, achievements).
- Visual workflow editor (drag-and-drop).
- Scene asset management.
- Project configuration panel.
- Live2D rendering (PixiJS).
- Tauri desktop application.
- Professional dark theme UI design system.
- Browser preview fallback for non-Tauri UI review.
