import { FC } from "react";
import { usePlaylists } from "../api/playlists";
import { PlaylistCard } from "../widgets/PlaylistCard";
import { EnhancedPlaylistCard } from "../components/EnhancedPlaylistCard";
import { CreatePlaylistDialog } from "../components/CreatePlaylistDialog";
import type { components } from "../api/schema.gen";

type PlaylistInfo = components["schemas"]["PlaylistInfo"];

export const PlaylistList: FC<{}> = ({}) => {
  const playlists = usePlaylists();

  if (playlists.isLoading) return <div className="text-center text-gray-400 py-8">Loading...</div>;
  if (playlists.error || !playlists.data) return <div className="text-center text-red-400 py-8">Error loading playlists</div>;

  return (
    <div className="space-y-6">
      {/* Header with create button */}
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold text-gray-100">Playlists</h2>
        <CreatePlaylistDialog />
      </div>

      {/* Playlists grid */}
      {playlists.data.length > 0 ? (
        <div className="space-y-6">
          {playlists.data.map((pl: PlaylistInfo) => (
            <EnhancedPlaylistCard key={pl.id} playlist={pl} />
          ))}
        </div>
      ) : (
        <div className="text-center text-gray-400 py-12">
          <p className="text-lg mb-4">No playlists found</p>
          <CreatePlaylistDialog trigger={
            <button className="px-6 py-3 bg-gray-700 text-white rounded-lg hover:bg-gray-600 transition-colors">
              Create Your First Playlist
            </button>
          } />
        </div>
      )}
    </div>
  );
}; 