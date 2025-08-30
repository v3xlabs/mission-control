import { FC } from "react";
import classnames from "classnames";
import { TabList } from "./TabList";
import { useActivatePlaylist } from "../hooks/useActivatePlaylist";
import { useCurrentPlaylist } from "../hooks/useCurrentPlaylist";

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
  const { data: currentPlaylist } = useCurrentPlaylist();
  
  // Use only the current playlist from status API as the single source of truth
  const isActive = currentPlaylist === playlist.id;
  
  return (
    <div
      className={classnames(
        "border rounded-lg p-4 shadow-sm transition-colors",
        isActive ? "border-green-500" : "border-gray-700"
      )}
    >
      <div className="flex items-center justify-between mb-2">
        <h2 className="text-xl font-semibold">{playlist.name}</h2>
        {isActive ? (
          <span className="px-2 py-1 text-xs bg-green-600 rounded">Active</span>
        ) : (
          <button
            onClick={() => activate.mutate(playlist.id)}
            className="px-2 py-1 text-xs bg-gray-700 rounded hover:bg-gray-600"
            disabled={activate.isPending}
          >
            {activate.isPending ? "Activating..." : "Activate"}
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