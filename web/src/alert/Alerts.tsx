import { type FC } from "react";

import { AlertCard } from "./AlertCard";
import { useAlerts } from "./useAlerts";

/**
 * The same page serves both presentations. As a takeover it fills the screen; as a sidebar it is
 * its own window and the compositor decides how wide that is, so it only has to stack.
 */
export const Alerts: FC<{ isSidebar: boolean; }> = ({ isSidebar }) => {
  const alerts = useAlerts();

  if (alerts.length === 0) {
    return <main className="h-screen w-screen bg-gray-950" />;
  }

  if (isSidebar) {
    return (
      <main className="flex h-screen w-screen flex-col gap-3 overflow-y-auto bg-gray-950 p-4">
        {alerts.map(alert => <AlertCard key={alert.notification_id} alert={alert} isLarge={false} />)}
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
