import classNames from "classnames";
import { type FC } from "react";

import type { Alert } from "./useAlerts";

const edge = {
  info: "bg-sky-500",
  warning: "bg-amber-500",
  critical: "bg-red-500",
} as const;

export const AlertCard: FC<{ alert: Alert; isLarge: boolean; }> = ({ alert, isLarge }) => (
  <article className={classNames("flex w-full bg-gray-900", isLarge && "max-w-5xl")}>
    {/* The level is a colour on one edge rather than a tint over the text, so the words stay
            at full contrast from across a room. */}
    <div className={classNames("w-2 shrink-0", edge[alert.level])} />
    <div className={isLarge ? "p-12" : "p-4"}>
      <h1 className={classNames("font-semibold text-gray-100", isLarge ? "text-7xl" : "text-xl")}>
        {alert.title}
      </h1>
      {alert.body && (
        <p className={classNames("mt-4 text-gray-400", isLarge ? "text-4xl" : "text-base")}>
          {alert.body}
        </p>
      )}
    </div>
  </article>
);
