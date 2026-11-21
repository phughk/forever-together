# Tickets

One file per ticket, named `NNNN-slug.md`. Status is `open` unless it says otherwise. Each ticket has `Status` (open, in progress, blocked, done, dropped), `Kind`, `Source`, `ADRs` and `EGM` (links, or none). See [ADR 0036](../docs/adr/0036-how-designs-and-decisions-are-maintained.md). When work is decided, write an ADR (see CLAUDE.md); a ticket tracks what is still to do.

| # | Ticket | Kind |
|---|--------|------|
| 0001 | [Verify region values and the Hardcore realm value (item level and RP-PvE done)](0001-verify-live-client-calls.md) | verification |
| 0002 | [Verify a phone scans the addon QR](0002-verify-qr-phone-scan.md) | verification |
| 0003 | [Verify varaq key-file permissions and maturity](0003-verify-varaq-key-permissions.md) | verification |
| 0004 | [Verify iroh over the real network](0004-verify-iroh-live-network.md) | verification |
| 0005 | [Verify leader invitations on a live network](0005-verify-agreement-live.md) | verification |
| 0006 | [Verify the addon installer on a real install](0006-verify-addon-installer-real-install.md) | verification |
| 0007 | [Addon installer stage 2: native folder dialog and Linux detection](0007-addon-installer-stage-2.md) | feature |
| 0008 | [Supply the built-in trusted peer key](0008-trusted-peer-builtin-key.md) | prerequisite |
| 0009 | [Load a chosen private key into an instance](0009-load-private-key-into-instance.md) | feature |
| 0010 | [Decide who may run trusted peers and how keys are added](0010-trusted-peer-deployment-policy.md) | decision |
| 0011 | [Supply the addon page URL](0011-addon-page-url.md) | prerequisite |
| 0012 | [Addon release flow and update tracking](0012-addon-releases-and-updates.md) | feature |
| 0013 | [Blocking, verification and trust networks](0013-blocking-and-trust.md) | idea |
| 0014 | [Chat in the app once a group is formed](0014-chat-after-party.md) | idea |
| 0015 | [Join a party by QR code](0015-join-party-qr.md) | idea |
| 0016 | [Matchmaking for crafting, auctions, quests, PvP and arenas](0016-other-matchmaking-uses.md) | idea |
| 0017 | [Add map position (x, y) to the character code](0017-location-xy-in-code.md) | feature |
| 0018 | [Match zones on map IDs, not English names](0018-zone-matching-localisation.md) | feature |
| 0019 | [Tell apart dungeons that share a zone](0019-shared-zone-dungeons.md) | feature |
| 0020 | [Professions and gear detail in the code](0020-professions-and-gear-in-code.md) | idea |
| 0021 | [Shard busy topics](0021-topic-sharding.md) | idea |
| 0022 | [Encrypt the phone link](0022-encrypted-lan-link.md) | feature |
| 0023 | [Cross-faction groups](0023-cross-faction-groups.md) | idea |
| 0024 | [Re-audit the revised design](0024-re-audit-revised-design.md) | review |
| 0025 | [Search diagnosis: attempt counters](0025-diagnosis-attempt-counters.md) | feature |
| 0026 | [Search diagnosis: hysteresis, doubt level and hints](0026-diagnosis-hysteresis-and-hints.md) | idea |
| 0027 | [Search history estimator (stage 8b)](0027-history-estimator.md) | feature |
| 0028 | [Clear search history action](0028-history-clear-action.md) | feature |
| 0029 | [Record history checkpoints at about 1 and 2 minutes](0029-history-checkpoints.md) | feature |
| 0030 | [Show how many phones are linked](0030-per-phone-count.md) | feature |
| 0031 | [Phone link auto-off timer](0031-auto-off-timer.md) | feature |
| 0032 | [Replace the hostname guess for the phone read-only flag](0032-remote-flag-from-server.md) | bug |
| 0033 | [Remove remaining scroll in edge variations](0033-ui-scroll-edge-cases.md) | bug |
| 0034 | [Components use available space on all screen sizes](0034-empty-space-components.md) | bug |
| 0035 | [Run the web app against the real API in a real window and on a phone](0035-web-app-only-mock-tested.md) | verification |
| 0036 | [Document the submodule clone failure for test-addon](0036-mise-test-addon-submodule.md) | chore |
| 0037 | [Mark old names in ADRs and the design doc](0037-adrs-mention-old-names.md) | chore |
| 0038 | [Decide whether to self-host iroh relay and address lookup](0038-self-host-iroh-relay.md) | decision |
