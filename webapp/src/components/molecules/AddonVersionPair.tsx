import "../../styles/settings.css";

export interface AddonVersionPairProps {
  /** Version found in the game's AddOns folder. Omit when nothing readable is installed. */
  inGame?: string | null;
  /** Version of the addon copy inside this app. */
  thisApp: string;
  /** Label of the app's value: "This app" beside a game version, "In this app" alone, "New" for an update, "Now" after an install. */
  thisAppLabel?: "This app" | "In this app" | "New" | "Now";
}

/** The versions as short labelled values, for example "In game: 0.1.0" and "This app: 0.2.0". Both show when equal. */
export function AddonVersionPair({ inGame, thisApp, thisAppLabel }: AddonVersionPairProps) {
  const label = thisAppLabel ?? (inGame ? "This app" : "In this app");
  return (
    <dl className="ft-addonversions">
      {inGame && (
        <div className="ft-addonversions__item">
          <dt>In game:</dt>
          <dd>{inGame}</dd>
        </div>
      )}
      <div className="ft-addonversions__item">
        <dt>{label}:</dt>
        <dd>{thisApp}</dd>
      </div>
    </dl>
  );
}
