import type { ButtonHTMLAttributes } from "react";

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "default" | "primary" | "danger" | "ghost";
  size?: "normal" | "small";
  block?: boolean;
}

export function Button({ variant = "default", size = "normal", block, className, ...rest }: ButtonProps) {
  const cls = [
    "ft-button",
    variant !== "default" && `ft-button--${variant}`,
    size === "small" && "ft-button--small",
    block && "ft-button--block",
    className,
  ]
    .filter(Boolean)
    .join(" ");
  return <button type="button" className={cls} {...rest} />;
}
