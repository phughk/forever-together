import { useEffect, useRef } from "react";
import gongUrl from "../assets/sounds/singing-bowl-gong.wav";
import type { PartyView } from "../api/types";

let gong: HTMLAudioElement | null = null;

function audio(): HTMLAudioElement {
  if (!gong) {
    gong = new Audio(gongUrl);
    gong.preload = "auto";
  }
  return gong;
}

/** Browsers refuse to play sound until the page has had a tap or click. Pressing Search is
 * one, so play the clip once, muted, on the first interaction to unlock it for later. */
function unlockOnFirstInteraction() {
  const unlock = () => {
    const a = audio();
    a.muted = true;
    a.play()
      .then(() => {
        a.pause();
        a.currentTime = 0;
      })
      .catch(() => undefined)
      .finally(() => {
        a.muted = false;
      });
    window.removeEventListener("pointerdown", unlock);
    window.removeEventListener("keydown", unlock);
  };
  window.addEventListener("pointerdown", unlock);
  window.addEventListener("keydown", unlock);
  return () => {
    window.removeEventListener("pointerdown", unlock);
    window.removeEventListener("keydown", unlock);
  };
}

/** Play the gong once when a group forms after a search: a party this page has not seen yet that is
 * not already dropped. A party that is already there when the page first gets state (a reload,
 * a phone joining) stays silent. */
export function useGroupFormedSound(hasState: boolean, party: PartyView | null) {
  const seen = useRef<{ first: boolean; id: string | null }>({ first: true, id: null });

  useEffect(unlockOnFirstInteraction, []);

  useEffect(() => {
    if (!hasState) return;
    const s = seen.current;
    const id = party?.id ?? null;
    if (s.first) {
      s.first = false;
      s.id = id;
      return;
    }
    if (id === s.id) return;
    s.id = id;
    if (party && party.phase !== "dropped") {
      const a = audio();
      a.currentTime = 0;
      void a.play().catch(() => undefined);
    }
  }, [hasState, party]);
}
