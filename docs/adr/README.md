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
