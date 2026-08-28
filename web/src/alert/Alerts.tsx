import { type FC } from "react";

import { AlertCard } from "./AlertCard";
import type { Alert } from "./useAlerts";
import { useAlerts } from "./useAlerts";

type Presentation = Alert["mode"];

/**
 * One page serves all three presentations, and each shows only what is addressed to it. Rendering
 * the whole list everywhere puts the agenda underneath a doorbell alert and counts the wrong
 * things in "and 2 more".
 */
export const Alerts: FC<{ presentation: Presentation; }> = ({ presentation }) => {
  const alerts = useAlerts().filter(alert => alert.mode === presentation);

  if (alerts.length === 0) {
    return <main className="h-screen w-screen bg-gray-950" />;
  }

  if (presentation === "sidebar") {
    return (
      <main className="flex h-screen w-screen flex-col gap-3 overflow-y-auto bg-gray-950 p-4">
        {alerts.map(alert => <AlertCard key={alert.notification_id} alert={alert} isLarge={false} />)}
      </main>
    );
  }

  // A toast is one thing in a corner. The card fills the window, because the daemon sized that
  // window for it and a card floating inside leaves bands of background around the message.
  if (presentation === "toast") {
    const [newest] = alerts.slice(-1);

    return (
      <main className="flex h-screen w-screen bg-gray-950">
        <AlertCard alert={newest} isLarge={false} />
      </main>
    );
  }

  // A takeover shows one thing. Anything else queued is a count, not a second card competing
  // for the same wall.
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
