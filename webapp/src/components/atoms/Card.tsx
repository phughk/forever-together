import type { HTMLAttributes, ReactNode } from "react";
import { InfoTip } from "./InfoTip";
import { Heading } from "./Typography";

export interface CardProps extends Omit<HTMLAttributes<HTMLElement>, "title"> {
  title?: ReactNode;
  /** Short text for an info tip beside the title. Needs a title. */
  hint?: string;
}

/** A bordered surface with an optional heading. The container every organism section sits in. */
export function Card({ title, hint, children, className, ...rest }: CardProps) {
  const heading = title && (
    <Heading level={2} className="ft-card__title">
      {title}
    </Heading>
  );
  return (
    <section className={["ft-card", className].filter(Boolean).join(" ")} {...rest}>
      {heading && hint ? (
        <div className="ft-card__titlerow">
          {heading}
          <InfoTip label={`About ${typeof title === "string" ? title : "this section"}`}>{hint}</InfoTip>
        </div>
      ) : (
        heading
      )}
      {children}
    </section>
  );
}
