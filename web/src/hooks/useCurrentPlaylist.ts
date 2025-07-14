import { useQueryClient } from "@tanstack/react-query";
import { useStatus } from "./useStatus";
import type { components } from "../api/schema.gen";

type DeviceStatus = components["schemas"]["DeviceStatus"];

export const useCurrentPlaylist = () => {
  const queryClient = useQueryClient();
  const statusQuery = useStatus();

  // Helper function to optimistically update current playlist
  const setCurrentPlaylist = (playlistId: string) => {
    queryClient.setQueryData(['status'], (old: DeviceStatus | undefined) => {
      if (!old) return old;
      return {
        ...old,
        current_playlist: playlistId,
        current_tab: undefined, // Reset current tab when playlist changes
      };
    });
  };

  // Helper function to optimistically update current tab
  const setCurrentTab = (tabId: string) => {
    queryClient.setQueryData(['status'], (old: DeviceStatus | undefined) => {
      if (!old) return old;
      return {
        ...old,
        current_tab: tabId,
      };
    });
  };

  return {
    ...statusQuery,
    currentPlaylist: statusQuery.data?.current_playlist,
    currentTab: statusQuery.data?.current_tab,
    setCurrentPlaylist,
    setCurrentTab,
  };
}; 