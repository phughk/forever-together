# Architecture decision records

Decisions for forever-together (short code `wowft`, working name `p2prdf` in older records), one file each. Status values: **Accepted** (decided by the author), **Proposed** (a choice made while drafting that awaits the author's review), **Superseded**. When a decision changes, add a new record or revise the existing one and say what it supersedes. Dates are 2026-11-20 unless noted.

| # | Decision | Status |
|---|----------|--------|
| 0001 | [Use a native Tauri desktop app instead of a browser-only page](0001-native-desktop-app.md) | Accepted |
| 0002 | [The addon only shows character data and does no networking or pairing](0002-addon-shows-data-only.md) | Accepted |
| 0003 | [Character data is self-reported, with a typo check and no forgery protection](0003-self-reported-data.md) | Accepted |
| 0004 | [A realm is a ruleset on a shared mega server, supplied by the addon as an opaque value](0004-realm-is-a-ruleset.md) | Accepted |
| 0005 | [A topic is region + realm + faction + dungeon, and level is not in it](0005-topic-without-level.md) | Accepted (amended: one shard in v1) |
| 0006 | [The app hard-codes a dungeon table and recommends dungeons from level and zone name](0006-dungeon-table.md) | Accepted |
| 0007 | [Parties are same-faction only, and faction is in the code and the topic](0007-same-faction-parties.md) | Accepted |
| 0008 | [Use iroh and iroh-gossip, rustls, varaq storage and permanent keys](0008-iroh-and-local-storage.md) | Accepted |
| 0009 | [Bootstrap through remembered peers, peer exchange, then trusted peers, with an LRU known-peers table](0009-bootstrap-and-peer-exchange.md) | Accepted |
| 0010 | [Peers agree on parties by converging announcements and a human Confirm](0010-converging-announcements.md) | Superseded by 0025 |
| 0011 | [A search loosens over time on a shared schedule](0011-search-widening.md) | Accepted (revised) |
| 0012 | [One web app served by the desktop app to its own window and to phones, with a token for phones](0012-one-web-app-for-phone-and-desktop.md) | Accepted |
| 0013 | [The desktop app is one Rust crate with modules](0013-single-rust-crate.md) | Accepted |
| 0014 | [Tests cover functional behaviour, with no performance or large-scale tests](0014-functional-tests-only.md) | Accepted (amended: simulation) |
| 0015 | [Diagrams in project documents use Mermaid only](0015-mermaid-only-diagrams.md) | Accepted |
| 0016 | [Peers are identified by their iroh key, never by character name](0016-peers-identified-by-key.md) | Accepted |
| 0017 | [The addon is an rdf/ module inside the existing Social Hub addon](0017-addon-in-social-hub.md) | Superseded by 0020 |
| 0018 | [The character code is a versioned binary record in base45 text with shared test vectors](0018-code-format.md) | Accepted |
| 0019 | [Gossip messages are signed by their author](0019-signed-gossip-messages.md) | Accepted |
| 0020 | [One repository (forever-together) holds a standalone addon, the app and the web app](0020-one-new-repository-standalone-addon.md) | Accepted |
| 0021 | [The headless test VM stays in wow-forever-addons and is referenced as a submodule](0021-test-vm-referenced-from-wow-forever-addons.md) | Accepted |
| 0022 | [A character's name is two fields, first name and surname](0022-name-is-first-name-and-surname.md) | Accepted |
| 0023 | [Avoid supernodes: trusted peers are ordinary peers](0023-no-supernodes.md) | Accepted |
| 0024 | [A committed party needs a Ready check, a human Confirm, and a Decline without penalty](0024-liveness-and-decline.md) | Accepted (amended by 0025) |
| 0025 | [Peers agree on a party by leader invitations: a tank invites from its own sample, invitees follow the best-ranked leader](0025-leader-invitations.md) | Accepted |
| 0026 | [The app reports search health from this peer's own observations](0026-search-health-from-own-observations.md) | Accepted (first slice; amends 0011) |
| 0027 | [The app keeps a local history of finished searches, one entry each, and shows none of it](0027-search-history.md) | Accepted (stage 8a, record only; extends 0008, amends 0013) |
| 0028 | [The desktop app installs the bundled addon](0028-the-desktop-app-installs-the-bundled-addon.md) | Accepted (first slice; amends 0002, 0013, 0020) |
| 0029 | [The web app follows atomic design and "show, don't tell"](0029-atomic-design-and-show-dont-tell.md) | Accepted |
| 0030 | [The web app is dark-only, with palette tokens and two fonts](0030-dark-only-theme-and-fonts.md) | Accepted |
| 0031 | [The finder shows one screen per state, and a tap on a dungeon starts the search](0031-one-screen-per-state-and-click-to-search.md) | Accepted (amends 0011, 0026 presentation) |
| 0032 | [Settings group phone access in one card, put expert settings behind Advanced, and lock them on a phone](0032-settings-phone-link-and-advanced.md) | Accepted |
| 0033 | [The header shows only a gear and Back, and the character strip always offers Change character](0033-header-and-character-strip.md) | Accepted |
| 0034 | [System overview, goals, user journeys, non-functional requirements and risks](0034-system-overview-goals-and-risks.md) | Accepted (contains the archived design doc) |
| 0035 | [Protocol constants, widening schedule, engine contract and local API](0035-protocol-constants-engine-contract-and-api.md) | Accepted (contains the archived implementation plan) |
| 0036 | [Tickets drive work, EGM iterations live in one folder per ticket, and ADRs record the outcome](0036-how-designs-and-decisions-are-maintained.md) | Accepted |
| 0038 | [The app keeps a log of the codes it has scanned and lets the player export it to attach to a GitHub issue](0038-scan-log-and-opt-in-export.md) | Accepted (not built) |
| 0039 | [The app's keys can be exported, imported and reset from the command line](0039-key-management.md) | Accepted (requirements; not built) |
| 0040 | [Every node joins a membership topic, and new members are announced on it](0040-membership-topic.md) | Accepted (built; verified on one machine) |
| 0041 | [Address lookup through n0 DNS and the DHT, direct addresses published, and a rate-limited hole-punching relay in headless nodes](0041-address-lookup-and-hole-punching-relay.md) | Accepted (not built) |
| 0042 | [The window is a Cargo feature, so a headless build needs none of its libraries](0042-ui-is-a-cargo-feature.md) | Accepted (built) |
| 0043 | [A WAV gong plays in the web app when a group forms](0043-group-formed-sound.md) | Accepted (built) |
