# Volume 2 Dynamic Roleplay Plan

This production simulation derives scene facts from the user's local Volume 2
EPUB without copying its prose. All playable dialogue will be newly generated
or newly authored for bounded recovery and epilogues.

## Chapter 1 source boundary

- Source: `F:\下载\美好世界\《为美好的世界献上祝福》轻小说 epub\正文\2.epub`
- EPUB sections: prologue, five chapters, epilogue
- First production unit: Chapter 1, party exchange and winter death

## Stable content map

| Kind | IDs |
| --- | --- |
| New characters | `eris`, `dust`, `taylor`, `lynn`, `keith` |
| New knowledge | `volume2_winter_general`, `volume2_party_exchange` |
| New scenes | `axel_winter_spirit_field`, `eris_afterlife_hall`, `axel_goblin_mountain_road` |
| Roleplay | `volume2_chapter1_roleplay` |
| Campaign | `konosuba_volume2` |

## Intended live nodes

1. Negotiate a bounded snow-spirit job with Aqua, Megumin, and Darkness.
2. Recognize the Winter General and choose de-escalation over a forged victory.
3. Discuss death, attachment, and voluntary resurrection with Eris.
4. Define a one-day party exchange with Dust and every affected member.
5. Join Taylor, Lynn, and Keith with explicit roles and command boundaries.
6. Adapt Kazuma's basic skills into a coordinated goblin-route plan.
7. Debrief both temporary teams and decide what the exchange actually proved.

The NPC generator remains responsible only for visible in-character replies.
An independent evaluator may propose bounded score, evidence, and active-speaker
relationship changes. The deterministic Scene Roleplay state machine alone
selects nodes and endings.

## Foundation status

- Added and delivery-validated: Eris, Dust, Taylor, localized Rin (`lynn`),
  Keith, three scenes, two Knowledge entries, and eight renderer assets.
- Chapter 1 is playable through all seven live nodes with three deterministic
  endings and the `konosuba_volume2` Campaign.
- The clean Quality route executes 14 turns, visits 7/7 nodes, records 7/7
  evidence items, performs zero guard substitutions, and reaches
  `volume2_exchange_understood`.
- The adversarial route detects role override, prompt extraction, and state
  manipulation, guards the visible response, and leaves every score and
  evidence item unchanged.
- Fixed Dialogue is reserved for routed epilogues and recovery boundaries; it
  is not used as the primary conversation loop.
