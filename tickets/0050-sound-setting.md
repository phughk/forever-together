# 0050. A mute and volume setting for the group-formed sound

* **Status:** open
* **Kind:** feature
* **Priority:** P2
* **Source:** ADR 0043
* **ADRs:** [ADR 0043](../docs/adr/0043-group-formed-sound.md)
* **EGM:** none

## What

The gong plays whenever a group forms, with no way to turn it off or lower it. Add a setting (Settings page and the settings API) and apply it in `webapp/src/lib/useGroupFormedSound.ts`. Also check playback in the Tauri window on Linux and on a phone, and consider a smaller mono copy of the file.

## Done when

The player can mute the sound, and playback is confirmed on desktop and on a phone.
