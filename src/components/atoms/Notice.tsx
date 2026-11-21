import type { HTMLAttributes } from "react";

export interface NoticeProps extends HTMLAttributes<HTMLDivElement> {
  tone?: "info" | "warn" | "bad" | "good";
}

export function Notice({ tone = "info", className, ...rest }: NoticeProps) {
  return (
    <div
      role={tone === "bad" ? "alert" : "status"}
      className={["ft-notice", `ft-notice--${tone}`, className].filter(Boolean).join(" ")}
      {...rest}
    />
  );
}
