import { useMutation, useQueryClient } from "@tanstack/react-query";
import { apiRequest } from "../api/api";

export const useActivateTab = () => {
  const qc = useQueryClient();
  
  return useMutation({
    mutationFn: async ({ playlistId, tabId }: { playlistId: string; tabId: string }) => {
      const response = await apiRequest('/playlists/{playlist_id}/tabs/{tab_id}/activate', 'post', {
        path: {
          playlist_id: playlistId,
          tab_id: tabId,
        },
      });
      return response.data;
    },
    onMutate: async ({ playlistId, tabId }: { playlistId: string; tabId: string }) => {
      // Only optimistically update the current tab and playlist from status API
      qc.setQueryData(['status'], (old: any) => {
        if (!old) return old;
        return {
          ...old,
          current_playlist: playlistId,
          current_tab: tabId,
        };
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["status"] });
    },
    onError: () => {
      // Revert optimistic updates on error
      qc.invalidateQueries({ queryKey: ["status"] });
    },
  });
}; 