import { type FC } from "react";

import { AlertCard } from "./AlertCard";
import { Rail } from "./Rail";
import type { Alert } from "./useAlerts";
import { useAlerts } from "./useAlerts";

export type Presentation = "takeover" | "sidebar" | "toast" | "agenda";

/** The agenda is the rail's content at the size of a wall, so it has no mode of its own. */
const shows = (presentation: Presentation, alert: Alert) => {
  if (presentation === "agenda") {
    return alert.mode === "sidebar";
  }

  return alert.mode === presentation;
};

export const Alerts: FC<{ presentation: Presentation; }> = ({ presentation }) => {
  const alerts = useAlerts().filter(alert => shows(presentation, alert));

  if (presentation === "sidebar" || presentation === "agenda") {
    return <Rail alerts={alerts} isWall={presentation === "agenda"} />;
  }

  if (alerts.length === 0) {
    return <main className="h-screen w-screen bg-gray-950" />;
  }

  // The card fills the window, which the daemon has already sized for one toast.
  if (presentation === "toast") {
    const [newest] = alerts.slice(-1);

    return (
      <main className="flex h-screen w-screen bg-gray-950">
        <AlertCard alert={newest} isLarge={false} />
      </main>
    );
  }

  const [newest] = alerts.slice(-1);

  return (
    <main className="flex h-screen w-screen flex-col items-center justify-center bg-gray-950 p-16">
      <AlertCard alert={newest} isLarge />
      {alerts.length > 1 && (
        <p className="mt-8 text-2xl text-gray-500">
          {`and ${alerts.length - 1} more`}
        </p>
      )}
    </main>
  );
};
