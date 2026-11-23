# Tickets

One file per ticket, named `NNNN-slug.md`. Each has `Status` (open, in progress, blocked, done, dropped), `Priority` (P0 to P3, below), `Kind`, `Source`, `ADRs`, `EGM`, and `Blocked by` when it waits on another ticket. See [ADR 0036](../docs/adr/0036-how-designs-and-decisions-are-maintained.md). When work is decided, write an ADR (see CLAUDE.md); a ticket tracks what is still to do.

Priority was set on 2026-11-21 by one question: what must be true before real players can use this, and what does the ticket unblock? The network and the agreement mechanism have only run in simulation and on loopback, so live verification comes first.

## Do next, in order

1. **0008** `BUILTIN` is filled (trusted1, trusted2); install one on an always-on machine and check a fresh install reaches it.
2. **0004** two machines across the internet, using that trusted peer.
3. **0005** the agreement mechanism live (the 70% floor).
4. **0035** one real end-to-end session: window, phone, game.
5. In parallel, the cheap P1s: **0036**, **0011**, **0028**.

## P0: prove the core works (blocks any launch)

| # | Ticket | Kind | Why | Blocked by |
|---|--------|------|-----|------------|
| 0008 | [Supply the built-in trusted peer key](0008-trusted-peer-builtin-key.md) | prerequisite | Fill `BUILTIN`: cold-start fallback and the scan-export recipient. The key tooling now exists, so this is a short task. |  |
| 0004 | [Verify iroh over the real network](0004-verify-iroh-live-network.md) | verification | Nothing has run beyond loopback. Proves bootstrap, address lookup and relays across the internet. Everything else rests on it. | 8 |
| 0005 | [Verify leader invitations on a live network](0005-verify-agreement-live.md) | verification | The agreement mechanism passes only in simulation. If the 70% floor fails live, the product does not work. | 4 |
| 0035 | [Run the web app against the real API in a real window and on a phone](0035-web-app-only-mock-tested.md) | verification | No real window, phone or game session has been run end to end. | 8 |

## P1: needed before real players

| # | Ticket | Kind | Why | Blocked by |
|---|--------|------|-----|------------|
| 0049 | [Measure the relay rate limit and the DHT lookup](0049-measure-relay-rate-and-dht-latency.md) | verification | The relay rate, DHT latency and two build checks that ADR 0041 depends on. | 4 |
| 0048 | [Build the DHT lookup, published addresses and the headless relay](0048-build-address-lookup-and-headless-relay.md) | feature | Implements ADR 0041. | 49 |
| 0006 | [Verify the addon installer on a real install](0006-verify-addon-installer-real-install.md) | verification | The first-run path for players: the installer is untested on any real install. |  |
| 0011 | [Supply the addon page URL](0011-addon-page-url.md) | prerequisite | First-run step 1 is plain text until an addon page URL exists. |  |
| 0012 | [Addon release flow and update tracking](0012-addon-releases-and-updates.md) | feature | Players need release artefacts and a version flow before launch. |  |
| 0002 | [Verify a phone scans the addon QR](0002-verify-qr-phone-scan.md) | verification | Phone-scan path of the addon QR is unverified. |  |
| 0003 | [Verify varaq key-file permissions and maturity](0003-verify-varaq-key-permissions.md) | verification | Record store file modes on Linux and Windows (the lock result is done). |  |
| 0028 | [Clear search history action](0028-history-clear-action.md) | feature | Search history is recorded but cannot be cleared; a privacy gap (ADR 0027). |  |
| 0036 | [Document the submodule clone failure for test-addon](0036-mise-test-addon-submodule.md) | chore | Cheap fix that stops the addon tests failing on a fresh clone. |  |

## P2: next, once P0 and P1 are moving

| # | Ticket | Kind | Why | Blocked by |
|---|--------|------|-----|------------|
| 0039 | [Scan log: store module](0039-scan-log-store.md) | feature | Start of the scan-log chain: the log of pasted codes. |  |
| 0040 | [Scan log: export and clear API](0040-scan-log-export-api.md) | feature | Export and clear routes for the scan log. | 39, 44 |
| 0041 | [Scan log: Settings card and export flow](0041-scan-log-ui.md) | feature | Settings card for the scan log. | 40 |
| 0044 | [Encrypt the scan export to the first trusted peer key](0044-encrypted-scan-export.md) | feature | Encrypted export (age to `BUILTIN[0]`); run `cargo audit` first. | 8, 39, 40 |
| 0042 | [Collect region and realm values from players](0042-collect-region-and-realm-values.md) | verification | Collect US, KR, TW and CN values through GitHub issues. | 41, 44 |
| 0001 | [Verify addon client calls on a live client](0001-verify-live-client-calls.md) | verification | Region values other than EU and Hardcore; answered by the scan-log chain. | 42 |
| 0025 | [Search diagnosis: attempt counters](0025-diagnosis-attempt-counters.md) | feature | Attempt counters make the search diagnosis honest about invitations and drops. |  |
| 0029 | [Record history checkpoints at about 1 and 2 minutes](0029-history-checkpoints.md) | feature | History checkpoints: data only accumulates once recorded, so earlier is better. |  |
| 0046 | [Key actions in Settings, and the key commands on Windows](0046-key-actions-in-settings-and-windows.md) | feature | Key actions in Settings and the key commands on Windows (most players are on Windows). | 43 |
| 0007 | [Addon installer stage 2: native folder dialog and Linux detection](0007-addon-installer-stage-2.md) | feature | Installer stage 2: native folder dialog, Linux detection, first-run integration. | 6 |
| 0022 | [Encrypt the phone link](0022-encrypted-lan-link.md) | feature | The phone link sends its token over plain HTTP (accepted risk, ADR 0012). |  |
| 0032 | [Replace the hostname guess for the phone read-only flag](0032-remote-flag-from-server.md) | bug | Lock icons on the phone can be wrong until a 403 arrives. |  |
| 0024 | [Re-audit the revised design](0024-re-audit-revised-design.md) | review | Re-audit the design now that the ADRs hold the decisions. |  |
| 0018 | [Match zones on map IDs, not English names](0018-zone-matching-localisation.md) | feature | EU players on other client languages get no same-zone ordering. |  |
| 0033 | [Remove remaining scroll in edge variations](0033-ui-scroll-edge-cases.md) | bug | Recheck scrolling after any layout change. |  |
| 0034 | [Components use available space on all screen sizes](0034-empty-space-components.md) | bug | Check at several sizes and close if satisfied. |  |
| 0010 | [Decide who may run trusted peers and how keys are added](0010-trusted-peer-deployment-policy.md) | decision | Who may run trusted peers and how keys are added; decide when a second operator appears. | 8 |

## P3: later, ideas and cleanups

| # | Ticket | Kind | Why | Blocked by |
|---|--------|------|-----|------------|
| 0013 | [Blocking, verification and trust networks](0013-blocking-and-trust.md) | idea | Blocking and trust; needed once there are users to abuse the network. |  |
| 0027 | [Search history estimator (stage 8b)](0027-history-estimator.md) | feature | History estimator; blocked until live data shows the fields are stable. | 5, 29 |
| 0019 | [Tell apart dungeons that share a zone](0019-shared-zone-dungeons.md) | feature | Tell apart dungeons that share a zone. |  |
| 0017 | [Add map position (x, y) to the character code](0017-location-xy-in-code.md) | feature | Map IDs and x/y in the code (code version bump). |  |
| 0026 | [Search diagnosis: hysteresis, doubt level and hints](0026-diagnosis-hysteresis-and-hints.md) | idea | Diagnosis hysteresis and hints. | 25 |
| 0030 | [Show how many phones are linked](0030-per-phone-count.md) | feature | Per-phone count. |  |
| 0031 | [Phone link auto-off timer](0031-auto-off-timer.md) | feature | Phone link auto-off timer. |  |
| 0045 | [Replace `dyn Store` with generics or enums](0045-replace-dyn-store-with-generics.md) | chore | Replace `dyn Store` with generics. |  |
| 0037 | [Mark old names in ADRs and the design doc](0037-adrs-mention-old-names.md) | chore | Point old ADR wording at the rename table. |  |
| 0014 | [Chat in the app once a group is formed](0014-chat-after-party.md) | idea | Chat after a party forms. |  |
| 0015 | [Join a party by QR code](0015-join-party-qr.md) | idea | Join a party by QR. |  |
| 0016 | [Matchmaking for crafting, auctions, quests, PvP and arenas](0016-other-matchmaking-uses.md) | idea | Matchmaking beyond dungeons. |  |
| 0020 | [Professions and gear detail in the code](0020-professions-and-gear-in-code.md) | idea | Professions and gear in the code. |  |
| 0021 | [Shard busy topics](0021-topic-sharding.md) | idea | Topic sharding; needs real traffic numbers. | 4, 5 |
| 0023 | [Cross-faction groups](0023-cross-faction-groups.md) | idea | Cross-faction groups, if the game allows them. |  |

## Done

| # | Ticket | Note |
|---|--------|------|
| 0009 | [Load a chosen private key into an instance](0009-load-private-key-into-instance.md) | done, folded into 0043 |
| 0047 | [A headless trusted peer does not introduce players to each other](0047-headless-trusted-peer-introduces-nobody.md) | Fixed by the membership topic (ADR 0040); verified on one machine |
| 0038 | [Address lookup, published addresses and a headless relay (decision)](0038-self-host-iroh-relay.md) | decided in ADR 0041 |
| 0043 | [Key management: export, import and reset keys from the CLI](0043-key-management.md) | Part 1 done; Part 2 is 0044 |
