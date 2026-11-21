# 0022. A character's name is two fields, first name and surname, and the realm is never read from the name call

* **Status:** Accepted
* **Date:** 2026-11-21
* **Deciders:** Hugh Kaznowski

## Context

An earlier draft treated the name as one field and read the realm from the second return of `UnitFullName`. On Forever that second return is a surname. Installed addons on this machine say so (`FishFry/Core.lua`: "Forever exposes the complete 'First Surname' in GetUnitName. On this flavor the second UnitFullName/UnitName return is a SURNAME, not a realm."), and the API export lists `ShouldDisplaySurname` and `ReplaceSurnameSeparatorWithLinkSeparator`. The author confirms that a character name is always two parts, first name and surname, as separate fields.

## Decision

The code, the ads, the party members and the revealed names carry first name and surname as two separate fields. The addon reads them from `UnitName("player")` and checks them against `GetUnitName`. The realm is never taken from the name calls. Its source is the realm-value call the spike confirms. The text typed into the in-game invite is "First Surname".

## Consequences

The code has one more length byte and a second string. Whether `UnitName("player")` returns the pair on the real client, and whether the invite accepts "First Surname", are unverified and are checked in the spike. Duplicate-name rules (0016) apply to the combined first name and surname.

## Alternatives considered

* A single name field with the realm taken from the second return: wrong on Forever, rejected.
