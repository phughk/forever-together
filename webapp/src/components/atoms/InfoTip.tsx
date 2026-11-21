import { useCallback, useEffect, useId, useLayoutEffect, useRef, useState } from "react";

/** The most characters a tip may hold. If dropping the tip would not change what the player does, it should have been a label or a visual. */
export const INFO_TIP_MAX_CHARS = 160;

export interface InfoTipProps {
  /** What the tip is about, e.g. "About phone link". Becomes the button's accessible name. */
  label: string;
  /** Plain text only, at most INFO_TIP_MAX_CHARS characters. */
  children: string;
  className?: string;
}

const MARGIN = 8;

/** A round "i" button that opens a small text popover. Closes on Escape, outside tap, or when focus leaves. */
export function InfoTip({ label, children, className }: InfoTipProps) {
  const [open, setOpen] = useState(false);
  const [pos, setPos] = useState<{ left: number; top: number } | null>(null);
  const wrapRef = useRef<HTMLSpanElement>(null);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const popRef = useRef<HTMLSpanElement>(null);
  const id = useId();

  if (import.meta.env.DEV && children.length > INFO_TIP_MAX_CHARS) {
    console.warn(`InfoTip "${label}": ${children.length} characters, over the ${INFO_TIP_MAX_CHARS} cap.`);
  }

  const close = useCallback(() => setOpen(false), []);

  // Position the popover (fixed, so it cannot shift layout or add scroll) and clamp it into the viewport.
  useLayoutEffect(() => {
    if (!open || !buttonRef.current || !popRef.current) {
      setPos(null);
      return;
    }
    const b = buttonRef.current.getBoundingClientRect();
    const p = popRef.current.getBoundingClientRect();
    const vw = document.documentElement.clientWidth;
    const vh = window.innerHeight;
    const centre = b.left + b.width / 2;
    const left = Math.max(MARGIN, Math.min(centre - p.width / 2, vw - p.width - MARGIN));
    const below = b.bottom + 4;
    const top = below + p.height + MARGIN > vh && b.top - 4 - p.height >= MARGIN ? b.top - 4 - p.height : below;
    setPos({ left, top });
  }, [open, children]);

  useEffect(() => {
    if (!open) return;
    const onPointer = (e: Event) => {
      if (!wrapRef.current?.contains(e.target as Node)) close();
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        close();
        buttonRef.current?.focus();
      }
    };
    document.addEventListener("pointerdown", onPointer);
    document.addEventListener("keydown", onKey);
    window.addEventListener("resize", close);
    window.addEventListener("scroll", close, true);
    return () => {
      document.removeEventListener("pointerdown", onPointer);
      document.removeEventListener("keydown", onKey);
      window.removeEventListener("resize", close);
      window.removeEventListener("scroll", close, true);
    };
  }, [open, close]);

  return (
    <span
      ref={wrapRef}
      className={["ft-infotip", className].filter(Boolean).join(" ")}
      onBlur={(e) => {
        if (!wrapRef.current?.contains(e.relatedTarget as Node | null)) close();
      }}
    >
      <button
        ref={buttonRef}
        type="button"
        className="ft-infotip__button"
        aria-label={label}
        aria-expanded={open}
        aria-describedby={open ? id : undefined}
        onClick={() => setOpen((o) => !o)}
      >
        <span className="ft-infotip__icon" aria-hidden="true">
          i
        </span>
      </button>
      {open && (
        <span
          ref={popRef}
          id={id}
          role="tooltip"
          className="ft-infotip__pop"
          style={pos ? { left: pos.left, top: pos.top } : { left: 0, top: 0, visibility: "hidden" }}
        >
          {children}
        </span>
      )}
    </span>
  );
}
