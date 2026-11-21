import type { HTMLAttributes, ReactNode } from "react";
import { Heading } from "./Typography";

export interface CardProps extends Omit<HTMLAttributes<HTMLElement>, "title"> {
  title?: ReactNode;
}

/** A bordered surface with an optional heading. The container every organism section sits in. */
export function Card({ title, children, className, ...rest }: CardProps) {
  return (
    <section className={["ft-card", className].filter(Boolean).join(" ")} {...rest}>
      {title && (
        <Heading level={2} className="ft-card__title">
          {title}
        </Heading>
      )}
      {children}
    </section>
  );
}
