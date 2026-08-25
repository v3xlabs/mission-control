import { type FC } from "react";

import { usePlaylists } from "../api/playlists";
import { CreatePlaylistDialog } from "../components/CreatePlaylistDialog";
import { PlaylistCard } from "../components/PlaylistCard";

export const PlaylistList: FC = () => {
  const { data: playlists, isLoading, error } = usePlaylists();

  if (isLoading) {
    return <p className="p-4 text-gray-500">Loading playlists...</p>;
  }

  if (error || !playlists) {
    return <p className="p-4 text-red-400">Could not load playlists.</p>;
  }

  return (
    <div className="flex flex-col gap-6 p-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold text-gray-100">Playlists</h2>
        <CreatePlaylistDialog />
      </div>

      {playlists.length === 0
        ? <p className="text-gray-500">No playlists configured.</p>
        : playlists.map(playlist => (
            <PlaylistCard key={playlist.playlist_id} playlist={playlist} />
          ))}
    </div>
  );
};
