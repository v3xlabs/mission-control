import type { Alert } from "./useAlerts";

/** Two meetings that start together are one entry on the rail, under one time. */
export type Slot = {
  startsAt: number;
  alerts: Alert[];
};

/** How near a meeting is, which is what the rail says with size, weight and contrast. */
export type Nearness = "live" | "starting" | "soon" | "later";

const STARTING_MS = 5 * 60_000;
const SOON_MS = 60 * 60_000;

export const scheduleOf = (alerts: Alert[]): Slot[] => {
  const slots = new Map<number, Alert[]>();

  for (const alert of alerts) {
    if (!alert.starts_at) {
      continue;
    }

    const startsAt = new Date(alert.starts_at).getTime();
    const together = slots.get(startsAt);

    if (together) {
      together.push(alert);
    }
    else {
      slots.set(startsAt, [alert]);
    }
  }

  return [...slots]
    .map(([startsAt, together]) => ({ startsAt, alerts: together }))
    .toSorted((one, other) => one.startsAt - other.startsAt);
};

export const nearnessOf = (startsAt: number, now: number): Nearness => {
  const untilMs = startsAt - now;

  if (untilMs <= 0) {
    return "live";
  }

  if (untilMs <= STARTING_MS) {
    return "starting";
  }

  if (untilMs <= SOON_MS) {
    return "soon";
  }

  return "later";
};

/**
 * A phrase is what a countdown is for, so it stops where a countdown stops being one. Beyond the
 * hour the clock time is the answer, and the rail says how far away it is by how quiet it draws it.
 */
export const phraseOf = (startsAt: number, now: number): string | undefined => {
  const nearness = nearnessOf(startsAt, now);

  if (nearness === "live") {
    return "now";
  }

  if (nearness === "later") {
    return undefined;
  }

  return `in ${Math.max(1, Math.round((startsAt - now) / 60_000))} min`;
};
