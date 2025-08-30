import { FC, useEffect, useState } from "react";
import classnames from "classnames";
import { useCurrentPlaylist } from "../hooks/useCurrentPlaylist";
import { useActivateTab } from "../hooks/useActivateTab";
import { useTabs } from "../api/tabs";
import type { components } from "../api/schema.gen";

type TabInfo = components["schemas"]["TabInfo"];

interface Props {
  playlistId: string;
}

export const TabList: FC<Props> = ({ playlistId }) => {
  const { data: currentPlaylist } = useCurrentPlaylist();
  const currentTab = ""; // TODO: need to add current tab to status
  const currentTabId =
    currentPlaylist === playlistId ? currentTab : null;

  const { data, error, isLoading } = useTabs(playlistId);

  const activateTab = useActivateTab();

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
    <ul className="flex gap-2 flex-wrap">
      {data.map((tab: TabInfo) => {
        const isActive = tab.id === currentTabId;
        const imgSrc = isActive
          ? `/api/preview_live/${tab.id}`
          : `/api/preview/${tab.id}?t=${tick}`;
        return (
          <li
            key={tab.id}
            onClick={() => activateTab.mutate({ playlistId, tabId: tab.id })}
            className={classnames(
              "flex items-center space-x-3 p-2 rounded hover:bg-gray-800 cursor-pointer transition-colors",
              isActive ? "border border-green-500 bg-green-900/20" : "border border-gray-700",
              activateTab.isPending ? "opacity-50 cursor-not-allowed" : ""
            )}
          >
            <div 
              className="h-24 object-cover rounded border overflow-hidden"
              style={{
                aspectRatio: tab.viewport_width && tab.viewport_height 
                  ? `${tab.viewport_width} / ${tab.viewport_height}`
                  : '16 / 9' // fallback to 16:9 if dimensions not available
              }}
            >
              <img
                src={imgSrc}
                alt={tab.name}
                onError={() => handleImgError(tab.id)}
                className="h-full w-full object-cover"
                style={{
                  aspectRatio: tab.viewport_width && tab.viewport_height 
                    ? `${tab.viewport_width} / ${tab.viewport_height}`
                    : '16 / 9' // fallback to 16:9 if dimensions not available
                }}
              />
            </div>
            <div className="flex-1">
              <div className="text-sm">{tab.name}</div>
              <a
                href={tab.url}
                className="text-xs text-gray-300 hover:text-gray-200 hover:underline"
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