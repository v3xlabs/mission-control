import { useEffect, useState } from "react";

import type { components } from "../api/schema.gen";

export type Alert = components["schemas"]["Notification"];

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
