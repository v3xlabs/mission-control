import classNames from "classnames";
import { type FC } from "react";

import { clock } from "./clock";
import { RailEntry } from "./RailEntry";
import { RailNotice } from "./RailNotice";
import { RailRunning } from "./RailRunning";
import { percentOf, placed, spanOf, ticksOf, timedOf } from "./schedule";
import type { Alert } from "./useAlerts";
import { useNow } from "./useNow";

/** Near enough for "in 4 min" to be true, far enough to leave the display alone. */
const TICK_MS = 15_000;

const today = new Intl.DateTimeFormat(undefined, { weekday: "long", day: "numeric", month: "long" });
const otherDay = new Intl.DateTimeFormat(undefined, { weekday: "long" });

export const Rail: FC<{ alerts: Alert[]; isWall: boolean; }> = ({ alerts, isWall }) => {
  const now = useNow(TICK_MS);
  const notices = alerts.filter(alert => !alert.starts_at);
  const timed = timedOf(alerts);
  const running = timed.filter(entry => entry.startsAt <= now);
  const ahead = timed.filter(entry => entry.startsAt > now);
  const spanMs = spanOf(ahead, now);
  const ticks = ticksOf(now, spanMs);

  return (
    <main
      className={classNames(
        "flex h-screen w-screen flex-col bg-black",
        isWall ? "text-[24px]" : "text-[15px]",
      )}
    >
      <header className="flex shrink-0 items-baseline justify-between px-[1.3em] pt-[1.1em] pb-[1em]">
        <p className="text-[0.8em] font-medium tracking-[0.18em] text-gray-500 uppercase">
          {today.format(now)}
        </p>
        <time className="text-[1.1em] font-medium tabular-nums text-gray-300">{clock.format(now)}</time>
      </header>

      {notices.length > 0 && (
        <ul className="shrink-0 px-[0.6em] pb-[0.6em]">
          {notices.map(alert => <RailNotice key={alert.notification_id} alert={alert} />)}
        </ul>
      )}

      {running.map(entry => (
        <RailRunning key={entry.alert.notification_id} entry={entry} now={now} />
      ))}

      {ahead.length === 0
        ? (
            <p className="flex flex-1 items-center justify-center text-[1.2em] text-gray-600">
              {running.length > 0 ? "Nothing else today" : "Nothing scheduled"}
            </p>
          )
        : (
            <div className="relative min-h-0 flex-1 px-[0.7em] pb-[0.7em]">
              <time className="absolute top-0 left-[0.7em] text-[0.78em] font-semibold text-white tabular-nums">
                {clock.format(now)}
              </time>

              <ol>
                {ticks.map(at => (
                  <li
                    key={at}
                    className="absolute inset-x-[0.7em] flex items-center gap-[0.6em]"
                    style={{ top: `${percentOf(at, now, spanMs)}%` }}
                  >
                    <time className="w-[4.6em] shrink-0 text-[0.78em] whitespace-nowrap text-gray-600 tabular-nums">
                      {new Date(at).getHours() === 0 ? otherDay.format(at) : clock.format(at)}
                    </time>
                    <span className="h-px flex-1 bg-white/8" />
                  </li>
                ))}
              </ol>

              {/* The gutter is the axis, so the entries hang to the right of every hour label. */}
              <ol className="absolute top-0 right-[0.7em] bottom-[0.7em] left-[5.9em]">
                {placed(ahead).map(entry => (
                  <RailEntry
                    key={entry.alert.notification_id}
                    entry={entry}
                    from={now}
                    spanMs={spanMs}
                    now={now}
                  />
                ))}
              </ol>
            </div>
          )}
    </main>
  );
};
