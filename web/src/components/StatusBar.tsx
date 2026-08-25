import { type FC, useState } from "react";
import { FiChevronLeft, FiChevronRight, FiMonitor, FiPause, FiPlay } from "react-icons/fi";

import { adminKey, setAdminKey } from "../api/auth";
import { useDisplayPower } from "../hooks/useDisplayPower";
import { usePlayback } from "../hooks/usePlayback";
import { useStatus } from "../hooks/useStatus";

export const StatusBar: FC = () => {
  const { data: status } = useStatus();
  const playback = usePlayback();
  const power = useDisplayPower();
  const [key, setKey] = useState(adminKey());

  if (!status) {
    return <header className="border-b border-gray-800 px-4 py-3 text-gray-500">Connecting...</header>;
  }

  return (
    <header className="flex flex-wrap items-center gap-3 border-b border-gray-800 px-4 py-3">
      <h1 className="font-semibold text-gray-100">{status.device_name}</h1>

      <span className="text-xs text-gray-500">
        {status.current_tab_id ?? "nothing on screen"}
      </span>

      {status.config_read_only && (
        <span
          className="bg-amber-700 px-1.5 py-0.5 text-xs text-white"
          title="The config directory is managed elsewhere. Changes apply now but do not survive a restart."
        >
          config from Nix
        </span>
      )}

      <span className="flex-1" />

      <div className="flex items-center gap-2 text-gray-300">
        <button type="button" onClick={() => playback.mutate("previous")} title="Previous tab">
          <FiChevronLeft />
        </button>
        <button
          type="button"
          onClick={() => playback.mutate(status.auto_rotate ? "pause" : "resume")}
          title={status.auto_rotate ? "Pause rotation" : "Resume rotation"}
        >
          {status.auto_rotate ? <FiPause /> : <FiPlay />}
        </button>
        <button type="button" onClick={() => playback.mutate("next")} title="Next tab">
          <FiChevronRight />
        </button>
        <button
          type="button"
          onClick={() => power.mutate(!status.screen_on)}
          title={status.screen_on ? "Turn the screen off" : "Turn the screen on"}
          className={status.screen_on ? "text-emerald-400" : "text-gray-600"}
        >
          <FiMonitor />
        </button>
      </div>

      <input
        type="password"
        value={key}
        onChange={(event) => {
          setKey(event.target.value);
          setAdminKey(event.target.value);
        }}
        placeholder="admin key"
        className="w-40 border border-gray-800 bg-gray-900 px-2 py-1 text-sm text-gray-100"
      />
    </header>
  );
};
