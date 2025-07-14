import { FC } from "react";
import classnames from "classnames";
import { TabList } from "./TabList";
import { useActivatePlaylist } from "../hooks/useActivatePlaylist";

interface PlaylistInfo {
  id: string;
  name: string;
  tab_count: number;
  interval_seconds: number;
  is_active: boolean;
}

interface Props {
  playlist: PlaylistInfo;
}

export const PlaylistCard: FC<Props> = ({ playlist }) => {
  const activate = useActivatePlaylist();
  return (
    <div
      className={classnames(
        "border rounded-lg p-4 shadow-sm transition-colors",
        playlist.is_active ? "border-green-500" : "border-gray-700"
      )}
    >
      <div className="flex items-center justify-between mb-2">
        <h2 className="text-xl font-semibold">{playlist.name}</h2>
        {playlist.is_active ? (
          <span className="px-2 py-1 text-xs bg-green-600 rounded">Active</span>
        ) : (
          <button
            onClick={() => activate.mutate(playlist.id)}
            className="px-2 py-1 text-xs bg-blue-600 rounded hover:bg-blue-700"
          >
            Activate
          </button>
        )}
      </div>
      <p className="text-sm text-gray-400 mb-2">
        {playlist.tab_count} tabs &middot; interval {playlist.interval_seconds}s
      </p>
      <TabList playlistId={playlist.id} />
    </div>
  );
}; 