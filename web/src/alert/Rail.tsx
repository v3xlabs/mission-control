import classNames from "classnames";
import { type FC, Fragment } from "react";

import { RailNotice } from "./RailNotice";
import { RailSlot } from "./RailSlot";
import { scheduleOf } from "./schedule";
import type { Alert } from "./useAlerts";
import { useNow } from "./useNow";

/** Near enough for "in 4 min" to be true, far enough to leave the display alone. */
const TICK_MS = 15_000;

const clock = new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit" });
const today = new Intl.DateTimeFormat(undefined, { weekday: "long", day: "numeric", month: "long" });
const otherDay = new Intl.DateTimeFormat(undefined, { weekday: "long", day: "numeric" });

const isSameDay = (one: number, other: number) =>
  new Date(one).toDateString() === new Date(other).toDateString();

export const Rail: FC<{ alerts: Alert[]; isWall: boolean; }> = ({ alerts, isWall }) => {
  const now = useNow(TICK_MS);
  const notices = alerts.filter(alert => !alert.starts_at);
  const slots = scheduleOf(alerts);
  const next = slots.findIndex(slot => slot.startsAt > now);

  return (
    <main
      className={classNames(
        "flex h-screen w-screen flex-col bg-black",
        isWall ? "text-[28px]" : "text-[15px]",
      )}
    >
      <header className="flex shrink-0 items-baseline justify-between border-b border-white/10 px-[1.3em] py-[1.1em]">
        <p className="text-[0.8em] font-medium tracking-[0.18em] text-gray-500 uppercase">
          {today.format(now)}
        </p>
        <time className="text-[1.1em] font-medium tabular-nums text-gray-300">{clock.format(now)}</time>
      </header>

      {slots.length === 0 && notices.length === 0
        ? (
            <p className="flex flex-1 items-center justify-center text-[1.4em] text-gray-600">
              Nothing scheduled
            </p>
          )
        : (
            // The display has no pointer, so a scrollbar is decoration that covers a meeting.
            <ol className="min-h-0 flex-1 space-y-[0.5em] overflow-y-auto p-[0.7em] [scrollbar-width:none]">
              {notices.map(alert => <RailNotice key={alert.notification_id} alert={alert} />)}
              {slots.map((slot, index) => (
                <Fragment key={slot.startsAt}>
                  {index > 0 && !isSameDay(slots[index - 1].startsAt, slot.startsAt) && (
                    <li className="px-[0.7em] pt-[1.2em] pb-[0.3em] text-[0.8em] font-medium tracking-[0.18em] text-gray-600 uppercase">
                      {otherDay.format(slot.startsAt)}
                    </li>
                  )}
                  {index === next && (
                    <li aria-hidden className="flex items-center gap-[0.6em] px-[0.7em] py-[0.4em]">
                      <span className="size-[0.4em] shrink-0 rounded-full bg-white" />
                      <span className="h-px flex-1 bg-white/25" />
                    </li>
                  )}
                  <RailSlot slot={slot} now={now} />
                </Fragment>
              ))}
            </ol>
          )}
    </main>
  );
};
