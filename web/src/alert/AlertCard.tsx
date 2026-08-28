import classNames from "classnames";
import { type FC } from "react";

import { MeetingIcon } from "./MeetingIcon";
import type { Alert } from "./useAlerts";
import { useCountdown } from "./useCountdown";

const edge = {
  info: "bg-sky-500",
  warning: "bg-amber-500",
  critical: "bg-red-500",
} as const;

const clock = new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit" });

export const AlertCard: FC<{ alert: Alert; isLarge: boolean; isWide?: boolean; }> = ({
  alert,
  isLarge,
  isWide = false,
}) => {
  const relative = useCountdown(alert.starts_at);

  if (isWide) {
    return (
      <article className="flex w-full items-center bg-gray-900">
        <div className={classNames("w-2 self-stretch shrink-0", edge[alert.level])} />
        <div className="w-64 shrink-0 p-6">
          {alert.starts_at && (
            <>
              <p className="text-4xl font-medium text-gray-200">
                {clock.format(new Date(alert.starts_at))}
              </p>
              {relative && <p className="mt-1 text-xl text-gray-500">{relative}</p>}
            </>
          )}
        </div>
        <div className="min-w-0 py-6 pr-6">
          <div className="flex items-center gap-4">
            {alert.meeting && <MeetingIcon meeting={alert.meeting} className="size-8 shrink-0" />}
            <h1 className="truncate text-4xl font-semibold text-gray-100">{alert.title}</h1>
          </div>
          {alert.location && <p className="mt-2 text-2xl text-gray-500">{alert.location}</p>}
        </div>
      </article>
    );
  }

  return (
    <article className={classNames("flex w-full bg-gray-900", isLarge && "max-w-5xl")}>
      <div className={classNames("w-2 shrink-0", edge[alert.level])} />
      <div className={classNames("min-w-0", isLarge ? "p-12" : "p-4")}>
        {alert.starts_at && (
          <p className={classNames(
            "font-medium text-gray-400",
            isLarge ? "mb-3 text-3xl" : "mb-1 text-sm",
          )}
          >
            {clock.format(new Date(alert.starts_at))}
            {relative && <span className="ml-3 text-gray-500">{relative}</span>}
          </p>
        )}
        <div className={classNames("flex items-center", isLarge ? "gap-5" : "gap-2")}>
          {alert.meeting && (
            <MeetingIcon
              meeting={alert.meeting}
              className={classNames("shrink-0", isLarge ? "size-14" : "size-5")}
            />
          )}
          <h1 className={classNames("truncate font-semibold text-gray-100", isLarge ? "text-7xl" : "text-xl")}>
            {alert.title}
          </h1>
        </div>
        {alert.body && (
          <p className={classNames("mt-4 text-gray-400", isLarge ? "text-4xl" : "text-base")}>
            {alert.body}
          </p>
        )}
        {alert.location && (
          <p className={classNames("text-gray-500", isLarge ? "mt-4 text-3xl" : "mt-1 text-sm")}>
            {alert.location}
          </p>
        )}
      </div>
    </article>
  );
};
