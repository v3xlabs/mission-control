import classNames from "classnames";
import { type FC } from "react";

import { MeetingIcon } from "./MeetingIcon";
import type { Nearness, Slot } from "./schedule";
import { nearnessOf, phraseOf } from "./schedule";

const clock = new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit" });

type Emphasis = {
  row: string;
  time: string;
  phrase: string;
  title: string;
  icon: string;
  meta: string;
};

/** Sizes are in `em` so the rail and the full-screen agenda are one design at two scales. */
const emphasis: Record<Nearness, Emphasis> = {
  live: {
    row: "bg-white/10",
    time: "text-white",
    phrase: "text-white",
    title: "text-[1.35em] font-semibold text-white",
    icon: "size-[1.3em]",
    meta: "text-gray-300",
  },
  starting: {
    row: "bg-amber-400/15",
    time: "text-amber-200",
    phrase: "text-amber-300",
    title: "text-[1.35em] font-semibold text-white",
    icon: "size-[1.3em]",
    meta: "text-gray-300",
  },
  soon: {
    row: "",
    time: "text-gray-300",
    phrase: "text-gray-500",
    title: "text-[1.15em] font-medium text-gray-100",
    icon: "size-[1.15em]",
    meta: "text-gray-500",
  },
  later: {
    row: "",
    time: "text-gray-600",
    phrase: "",
    title: "text-[1em] font-normal text-gray-400",
    icon: "size-[1em]",
    meta: "text-gray-600",
  },
};

const detailsOf = (endsAt: string | undefined, location: string | undefined) => {
  const details: string[] = [];

  if (endsAt) {
    details.push(`until ${clock.format(new Date(endsAt))}`);
  }

  if (location) {
    details.push(location);
  }

  return details;
};

export const RailSlot: FC<{ slot: Slot; now: number; }> = ({ slot, now }) => {
  const style = emphasis[nearnessOf(slot.startsAt, now)];
  const phrase = phraseOf(slot.startsAt, now);

  return (
    <li
      className={classNames(
        "grid grid-cols-[5.6em_1fr] items-start gap-[0.9em] rounded-[0.4em] px-[0.7em] py-[0.6em]",
        style.row,
      )}
    >
      <div>
        <time
          dateTime={new Date(slot.startsAt).toISOString()}
          className={classNames("block text-[1.1em] leading-none font-medium whitespace-nowrap tabular-nums", style.time)}
        >
          {clock.format(slot.startsAt)}
        </time>
        {phrase && (
          <p className={classNames("mt-[0.6em] text-[0.75em] tracking-[0.14em] uppercase", style.phrase)}>
            {phrase}
          </p>
        )}
      </div>
      {/* One hairline per extra meeting: what shares a time still has to read as two things. */}
      <ul className="min-w-0 divide-y divide-white/10">
        {slot.alerts.map((alert) => {
          const details = detailsOf(alert.ends_at, alert.location);

          return (
            <li key={alert.notification_id} className="min-w-0 py-[0.4em] first:pt-0 last:pb-0">
              <div className="flex items-center gap-[0.5em]">
                {alert.meeting && (
                  <MeetingIcon meeting={alert.meeting} className={classNames("shrink-0", style.icon)} />
                )}
                <h2 className={classNames("line-clamp-2 min-w-0", style.title)}>{alert.title}</h2>
              </div>
              {details.length > 0 && (
                <p className={classNames("mt-[0.3em] flex gap-[0.8em] text-[0.85em]", style.meta)}>
                  {details.map(detail => <span key={detail} className="truncate">{detail}</span>)}
                </p>
              )}
            </li>
          );
        })}
      </ul>
    </li>
  );
};
