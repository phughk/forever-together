# Author decisions D1 to D9

Recorded 2026-11-21 in answer to the open questions in `consolidated_review_report.md`. Nothing in the design (`docs/designs/p2prdf.md`) or the ADRs has been edited yet. Each row says what must change once the decision is applied.

| # | Decision | Effect on the design and ADRs |
| --- | --- | --- |
| D1 | **Yes, simulate.** A seeded simulation of the agreement rule on the pure `engine` is allowed. It must cover the timing windows below. | Amend ADR 0014: a seeded algorithm simulation on the in-memory fake is functional testing, not a load test. The commit rate becomes a measured number with a kill criterion in spike S0 (B1 task 5). |
| D2 | **Evaluate both in spike S0.** Converging announcements (ADR 0010) is the baseline. A proposer model is simulated against it, and the choice follows the commit rate. | ADR 0010 stays Accepted until the spike ends. Add the proposer model as an alternative, with the author's description below. Spike S0 gets an exit criterion that picks one. |
| D3 | **Keep one v1.** Phone, LAN, token and override stay in v1. | No cut line. The plan needs a build order only (B3). ADR 0012 unchanged. |
| D4 | **One shard per topic, keep the hook.** The shard number stays in the topic hash and is always 0 in v1. | Remove the shard table, `shardCount` and the "more shards" widening stage from v1, and list sharding under Future Improvements. Topic hash input is unchanged, so adding shards later keeps topic names stable. Update ADR 0005. |
| D5 | **Build peer exchange. No spike gate.** Principle: avoid supernodes. Trusted peers are ordinary peers that also refresh their tables through peer exchange. They are the fallback when nothing else works, not the hub. | ADR 0009 stays. Reword section 4 so trusted peers are not special. Trusted-peer load on cold starts is an accepted risk (M5, M6), not a solved one. |
| D6 | **Yes, add a machine-level check before the human Confirm.** See "Timing and liveness rules". | Revise ADR 0010 and section 6. Replace "non-confirmer gets a local backoff" with the Decline and heartbeat rules below. |
| D7 | **Rename.** Full name `forever-together`, short code `wowft`. | Amend ADR 0020. Identifiers below are proposals awaiting the author's sign-off. |
| D8 | **Accept ADR 0018 and 0019**, and the implementation details in 0012. | Change the three statuses to Accepted. 0019 keeps its caveat: if the spike shows that iroh-gossip identifies authors, it is replaced. |
| D9 | **Move implementation detail out of the design.** | The design keeps flow, interfaces and data flow. The byte layout goes to `docs/rdf-code-spec.md`. Defaults, the API table and the per-file list go to a separate implementation plan. Each repeated fact gets one authoritative copy. |

## Timing and liveness rules (D1, D2, D6)

Stated by the author. Numbers are not yet chosen.

1.  **Honour-system "taken" broadcast.** When a party forms, its members broadcast that they are taken. Other peers drop them from their candidate lists. Today this exists only as a side note in section 5, and it becomes a core rule.
2.  **Proposer with a response window (to be simulated against ADR 0010).** A partial group advertises what it still needs, for example "I have 2 damage dealers and need a tank, a healer and 1 damage dealer". Responses come in, and the proposer must decide within a few seconds of each response. If nobody responds, the group fails.
3.  **Confirm needs a human click, and the protocol carries it.** The app or web app must be clicked to confirm, and the protocol sends that confirmation. If nobody clicks in time, the app sends a message saying the check failed (a Decline). The peer is not penalised and stays in the network.
4.  **No heartbeat response means unreachable.** If a peer does not even answer the heartbeat, the others mark it unreachable locally and ban it temporarily, about 1 minute. The ban is local and is never gossiped.

Open points for these rules:

*   The window lengths are not set: the response window, the decision window, the heartbeat window and the human click window.
*   A responder that accepts one proposer and refuses another is a lease. The author accepts this for the proposer model if it is chosen.
*   The 1-minute ban is per key. Someone who rotates keys avoids it (M4 stays open).
*   This replaces the current rule that a non-confirmer gets a local backoff.

## Proposed identifiers for D7 (not yet approved)

| Item | Was | Proposed |
| --- | --- | --- |
| Addon folder and TOC | `P2PRDF` | `ForeverTogether` |
| Topic prefix | `p2prdf/1/...` | `wowft/1/...` |
| Peer exchange ALPN | `p2prdf/peers/1` | `wowft/peers/1` |
| Slash command | `/rdf` and `/p2prdf` | `/wowft` |
| Environment variable | `P2PRDF_WEB_DIR` | `WOWFT_WEB_DIR` |
| Code prefix | `P2R1:` | `WFT1:` |

## What these decisions do to the blockers

*   **B1 (agreement mechanism):** still open until spike S0 picks between ADR 0010 and the proposer model. The doc can state both and the kill criterion now.
*   **B3 (phase gates):** needs no cut line (D3). Map items to steps.
*   **B4 (spike):** S0 gains the D1 simulation and the D2 choice. D5 adds no criterion.
*   **B7 (data inputs):** the shard table and `shardCount` go away (D4). The dungeon table and the trusted-peer test key are still needed.
*   **M3 (sharding):** mostly resolved by D4.
*   **M14 (altitude):** resolved by D9, which is a large edit.

## Answers to the escalations (2026-11-21)

*   **E-1 (problem premise).** Forever has no random dungeon finder as in retail. It has a classic-style group finder, which is manual, so groups are hard to find. This project is a faster, more automatic finder. The Problem section should say this and drop the "I have not checked" note. The in-game alternative is still written up as an alternative (classic-style manual finder), with that as the reason.
*   **E-2 (test VM).** The VM is at `/Users/hughkaznowski/Projects/gitlloop/wow-forever-addons` (the project moved). Checked in `wowluavm/src/prelude.lua`:
    *   `G.GetCurrentRegion() return 3 end` exists at line 154, before `world` is defined (line 523). The plan must **modify** it, for example `return setup.region or 3`. `vm.world = setup` (line 53), so the key is a top-level `region` in the setup JSON.
    *   `GetAverageItemLevel` is absent (0 matches). `vm.read_json` is absent in `main.rs`.
    *   `C_SpecializationInfo` (line 659) has `GetSpecialization` and `GetSpecializationInfo`, whose fifth return is the role. `GetSpecializationRole` is a global in the API export and not part of `C_SpecializationInfo`. Use `GetSpecialization` plus `GetSpecializationInfo`, so the real call is not reported as unimplemented.
    *   `UnitName` returns `name, realm` with realm nil at home. `GetUnitName` returns only `name`. The VM has no surname concept, so a surname test needs either a VM change or `Collect` splitting a combined name.
*   **E-3 (numbers, chosen by Claude as sensible starting values, to be tuned by the simulation).** These follow the author's guidance: 3-5 s for decisions and about 1 minute for searching.

    | Window | Starting value |
    | --- | --- |
    | Heartbeat or Ready reply | 4 s |
    | Proposer response window | 4 s |
    | Proposer decision after a response | 4 s |
    | Human Confirm click | 15 s (longer than a decision because a person must look at the screen) |
    | Local ban after no heartbeat reply | 60 s, never gossiped |
    | Search stage step (widening) | 60 s |
    | Simulation pass floor | at least 70% of peers that have a compatible five in their sample commit within 3 epochs, at 3,000 peers and a sample of 100 |

    The simulation pass floor is a guess. It is the number that most needs the author's or the simulation's correction.
*   **Template.** Copied from `/Users/hughkaznowski/Projects/gitlloop/template-rust-app` at commit `9048274` into this repo (tracked files only). `CLAUDE.md` merges the template's rules with this project's. This resolves part of M21 (template commit pinned). Crate versions are still unpinned.
