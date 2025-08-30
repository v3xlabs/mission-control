import { FC, useState, useEffect } from "react";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import * as Switch from "@radix-ui/react-switch";
import * as Tooltip from "@radix-ui/react-tooltip";
import { DotsHorizontalIcon, ReloadIcon, TrashIcon, Pencil2Icon, EnterIcon } from "@radix-ui/react-icons";
import { useRefreshTab } from "../hooks/useRefreshTab";
import { useRecreateTab } from "../hooks/useRecreateTab";
import { useActivateTab } from "../hooks/useActivateTab";
import { useToggleTabEnabled } from "../hooks/useToggleTabEnabled";
import { useDeleteTab } from "../hooks/useDeleteTab";
import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { RiDragDropLine } from "react-icons/ri";
import { useCurrentPlaylist } from "../hooks/useCurrentPlaylist";
import type { components } from "../api/schema.gen";
import { useStatus } from "../hooks/useStatus";
import { usePlaylists } from "../api/playlists";
import { ProgressCircle } from "./ProgressCircle";
import { useQueryClient } from "@tanstack/react-query";

type TabInfo = components["schemas"]["TabInfo"];

interface TabCardProps {
  tab: TabInfo;
  playlistId: string;
  enabled?: boolean;
  onEdit?: (tab: TabInfo) => void;
  onDelete?: (tabId: string) => void;
}

export const TabCard: FC<TabCardProps> = ({ tab, playlistId, enabled = true, onEdit, onDelete }) => {
  const { data: status, refetch: refetchStatus } = useStatus();
  const { data: playlists } = usePlaylists();
  const queryClient = useQueryClient();
  const currentTab = status?.current_tab;
  const currentPlaylist = status?.current_playlist;

  // track consecutive preview errors per tab
  const [errorMap, setErrorMap] = useState<Record<string, number>>({});
  const [tick, setTick] = useState(Date.now());
  
  useEffect(() => {
    const id = setInterval(() => setTick(Date.now()), 30000); // 30 seconds
    return () => clearInterval(id);
  }, []);
  
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: tab.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  const isActive = currentPlaylist === playlistId && currentTab === tab.id;
  
  // Progress circle data for active tab
  const currentPlaylistData = playlists?.find(p => p.id === currentPlaylist);
  const shouldShowProgress = isActive && 
    currentPlaylistData && 
    currentPlaylistData.interval_seconds > 0 && 
    status?.current_tab_opened_at;
  
  const imgSrc = isActive
    ? `/api/preview_live/${tab.id}`
    : `/api/preview/${tab.id}?t=${tick}`;

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

  const refreshTabMutation = useRefreshTab();
  const recreateTabMutation = useRecreateTab();
  const activateTabMutation = useActivateTab();
  const toggleEnabledMutation = useToggleTabEnabled();
  const deleteTabMutation = useDeleteTab({
    onSuccess: () => onDelete?.(tab.id)
  });

  return (
    <Tooltip.Provider>
      <div
        ref={setNodeRef}
        style={style}
        className={`group relative bg-gray-800 border border-gray-700 rounded-lg p-4 transition-all duration-200 ${
          isDragging ? "opacity-50 shadow-lg" : "hover:shadow-md hover:border-gray-600"
        } ${!enabled ? "opacity-60" : ""}`}
      >
        {/* Drag handle */}
        <div
          {...attributes}
          {...listeners}
          className="absolute top-2 left-2 opacity-0 group-hover:opacity-100 transition-opacity cursor-grab active:cursor-grabbing"
        >
          <RiDragDropLine className="w-4 h-4 text-gray-500" />
        </div>

        {/* Tab content with preview */}
        <div className="space-y-4">
          {/* Preview image */}
          <div 
            className={`w-full rounded border overflow-hidden border-gray-300 ${
              isActive ? "border-green-500 border-2" : ""
            }`}
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

          {/* Tab info */}
          <div className="flex-1 min-w-0">
            <div className="flex items-center justify-between mb-2">
              <h4 className="font-medium text-gray-100 truncate">{tab.name}</h4>
              <div className="flex items-center gap-2 flex-shrink-0">
                {shouldShowProgress && currentPlaylistData && status?.current_tab_opened_at && (
                  <ProgressCircle
                    duration={currentPlaylistData.interval_seconds}
                    startTime={status.current_tab_opened_at}
                    size={20}
                    strokeWidth={2}
                    color="#10b981"
                    onComplete={() => {
                      // Refetch status 10ms after progress completion to get updated tab and timestamp
                      refetchStatus();
                    }}
                  />
                )}
                <span className="text-xs text-gray-400">#{tab.order_index + 1}</span>
              </div>
            </div>
            
            <a
              href={tab.url}
              className="text-sm text-gray-300 hover:text-gray-200 hover:underline block truncate mb-3"
              target="_blank"
              rel="noopener noreferrer"
            >
              {new URL(tab.url).hostname}
            </a>
            
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Switch.Root
                  checked={enabled}
                  onCheckedChange={(checked) => toggleEnabledMutation.mutate({ playlistId, tabId: tab.id, enabled: checked })}
                  disabled={toggleEnabledMutation.isPending}
                  className="w-11 h-6 bg-gray-600 rounded-full relative data-[state=checked]:bg-green-600 outline-none cursor-default"
                >
                  <Switch.Thumb className="block w-5 h-5 bg-white rounded-full transition-transform duration-100 translate-x-0.5 will-change-transform data-[state=checked]:translate-x-[22px]" />
                </Switch.Root>
                <span className="text-xs text-gray-400">
                  {enabled ? "Enabled" : "Disabled"}
                </span>
              </div>

              <div className="flex items-center gap-1">
                <Tooltip.Root>
                  <Tooltip.Trigger asChild>
                    <button
                      onClick={() => refreshTabMutation.mutate(tab.id)}
                      disabled={refreshTabMutation.isPending}
                      className="p-1 text-gray-400 hover:text-blue-400 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      <ReloadIcon className={`w-4 h-4 ${refreshTabMutation.isPending ? "animate-spin" : ""}`} />
                    </button>
                  </Tooltip.Trigger>
                  <Tooltip.Portal>
                    <Tooltip.Content
                      className="bg-gray-900 text-white text-xs px-2 py-1 rounded shadow-lg"
                      sideOffset={5}
                    >
                      Refresh Tab
                      <Tooltip.Arrow className="fill-gray-900" />
                    </Tooltip.Content>
                  </Tooltip.Portal>
                </Tooltip.Root>

                <Tooltip.Root>
                  <Tooltip.Trigger asChild>
                    <button
                      onClick={() => activateTabMutation.mutate({ playlistId, tabId: tab.id })}
                      disabled={activateTabMutation.isPending || !enabled}
                      className="p-1 text-gray-400 hover:text-green-400 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      <EnterIcon className="w-4 h-4" />
                    </button>
                  </Tooltip.Trigger>
                  <Tooltip.Portal>
                    <Tooltip.Content
                      className="bg-gray-900 text-white text-xs px-2 py-1 rounded shadow-lg"
                      sideOffset={5}
                    >
                      Activate Tab
                      <Tooltip.Arrow className="fill-gray-900" />
                    </Tooltip.Content>
                  </Tooltip.Portal>
                </Tooltip.Root>
              </div>
            </div>
          </div>
        </div>

        {/* Action menu */}
        <DropdownMenu.Root>
          <DropdownMenu.Trigger asChild>
            <button className="absolute top-2 right-2 p-1 text-gray-500 hover:text-gray-300 opacity-0 group-hover:opacity-100 transition-opacity">
              <DotsHorizontalIcon className="w-4 h-4" />
            </button>
          </DropdownMenu.Trigger>

          <DropdownMenu.Portal>
            <DropdownMenu.Content
              className="bg-gray-800 border border-gray-600 rounded-lg shadow-lg p-1 z-50 min-w-[160px]"
              sideOffset={5}
            >
              <DropdownMenu.Item
                onSelect={() => onEdit?.(tab)}
                className="flex items-center gap-2 px-3 py-2 text-sm text-gray-300 hover:bg-gray-700 rounded-md cursor-pointer outline-none"
              >
                <Pencil2Icon className="w-4 h-4" />
                Edit Tab
              </DropdownMenu.Item>

              <DropdownMenu.Separator className="h-px bg-gray-600 m-1" />

              <DropdownMenu.Item
                onSelect={() => recreateTabMutation.mutate(tab.id)}
                disabled={recreateTabMutation.isPending}
                className="flex items-center gap-2 px-3 py-2 text-sm text-gray-300 hover:bg-gray-700 rounded-md cursor-pointer outline-none disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <ReloadIcon className={`w-4 h-4 ${recreateTabMutation.isPending ? "animate-spin" : ""}`} />
                Recreate Tab
              </DropdownMenu.Item>

              <DropdownMenu.Separator className="h-px bg-gray-600 m-1" />

              <DropdownMenu.Item
                onSelect={() => deleteTabMutation.mutate(tab.id)}
                disabled={deleteTabMutation.isPending}
                className="flex items-center gap-2 px-3 py-2 text-sm text-red-400 hover:bg-red-900/20 rounded-md cursor-pointer outline-none disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <TrashIcon className="w-4 h-4" />
                Delete Tab
              </DropdownMenu.Item>
            </DropdownMenu.Content>
          </DropdownMenu.Portal>
        </DropdownMenu.Root>
      </div>
    </Tooltip.Provider>
  );
};