import { useEffect, useReducer } from "react";

const relative = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });

const phrase = (startsAt: string) => {
  const minutes = Math.round((new Date(startsAt).getTime() - Date.now()) / 60_000);

  if (minutes <= 0 && minutes > -60) {
    return "now";
  }

  if (Math.abs(minutes) >= 60) {
    return relative.format(Math.round(minutes / 60), "hour");
  }

  return relative.format(minutes, "minute");
};

export const useCountdown = (startsAt: string | undefined) => {
  const [, tick] = useReducer((count: number) => count + 1, 0);

  useEffect(() => {
    if (!startsAt) {
      return;
    }

    const timer = setInterval(tick, 30_000);

    return () => clearInterval(timer);
  }, [startsAt]);

  return startsAt ? phrase(startsAt) : undefined;
};
