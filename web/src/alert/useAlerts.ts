import { useEffect, useState } from "react";

import type { components } from "../api/schema.gen";

export type Alert = components["schemas"]["Notification"];

/**
 * Subscribes rather than polls. The rail is open for as long as there is something on it, which
 * for a calendar is most of a working day, and a request a second for that is a lot of asking for
 * a list that changes a few times an hour.
 *
 * The daemon sweeps on a timer of its own, so an entry that ends without anything else happening
 * still arrives here as a message.
 */
export const useAlerts = () => {
  const [alerts, setAlerts] = useState<Alert[]>([]);

  useEffect(() => {
    const source = new EventSource("/api/notifications/stream");

    source.addEventListener("message", (event) => {
      try {
        setAlerts(JSON.parse(event.data as string) as Alert[]);
      }
      catch {
        // A half-written frame is not worth reporting on the wall.
      }
    });

    return () => source.close();
  }, []);

  return alerts;
};
