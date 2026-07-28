# Volume 6 Chapter 2 Roleplay Plan

`volume6_chapter2_roleplay` is the playable chapter. It is an eight-node
real-time interaction graph, not a fixed Dialogue.

## Runtime loop

1. The active node selects the scene and available NPC participants.
2. The selected NPC receives its character profile, node-local goal,
   relationship, bounded transcript, and pinned Knowledge.
3. The NPC model generates only the in-scene reply.
4. A separate evaluator proposes score deltas and exact player-quote evidence.
5. The deterministic state machine validates and clamps the proposal, then
   selects the next node or ending.
6. Detected control attacks skip both model calls and create an audit-only
   guarded transcript record without advancing a story turn.

## Route authority

The model cannot select a route, forge evidence, or directly mutate scores.
Only the authored score/evidence conditions and transition priorities can
advance the graph.

## Acceptance

- strict, supervised, and exhaustion endings are reachable;
- every clean route covers all eight nodes;
- all sixteen evidence IDs are independently required on the strict route;
- structural intrusion stays on `guest_status_and_return_contact`;
- structural intrusion leaves `total_turns` and `node_turns` at zero;
- browser and Rust state machines share the same no-progress attack semantics;
- fixed Dialogue files are limited to short ending epilogues.
