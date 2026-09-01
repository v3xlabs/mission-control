import classNames from "classnames";
import { type FC } from "react";

import { clock } from "./clock";
import { MeetingIcon } from "./MeetingIcon";
import type { Nearness, Placed } from "./schedule";
import { nearnessOf, percentOf } from "./schedule";

type Tone = {
  surface: string;
  title: string;
  meta: string;
};

/** Distance is drawn by the axis, so a tone only has to say how soon, not how far. */
const tones: Record<Nearness, Tone> = {
  starting: {
    surface: "bg-amber-400/15",
    title: "text-white",
    meta: "text-amber-200",
  },
  soon: {
    surface: "bg-white/8",
    title: "text-gray-100",
    meta: "text-gray-400",
  },
  later: {
    surface: "bg-white/5",
    title: "text-gray-300",
    meta: "text-gray-500",
  },
};

export const RailEntry: FC<{ entry: Placed; from: number; spanMs: number; now: number; }> = ({
  entry,
  from,
  spanMs,
  now,
}) => {
  const nearness = nearnessOf(entry.startsAt, now);
  const tone = tones[nearness];
  const { alert } = entry;

  const details = [];

  if (nearness === "starting") {
    details.push(`in ${Math.max(1, Math.round((entry.startsAt - now) / 60_000))} min`);
  }

  details.push(
    entry.endsAt > entry.startsAt
      ? `${clock.format(entry.startsAt)} to ${clock.format(entry.endsAt)}`
      : clock.format(entry.startsAt),
  );

  if (alert.location) {
    details.push(alert.location);
  }

  return (
    <li
      className="rail-entry absolute"
      style={{
        top: `${percentOf(entry.startsAt, from, spanMs)}%`,
        height: `${percentOf(entry.endsAt, from, spanMs) - percentOf(entry.startsAt, from, spanMs)}%`,
        left: `${(entry.lane / entry.lanes) * 100}%`,
        width: `${(1 / entry.lanes) * 100}%`,
        // A quarter of an hour is four pixels of honest height, and no title fits in four pixels.
        minHeight: "2.6em",
      }}
    >
      <div className={classNames("mr-[0.3em] mb-[0.15em] h-full overflow-hidden px-[0.6em] py-[0.3em]", tone.surface)}>
        <div className="flex items-center gap-[0.45em]">
          {alert.meeting && <MeetingIcon meeting={alert.meeting} className="size-[1.1em] shrink-0" />}
          <h2 className={classNames("truncate text-[1.05em] font-semibold", tone.title)}>{alert.title}</h2>
        </div>
        <p className={classNames("rail-entry-detail mt-[0.1em] gap-[0.7em] text-[0.8em] tabular-nums", tone.meta)}>
          {details.map(detail => <span key={detail} className="truncate">{detail}</span>)}
        </p>
      </div>
    </li>
  );
};
