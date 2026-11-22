# Professions and gear detail in the code

*   **Author:** Hugh Kaznowski
*   **Status:** Initial draft, unreviewed (idea ticket: this draft is about whether and when to build)
*   **Date:** 2026-11-22
*   **Ticket:** [ticket 0020](../../../tickets/0020-professions-and-gear-in-code.md)
*   **Related decisions:** [ADR 0003](../../adr/0003-self-reported-data.md), [ADR 0018](../../adr/0018-code-format.md). Where this document and an ADR disagree, the ADR wins.

--------------------------------------------------------------------------------

## Overview

The code today carries role, spec, class, race, level, average item level, zone and names (spec section 1). It carries no professions and no per-slot gear. The archived design lists "Full gear lists and, in v1, professions" as non-goals and "Professions and gear detail" under Future Improvements.

## Does it earn a place

Nothing in the finder uses these fields. Matching uses role, level, item level and faction (see `app/src/engine/` and spec section 3). A dungeon party does not need a profession to form. Adding fields has a cost: more bytes in a QR whose target is 122 characters (a typical record is about 70), a version bump that makes every user update, and fields that ADR 0003 says can be forged.

A use has to come first. Candidates, none of them in the ticket:

*   Showing party members' gear level or professions in the Confirm prompt, so a group sees who can, for instance, open locked chests. A display only; no matching change.
*   A gear floor as a search condition (item level already covers this).
*   Profession-based services (not matchmaking for dungeons; see [ticket 0016](../../../tickets/0016-other-matchmaking-uses.md)).

Unverified: which client calls give professions on Forever (retail uses `GetProfessions` and `GetProfessionInfo`, not checked here), and whether gear detail is readable for the player without inspect.

## If built: the shape

A version bump (`WFT2`, spec section 6). Professions: two entries of a skill ID (1 byte) and rank (2 bytes). Gear detail: at most one value per slot would be 18 slots, too large for the QR target, so a summary (for example item level per three groups, or missing-enchant count) is the only option that fits; that is an estimate to check.

Files that would change: `docs/rdf-code-spec.md`, `scripts/gen-code-vectors.py`, `docs/rdf-code-vectors.json`, `addon/ForeverTogether/Codec.lua` and `Core.lua`, `app/src/code.rs` (`Character`, decoder, test encoder), the engine `Ad` (`app/src/engine/ad.rs`) only if peers must see the data, `app/src/server/core.rs` `CharacterView`, the web types and a Storybook story for the display. An ADR amending 0018.

## Alternatives Considered

*   **Leave it out.** The current state.
*   **Send it only after a party forms, in `Ready`.** `Ready` already carries the sender's ad (spec section 3), so a later optional block could ride there with no change to the code or QR. Not in the ticket; noted as the cheaper route if the use is display only.

## Test and verification plan (only if built)

Live calls on a client (record in `docs/spike-s0.md`), regenerated vectors, `mise run test-addon`, `cargo test --lib code::`, and a size check that a typical record keeps QR version 5.

## Open questions for the author

1.  **Build now, later, or never?**
    *   Option A: not now; revisit when a feature needs the data. Cost: none. Fails if a user request comes before the format is next changed, and the bump is done twice.
    *   Option B: bundle it into the next unavoidable code bump (tickets 0017 or 0018). Cost: bytes added speculatively. Fails if the data turns out unused.
    *   Option C: build it alone now. Cost: a version bump for no matching gain. Fails for the reason in the section above.
2.  **What is it for?** (The answer decides the fields.) Option A: display in the Confirm prompt. Option B: a search condition. Option C: both. Option B fails if item level is already enough; option A fails if players ignore it.
3.  **Where does it travel?** Option A: in the code only, shown to the owner. Option B: in the `Ready` message only. Option C: in every `Ad`. Option C fails on gossip volume (ADR 0034: busy topics), option A fails if no one else can see it, option B fails if the party needs it before forming.
