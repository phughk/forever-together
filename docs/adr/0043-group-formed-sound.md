# 0043. A WAV gong plays in the web app when a group forms

* **Status:** Accepted (decided by the author 2026-11-24; built the same day)
* **Date:** 2026-11-24
* **Deciders:** Hugh Kaznowski

## Context

A search can run for a minute or more, and the player is often in the game with this app in the background. The party-ready prompt has a short Confirm window, so the player needs to hear it.

## Decision

1. The sound is played by the web app (`webapp/src/lib/useGroupFormedSound.ts`), not by the Rust side, so the desktop window and phones behave the same and the app needs no audio library (an audio crate would add ALSA development packages on Linux and undo ADR 0042).
2. The format is WAV, 16-bit PCM: every browser and webview plays it with no codec. The file is the author's `singing-bowl-gong.wav` (8 s, 32 kHz, stereo, about 1 MB) in `webapp/src/assets/sounds/`, bundled by Vite and so embedded in the binary.
3. It plays once when a party this page has not seen before appears and is not already dropped (`confirming` or `final`). A party present when the page first receives state (a reload, a phone joining) stays silent.
4. Browsers block sound until the page has had a tap or click. The hook unlocks playback with one muted play on the first pointer or key event; pressing Search is such an event.

## Consequences

* The binary grows by about 1 MB. A mono 22 kHz copy would be about 175 kB if that matters.
* There is no mute or volume setting yet (ticket 0050).
* A phone tab opened and never touched stays silent until its first tap.
* Not verified: playback in the Tauri window on Linux (WebKitGTK plays audio through GStreamer, and WAV needs only its base plugins), and on a phone.
