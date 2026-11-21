# forever-together code and protocol specification, version 1

Status: draft, awaiting spike S0. Values marked **provisional** may change when S0 reports. This file is the single source for the character code and the wire messages. The addon tests and the Rust tests both read `docs/rdf-code-vectors.json` (created in step 1; format below). The design is in [`designs/forever-together.md`](designs/forever-together.md) and the build order is in [`designs/forever-together-implementation-plan.md`](designs/forever-together-implementation-plan.md).

Short code: `wowft`. Names in this file: code prefix `WFT1:`, topic prefix `wowft/1/`, peer exchange ALPN `wowft/peers/1`.

## 1. Character code

The text is the prefix `WFT1:` followed by the base45 encoding (RFC 9285) of a binary record. The base45 alphabet (`0-9 A-Z space $ % * + - . / :`) is the QR alphanumeric alphabet, so the text goes into a QR code in alphanumeric mode, error correction level M. All numbers are big-endian. A string is a length byte followed by UTF-8 bytes, never cut inside a character.

| Offset | Field | Size | Meaning |
|--------|-------|------|---------|
| 0 | version | 1 | `1`. Any other value is refused with an "update" message. |
| 1 | flags | 1 | See "Flags". |
| 2 | class | 1 | class ID from the client |
| 3 | race | 1 | race ID from the client |
| 4 | level | 1 | 1 to 255. Level 0 is invalid. |
| 5 | item level | 2 | average item level, 0 when unknown |
| 7 | spec | 2 | spec ID from the client, 0 when unknown |
| 9 | created | 4 | epoch minutes (UTC) from the client's server time when the code was made |
| 13 | counter | 2 | increases each time the code is rebuilt in a session, wraps from 65535 to 0 |
| 15 | region | 1 | region number from `GetCurrentRegion()`, 0 when unknown |
| 16 | realm | string, 1 to 24 bytes | realm value, see "Realm and region" |
| after realm | first name | string, 1 to 24 bytes | character first name, never empty |
| after first name | surname | string, 1 to 24 bytes | character surname, never empty |
| after surname | zone | string, 0 to 32 bytes | zone name as the client reports it, empty when unknown |
| last 2 bytes | check | 2 | CRC-16/CCITT-FALSE over every earlier byte |

A decoder refuses trailing bytes between the zone and the check, and any record shorter than its fields require.

### Flags

| Bits | Meaning |
|------|---------|
| 0-1 | role: 0 unknown, 1 tank, 2 healer, 3 damage |
| 2-3 | secondary role, same values, 0 none. Always 0 in v1. |
| 4 | faction: 0 Alliance, 1 Horde. A neutral or unknown faction makes the code invalid in v1. |
| 5 | item level valid |
| 6-7 | reserved, must be 0 |

### Realm and region

The realm value and the region are opaque values from the client, compared as given. **Provisional:** which calls return them is an S0 output. Rule to keep topic strings equal across clients: the addon sends the realm as a decimal numeric ID (for example the game-mode record ID) when the client gives one, and otherwise the client's text lowercased in ASCII. If S0 finds that the only source is localized text, the spec adds a mapping table to canonical IDs before the code is frozen. Two peers on the same ruleset must produce the same topic string. Provisional implementation (see `spike-s0.md`): `C_GameRules.GetCurrentGameModeRecordID()` as a decimal string, else `GetRealmName()` lowercased in ASCII and cut at 24 bytes.

### Size

A typical record is about 45 bytes, about 70 characters of text with the prefix. Target: at most 122 characters of text, which is QR version 5 at level M in alphanumeric mode. The largest possible record (126 bytes) is 194 characters and fits version 8. Capacity at level M in alphanumeric mode, characters per version 1 to 10: 20, 38, 61, 90, 122, 154, 178, 221, 262, 311 (also in the vectors file). A data stream that ends exactly on a byte boundary after the terminator gets no extra zero byte before the pad bytes `0xEC`, `0x11`. The encoder picks the smallest version that fits, up to version 10. If the text does not fit version 10, the window shows the text only and says the QR is unavailable.

### Errors

One vocabulary on both sides: `UnknownVersion`, `BadCheck`, `Truncated`, `BadText`, `TrailingBytes`, `InvalidField(name)`. In Lua the same names are the second return value `err`.

Order of checks in a decoder, fixed by the vectors: strip the prefix and decode base45 (`BadText`); no bytes at all gives `Truncated`; a first byte other than `1` gives `UnknownVersion` (whatever follows, so a future layout is always reported as "update"); fewer than 25 bytes gives `Truncated`; a wrong CRC gives `BadCheck`; only then are the fields read, giving `InvalidField(name)` (`flags` for a reserved bit, `level`, `realm`, `firstName`, `surname`, `zone`; a length byte above its maximum or invalid UTF-8 also gives `InvalidField`), `Truncated` when a string runs past the end, and `TrailingBytes` when bytes remain between the zone and the check. So a text cut short but still 25 bytes or more is `BadCheck`, and a text cut shorter is `Truncated`. A decoder reports the item level as 0 when flag bit 5 is clear. Lua error strings are exactly these names, with the field in brackets, for example `InvalidField(level)`.

### Pasted text

Strip leading and trailing whitespace and line breaks only. Never strip spaces inside the text, because space is a base45 character. A test vector has an internal space.

### Collecting each field in the addon

| Field | Source | Fallback | Unknown value |
|-------|--------|----------|---------------|
| first name, surname | `UnitName("player")` and `GetUnitName("player")` (assumed to give "First Surname", unverified until S0) | split `GetUnitName("player")` on the first space | code is not built, window says why |
| realm | realm-value call confirmed by S0 | none | code is not built |
| region | `GetCurrentRegion()` | none | 0 |
| faction | `UnitFactionGroup("player")` | none | code is not built |
| class, race, level | `UnitClass`, `UnitRace`, `UnitLevel` | none | class and race 0, level invalid |
| spec, role | `GetSpecialization()` then `GetSpecializationInfo(index)`, whose fifth return is the role: TANK 1, HEALER 2, DAMAGER 3 | none | spec 0, role 0 |
| item level | `GetAverageItemLevel` | on the next `PLAYER_EQUIPMENT_CHANGED` event, an 18-slot sum, never a timer | 0 and flag bit 5 clear |
| zone | `GetRealZoneText()` | `GetZoneText()` | empty string |

Order of operations: read, clamp numbers to their field sizes, truncate strings at character boundaries, encode. Missing values are listed in a short diagnostic ("region=?") in the window.

## 2. Topics

`topic = SHA-256("wowft/1/" + region + "/" + realm + "/" + faction + "/" + slug + "/" + shard)`, 32 bytes. `region` and `shard` are decimal text, `faction` is `alliance` or `horde`, `slug` is the dungeon's fixed English slug. In v1 `shard` is always `0`. The slash-separated string is joined with no spaces.

## 3. Messages

Messages are serialized with `postcard`. Every gossip message carries the author's peer ID and an ed25519 signature made with the author's iroh key. The signature covers `topic || 0x01 || postcard(message without signature)`, so a message cannot be replayed into another swarm or protocol version. The `net` module verifies signatures. The `engine` receives only verified messages. If S0 shows that iroh-gossip already identifies authors, this rule is replaced (ADR 0019).

| Message | Fields | Meaning |
|---------|--------|---------|
| `Ad` | `peer, counter, role, secondaryRole, level, itemLevel, faction, stage, firstName, surname, zone, createdAt` | a peer is searching. `createdAt` is the sender's clock and is informational only. |
| `Taken` | `peer, counter` | the peer committed to a party. Others drop it from their samples. This is a core rule. |
| `PartyAnnounce` | `peer, epoch, members[5], partyId` | baseline mechanism |
| `Ready` | `peer, partyId, nonce` | the peer's app is alive and has shown the Confirm prompt |
| `Confirm` | `peer, partyId` | the human pressed Confirm |
| `Decline` | `peer, partyId` | the human did not press Confirm in time, or pressed Decline. No penalty. |
| `Heartbeat` / `HeartbeatAck` | `peer, partyId, nonce` | liveness check among party members, signed |
| `Propose` / `Respond` | **provisional**, defined only if the proposer model is adopted | see the design, section 6 |

`partyId = SHA-256(topic || epoch as 8 bytes big-endian || the five peer IDs sorted ascending, 32 bytes each)`.

Epoch ordering hash: `hash(epoch, peerId)` is the first 8 bytes of SHA-256 of `epoch as 8 bytes big-endian || peer ID`.

The proposer model, if adopted, replaces `PartyAnnounce` with `Propose` and `Respond`. The author's description: a partial group advertises the roles it needs, responders reply, and the proposer decides within a short window. A responder accepts one proposer at a time.

## 4. Peer exchange

ALPN `wowft/peers/1`. Each request and its response use one bidirectional stream. A `Hello` is its own stream, sent by each side when a connection opens.

| Message | Fields and limits |
|---------|-------------------|
| `Hello` | `topics[]`, at most 20 |
| `Request` | `topics[]`, at most 3, and `max`, at most 20 |
| `Response` | `entries[]`, at most 20, each `Entry { peer, topics[] (at most 20), lastSeen }` |

Messages are postcard, length-prefixed with a 4-byte big-endian length, at most 16 KiB. A stream with no complete message in 5 seconds is closed. A peer that exceeds a limit is ignored for 10 minutes. The ignore list is held in memory only. The stored record `known_peers/<peerId>` is `{ topics[], lastSeen, lastUsed }`.

## 5. Dungeon table

`src-tauri/data/dungeons.json`, committed before step 3. Fields: `slug`, `name`, `levelMin`, `levelMax`, `minLevel`, `factionHint` (`alliance`, `horde`, `both`, `none`), `zone`, `status` (`available`, `coming_soon`). The table has 29 entries, 20 available. **The author commits the file.** Recommendation for level L and zone Z: keep available entries with `minLevel <= L` and `levelMin - 3 <= L <= levelMax + 3`. Sort by: zone equal to Z (ASCII-lowercased compare) first; then the distance of L from the middle of the range; then `factionHint` equal to the character's faction, then `both`, then the other faction, then `none`; then the slug. The function is `recommend(character, table) -> Vec<DungeonSlug>`.

## 6. Change policy

A layout change means a new version byte and a new prefix. Changing the table's contents does not change the wire version, because a topic uses only the slug: a new dungeon is a new topic and moves no existing topic.

## 7. Test vectors

`docs/rdf-code-vectors.json` is generated once and committed; the addon tests read it through `vm.read_json`, and the Rust tests will read it too. Keys: `base45` and `base45_errors`, `crc16`, `records` (`fields`, `hex`, `text`), `decode_errors` (`text`, `error`), `paste` (`pasted`, `text`), `encode_errors` (`fields`, `error`), `counter_wrap`, `limits`, `qr` (`text`, `version`, `size`, `mask`, `rows` as strings of `0` and `1`), `qr_capacity_alnum_m`, `qr_errors`. Record `fields` use `role` and `secondaryRole` 0 to 3, `faction` as `alliance` or `horde`, `itemLevel` 0 for unknown, and the other names as in section 1.

The codec values come from an independent Python writer. The QR matrices come from `segno` 1.6.6 with one patch (segno appends a stray zero byte when the data ends on a byte boundary; the patch removes it), and each matrix was checked equal to `python-qrcode` forced to the same version and mask. The addon's mask choice follows segno's penalty scoring (matrix scored before the format information and the dark module are drawn), so a different penalty reading could pick another valid mask for a near-tie.
