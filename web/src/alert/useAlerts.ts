import { useEffect, useState } from "react";

export type Alert = {
  notification_id: number;
  title: string;
  body?: string;
  level: "info" | "warning" | "critical";
  expires_in_seconds: number;
  tab_id?: string;
};

/**
 * Polls rather than subscribing. This page is opened for the seconds an alert is up and closed
 * again, so a stream would spend its life reconnecting for no gain.
 */
export const useAlerts = () => {
  const [alerts, setAlerts] = useState<Alert[]>([]);

  useEffect(() => {
    let isCancelled = false;

    const read = async () => {
      try {
        const response = await fetch("/api/notifications");

        if (!isCancelled && response.ok) {
          setAlerts(await response.json() as Alert[]);
        }
      }
      catch {
        // A daemon that is restarting is not worth reporting on the wall.
      }
    };

    read();

    const timer = setInterval(read, 1000);

    return () => {
      isCancelled = true;
      clearInterval(timer);
    };
  }, []);

  return alerts;
};
