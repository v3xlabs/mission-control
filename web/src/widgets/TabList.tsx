import { FC, useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import classnames from "classnames";
import { useStatus } from "../hooks/useStatus";
import { useActivateTab } from "../hooks/useActivateTab";

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
  const { data: status } = useStatus();
  const currentTabId =
    status && status.current_playlist === playlistId ? status.current_tab : null;

  // existing query fetch
  const { data, error, isLoading } = useQuery({
    queryKey: ["tabs", playlistId],
    queryFn: () => fetchTabs(playlistId),
    refetchInterval: 5000,
  });

  const activateTab = useActivateTab(playlistId);

  // track consecutive preview errors per tab
  const [errorMap, setErrorMap] = useState<Record<string, number>>({});

  const [tick, setTick] = useState(Date.now());

  useEffect(() => {
    const id = setInterval(() => setTick(Date.now()), 30000); // 30 seconds
    return () => clearInterval(id);
  }, []);

  const handleImgError = (tabId: string) => {
    setErrorMap((prev) => {
      const cnt = (prev[tabId] ?? 0) + 1;
      return { ...prev, [tabId]: cnt };
    });

    const cnt = (errorMap[tabId] ?? 0) + 1;

    if (cnt < 3) {
      // quick retry
      setTick(Date.now());
    } else {
      // back-off for 40 s before next retry and reset error counter
      setTimeout(() => {
        setErrorMap((prev) => ({ ...prev, [tabId]: 0 }));
        setTick(Date.now());
      }, 40000);
    }
  };

  if (isLoading) return <div>Loading tabs...</div>;
  if (error || !data) return <div>Error loading tabs</div>;

  return (
    <ul className="space-y-2">
      {data.map((tab) => {
        const isActive = tab.id === currentTabId;
        const imgSrc = isActive
          ? `/api/preview_live/${tab.id}`
          : `/api/preview/${tab.id}?t=${tick}`;
        return (
          <li
            key={tab.id}
            onClick={() => activateTab.mutate(tab.id)}
            className={classnames(
              "flex items-center space-x-3 p-2 rounded hover:bg-gray-800 cursor-pointer",
              isActive ? "border border-green-500" : "border border-gray-700"
            )}
          >
            <div className="h-24 object-cover aspect-video rounded border overflow-hidden">
              <img
                src={imgSrc}
                alt={tab.name}
                onError={() => handleImgError(tab.id)}
                className="h-full w-full object-cover aspect-video"
              />
            </div>
            <div className="flex-1">
              <div className="text-sm">{tab.name}</div>
              <a
                href={tab.url}
                className="text-xs text-blue-400 hover:underline"
                target="_blank"
                rel="noopener noreferrer"
              >
                {new URL(tab.url).hostname}
              </a>
            </div>
          </li>
        );
      })}
    </ul>
  );
}; 