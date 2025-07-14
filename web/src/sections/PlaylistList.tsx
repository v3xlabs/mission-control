import { FC } from "react";
import { usePlaylists } from "../api/playlists";
import { PlaylistCard } from "../widgets/PlaylistCard";
import type { components } from "../api/schema.gen";

type PlaylistInfo = components["schemas"]["PlaylistInfo"];

export const PlaylistList: FC<{}> = ({}) => {
  const playlists = usePlaylists();

  if (playlists.isLoading) return <div>Loading...</div>;
  if (playlists.error || !playlists.data) return <div>Error loading playlists</div>;

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {playlists.data.map((pl: PlaylistInfo) => (
        <PlaylistCard key={pl.id} playlist={pl} />
      ))}
    </div>
  );
}; 