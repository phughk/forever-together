import type { ElementType, HTMLAttributes, LabelHTMLAttributes } from "react";

export interface HeadingProps extends HTMLAttributes<HTMLHeadingElement> {
  /** Visual size, 1 largest. */
  level?: 1 | 2 | 3;
  /** Element to render. Defaults to the matching h1 to h3. */
  as?: ElementType;
}

export function Heading({ level = 2, as, className, ...rest }: HeadingProps) {
  const Tag = (as ?? `h${level}`) as ElementType;
  return <Tag className={["ft-heading", `ft-heading--${level}`, className].filter(Boolean).join(" ")} {...rest} />;
}

export interface TextProps extends HTMLAttributes<HTMLParagraphElement> {
  size?: "normal" | "small";
  muted?: boolean;
  mono?: boolean;
  as?: ElementType;
}

export function Text({ size = "normal", muted, mono, as: Tag = "p", className, ...rest }: TextProps) {
  const cls = ["ft-text", size === "small" && "ft-text--sm", muted && "ft-text--muted", mono && "ft-text--mono", className]
    .filter(Boolean)
    .join(" ");
  return <Tag className={cls} {...rest} />;
}

export function Label({ className, ...rest }: LabelHTMLAttributes<HTMLLabelElement>) {
  return <label className={["ft-label", className].filter(Boolean).join(" ")} {...rest} />;
}
