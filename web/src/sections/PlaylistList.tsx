import { FC } from "react";
import { useQuery } from "@tanstack/react-query";
import classnames from "classnames";
import { PlaylistCard } from "../widgets/PlaylistCard";

interface PlaylistInfo {
  id: string;
  name: string;
  tab_count: number;
  interval_seconds: number;
  is_active: boolean;
}

const fetchPlaylists = async (): Promise<PlaylistInfo[]> => {
  const res = await fetch("/api/playlists");
  if (!res.ok) throw new Error("Failed to fetch playlists");
  return res.json();
};

export const PlaylistList: FC<{}> = ({}) => {
  const { data, error, isLoading } = useQuery({
    queryKey: ["playlists"],
    queryFn: fetchPlaylists,
    refetchInterval: 5000,
  });

  if (isLoading) return <div>Loading...</div>;
  if (error || !data) return <div>Error loading playlists</div>;

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {data.map((pl) => (
        <PlaylistCard key={pl.id} playlist={pl} />
      ))}
    </div>
  );
}; 