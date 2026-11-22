# 0018-zone-matching-localisation: match zones on map IDs

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0018](../../../tickets/0018-zone-matching-localisation.md)
* **ADRs:** [0006](../../adr/0006-dungeon-table.md), [0018](../../adr/0018-code-format.md)
* **Rounds:** none yet

## Question

How the app matches a character's zone to a dungeon's zone when the client reports a translated zone name. The code carries a zone name; the dungeon table holds English names; players on other client languages get no same-zone ordering.

## Files

* `design_draft.md`
