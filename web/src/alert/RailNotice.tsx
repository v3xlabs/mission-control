import classNames from "classnames";
import { type FC } from "react";

import type { Alert } from "./useAlerts";

const dot: Record<Alert["level"], string> = {
  info: "bg-sky-400",
  warning: "bg-amber-400",
  critical: "bg-red-500",
};

/** An alert with no time of its own: the feed that stopped answering, or something pushed by hand. */
export const RailNotice: FC<{ alert: Alert; }> = ({ alert }) => (
  <li className="flex items-start gap-[0.7em] px-[0.7em] py-[0.5em]">
    <span className={classNames("mt-[0.45em] size-[0.45em] shrink-0 rounded-full", dot[alert.level])} />
    <div className="min-w-0">
      <h2 className="line-clamp-2 text-[1em] font-medium text-gray-200">{alert.title}</h2>
      {alert.body && <p className="mt-[0.2em] line-clamp-2 text-[0.85em] text-gray-500">{alert.body}</p>}
    </div>
  </li>
);
