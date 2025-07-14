import { FC } from "react";
import { useQuery } from "@tanstack/react-query";
import classnames from "classnames";

interface TabInfo {
  id: string;
  name: string;
  url: string;
  order_index: number;
  persist: boolean;
}

const fetchTabs = async (playlistId: string): Promise<TabInfo[]> => {
  const res = await fetch(`/api/playlists/${playlistId}/tabs`);
  if (!res.ok) throw new Error("Failed to fetch tabs");
  return res.json();
};

interface Props {
  playlistId: string;
}

export const TabList: FC<Props> = ({ playlistId }) => {
  const { data, error, isLoading } = useQuery({
    queryKey: ["tabs", playlistId],
    queryFn: () => fetchTabs(playlistId),
    refetchInterval: 5000,
  });

  if (isLoading) return <div>Loading tabs...</div>;
  if (error || !data) return <div>Error loading tabs</div>;

  return (
    <ul className="space-y-1">
      {data.map((tab) => (
        <li
          key={tab.id}
          className={classnames(
            "flex items-center justify-between py-1 px-2 rounded hover:bg-gray-800",
            tab.persist ? "text-white" : "text-gray-400"
          )}
        >
          <span>{tab.name}</span>
          <a
            href={tab.url}
            className="text-xs text-blue-400 hover:underline"
            target="_blank"
            rel="noopener noreferrer"
          >
            open
          </a>
        </li>
      ))}
    </ul>
  );
}; 