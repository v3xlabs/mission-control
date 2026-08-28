import classNames from "classnames";
import { type FC } from "react";
import { LuVideo } from "react-icons/lu";
import { SiGooglemeet, SiJitsi, SiWebex, SiZoom } from "react-icons/si";

import type { Alert } from "./useAlerts";

const icons: Record<string, FC<{ className?: string; }>> = {
  zoom: SiZoom,
  meet: SiGooglemeet,
  jitsi: SiJitsi,
  webex: SiWebex,
};

const brand: Record<string, string> = {
  zoom: "text-[#0b5cff]",
  meet: "text-[#00832d]",
  jitsi: "text-[#1d76ba]",
  webex: "text-[#00bceb]",
};

export const MeetingIcon: FC<{ meeting: NonNullable<Alert["meeting"]>; className?: string; }> = ({
  meeting,
  className,
}) => {
  const provider = meeting.provider ?? "";
  const Icon = icons[provider] ?? LuVideo;

  return (
    <Icon
      className={classNames(brand[provider] ?? "text-gray-500", className)}
      aria-label={provider || "meeting"}
    />
  );
};
