import { useEffect, useState } from "react";

/** One clock for the whole rail, so every entry reads the same moment and moves on one tick. */
export const useNow = (intervalMs: number) => {
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), intervalMs);

    return () => clearInterval(timer);
  }, [intervalMs]);

  return now;
};
