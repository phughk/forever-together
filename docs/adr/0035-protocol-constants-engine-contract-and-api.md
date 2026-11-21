# 0035. Protocol constants, widening schedule, engine contract and local API

* **Status:** Accepted. This record contains what was in `docs/design-process/archive/forever-together-implementation-plan.md` (archived 2026-11-21): defaults, the engine contract, the API table, the build order and the verification plan. The numbers are starting values, unverified until a live run ([0025](0025-leader-invitations.md), `docs/spike-s0.md`).
* **Date:** 2026-11-21
* **Deciders:** Hugh Kaznowski

## Protocol constants

They are not user settings: a peer that changes them breaks agreement.

| Constant | Default |
|----------|---------|
| epoch (one round of invitations) | 5 s |
| ad keep-alive / expiry | 45 s / 120 s |
| ad cutoff before an epoch | 2 s |
| invitee decision time after epoch start | 2 s |
| sample size | 100 ads |
| clock skew | not applicable; each peer uses its own arrival time; an invitation for epoch n-1 or n+1 is kept one epoch |
| heartbeat and Ready reply window | 4 s |
| human Confirm click window | 15 s |
| local ban after no heartbeat reply | 60 s, never gossiped |
| search time limit | 10 min |
| widening step | 60 s |

| Stage | Starts | Rule |
|-------|--------|------|
| 0 | 0 s | item level within 15% of the lower value, level spread at most 3, exact roles |
| 1 | 60 s | secondary roles allowed (none in v1) |
| 2 | 120 s | item level within 30% |
| 3 | 180 s | level spread at most 6 |
| 4 | 240 s | level spread at most 10, and the other recommended dungeons |

The stage clock restarts when a dropped party returns the peer to searching. The stricter of two peers' stages applies to a pair.

User settings: phone link off, known-peers cache 100, no extra trusted peers, trusted-peer retry backoff 30 s doubling to 15 min with 20% jitter before the cap, peer exchange 1 request per 10 s per peer.

## Engine contract

* `compatible(a, b, stage)` is symmetric: same faction; level distance within the stage's spread; item level compatible when either is 0 or within the stage's percentage of the lower; both at least the dungeon's minimum level.
* Leader invitations: once per epoch a tank invites a healer, three damage peers and spares from its own sample, ordered by `pair_rank(epoch, tank, candidate)`; an invitee accepts the lowest-ranked leader 2 s into the epoch and commits only on that leader's announcement. A committed peer sends `Taken`.
* Human Confirm: after commit each app sends `Ready` within 4 s and shows the prompt; no click in 15 s sends `Decline` and everyone resumes with no penalty; a silent peer is banned locally for 60 s.
* The engine is pure: events in (`Tick`, `AdReceived`, `InviteReceived`, `AcceptReceived`, `AnnounceReceived`, `ReadyReceived`, `ConfirmReceived`, `DeclineReceived`, `HeartbeatTimeout`, user events), actions out (`SendAd`, `SendMessage`, `Join`, `Leave`, `ShowState`, `BanLocal`), and a `Network` trait with `join`, `leave`, `broadcast`, `send_direct`, `next_event`, `dial`.
* First engine test: five Alliance peers, levels 24 to 26, one tank, one healer, three damage, item levels within 15%: by the second epoch boundary all five commit to the same party and no sixth peer is included.

## Local API

Every route except static files needs `Authorization: Bearer <token>` (32 random bytes, hex, constant-time compare): missing or wrong gives 401; a wrong Host or Origin, or a non-loopback caller on a loopback route, gives 403. Errors are `{ "error", "message" }`. Bodies at most 8 KiB; 20 requests per second per token; a random free port.

| Route | Purpose |
|-------|---------|
| `GET /api/state` | snapshot: character, search (with optional `diagnosis`, 0026), stage, counts, party, settings summary |
| `GET /api/events` | live updates by `fetch` streaming, heartbeat every 15 s |
| `POST /api/character`, `DELETE /api/character` | store a code or forget it (also deletes history) |
| `DELETE /api/history` (loopback) | delete search history (0027) |
| `GET /api/dungeons` | table and recommendations |
| `POST /api/search`, `DELETE /api/search` | start or stop |
| `POST /api/party/confirm`, `POST /api/party/decline` | answer a committed party |
| `GET /api/settings`, `PUT /api/settings` | user settings; phone link and trusted peers are loopback only |
| `GET /api/pairing` (loopback), `POST /api/token/reset` (loopback) | pairing URL; new token, un-pairs phones |
| `GET /api/addon`, `PUT /api/addon/folder`, `POST /api/addon/install` (all loopback) | addon installer (0028) |

## Build order and verification

Order: spike S0, spec vectors and dungeon table, addon, app core (code, dungeons, store, config, identity, engine), `net`, server and shell, web app, integration. Exit conditions per step are the commands in `CLAUDE.md` (`mise run check` and its parts).

Verification: addon tests in the headless Lua VM; Rust unit tests per module; the seeded simulation in `engine::sim` as an algorithm check (0014); integration tests driving several in-process instances; manual checks the VM cannot do (installing the addon, `/wowft`, Ctrl+C copy, QR decode at UI scales 0.8, 1.0 and 1.2, phone pairing, token reset). Those manual checks are tickets 0001 to 0006 and 0035.

## Prerequisites outside the code

The trusted-peer key (ticket 0008) and the committed dungeon table (`app/data/dungeons.json`).
