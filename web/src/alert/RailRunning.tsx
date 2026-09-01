import { type FC } from "react";

import { clock } from "./clock";
import { MeetingIcon } from "./MeetingIcon";
import type { Timed } from "./schedule";
import { leftPhrase } from "./schedule";

/**
 * The meeting on now is lifted off the axis. Its start is behind us, so the question it has to
 * answer is how much of it is left, which a bar answers at a glance and the axis cannot.
 */
export const RailRunning: FC<{ entry: Timed; now: number; }> = ({ entry, now }) => {
  const { alert } = entry;
  const length = entry.endsAt - entry.startsAt;
  const elapsed = length > 0 ? Math.min(100, ((now - entry.startsAt) / length) * 100) : undefined;

  return (
    <article className="shrink-0 px-[1.3em] pb-[1.2em]">
      <div className="flex items-center gap-[0.5em]">
        {alert.meeting && <MeetingIcon meeting={alert.meeting} className="size-[1.3em] shrink-0" />}
        <h2 className="truncate text-[1.45em] leading-tight font-semibold text-white">{alert.title}</h2>
      </div>
      <p className="mt-[0.25em] flex gap-[0.8em] text-[0.85em] tabular-nums text-gray-400">
        {elapsed !== undefined && <span>{leftPhrase(entry.endsAt, now)}</span>}
        <span>{`until ${clock.format(entry.endsAt)}`}</span>
        {alert.location && <span className="truncate">{alert.location}</span>}
      </p>
      {elapsed !== undefined && (
        <div className="mt-[0.7em] h-[0.2em] bg-white/15">
          <div className="h-full bg-white" style={{ width: `${elapsed}%` }} />
        </div>
      )}
    </article>
  );
};
