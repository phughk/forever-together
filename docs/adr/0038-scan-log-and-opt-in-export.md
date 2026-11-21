# 0038. The app keeps a log of the codes it has scanned and lets the player export it to attach to a GitHub issue

* **Status:** Accepted (decided by the author 2026-11-21; not built). Extends [0008](0008-iroh-and-local-storage.md) (local storage). Amends [0013](0013-single-rust-crate.md) (the module list gains `scans`). Cites [0003](0003-self-reported-data.md), [0018](0018-code-format.md) and [0027](0027-search-history.md). Items marked *default* are my choices where the author did not say; they are revisable.
* **Date:** 2026-11-21
* **Deciders:** Hugh Kaznowski

## Context

Live facts about the game (region numbers for US, KR, TW and CN, the Hardcore realm value, how item level and zone strings look in other languages) can only come from real clients, and the author has one EU client. Players hold the rest. The codes they paste already contain region, realm, item-level flag and zone. A log of those codes, exported on request, gives the author real samples with no service and no automatic upload ([0034](0034-system-overview-goals-and-risks.md): nothing is hosted).

It also shows how one character's code changes over time (level, item level, zone, spec), which is data the spec has no other source for.

## Decision

* **What is logged.** Every code the app accepts (`POST /api/character` succeeds) is appended to a local scan log: the raw `WFT1:` text and the time it was accepted (UTC seconds). The record is not decoded again; the code is the source of truth ([0018](0018-code-format.md)). An identical code (same `created` and `counter`) is not stored twice.
* **Per character, bounded.** A character is identified by region, realm, first name and surname (a name alone is not unique across realms). *Default:* faction is not part of the identity. At most **20 entries per character**; the oldest is dropped when a 21st arrives. Each entry expires **3 months (90 days)** after it was accepted. Pruning runs on every write and before every export. *Default:* a refreshed code for a known character is simply a new entry, so the sequence shows how it changed; the cap and the expiry stop it growing.
* **Store.** One key per entry under `scans/`, like `known_peers/` and `history/`: `scans/<region>/<realm>/<first name>/<surname>/<accepted UTC seconds, zero-padded>`. Each entry carries a schema version. An entry that fails to decode is skipped and never blocks a scan. A failing write is logged and never rejects a paste.
* **A new module `scans`**, a pure module over the key-value store with no new dependency, outside the engine.
* **Local only until the player acts.** The app never uploads anything. **Export** is a desktop-only (loopback) action that produces one JSON file: app version, spec version, export time, and the characters with their entries (raw code and accepted time). The player reads it, then attaches it to a new GitHub issue through the existing report link. The JSON is too large for a link, so the app offers the file and the link, not a prefilled body.
* **Consent wording, one sentence, before export:** the file contains character names, realms, zones and the times the codes were made. Names and zones are in the codes themselves, so removing them would remove the samples that matter. *Default:* export includes every logged character; the player can delete entries or the whole log first.
* **Controls.** "Clear scan log" (desktop only). The log is independent of the current character: `ClearCharacter` does not delete it, because other characters are the point. Search history ([0027](0027-search-history.md)) is never part of this export.
* **Test and doc data** use invented characters only ("Testa Fakename", "Testville"). Real exported files are never committed. Facts learned from them are recorded in the spec, not the files, as with the live captures.

```mermaid
flowchart LR
    P[Player pastes code] --> V[Decode and validate]
    V --> S[Store as current character]
    V --> L[Append to scan log, prune]
    L --> X[Export JSON on request]
    X --> R[Player attaches it to a GitHub issue]
    R --> A[Author records facts in the spec]
```

## Consequences

* Region and realm values for players on other regions arrive as codes, and the author maps them as in ADR 0004.
* A third party reading an exported file learns names, realms and zones of the player's characters. The player chooses to share it.
* Entries older than 90 days vanish even if nobody has exported them.
* Not decided: whether to offer an export that hides names, and a size cap for the file. The cap of 20 entries per character bounds a character, not the number of characters.
