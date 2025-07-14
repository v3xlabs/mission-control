import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useApi } from "../api/api";

const activateTab = () => {
  return {
    mutationFn: async ({ playlistId, tabId }: { playlistId: string; tabId: string }) => {
      const response = await useApi('/playlists/{playlist_id}/tabs/{tab_id}/activate', 'post', {
        path: {
          playlist_id: playlistId,
          tab_id: tabId,
        },
      });
      return response.data;
    },
  };
};

export const useActivateTab = () => {
  const qc = useQueryClient();
  
  return useMutation({
    ...activateTab(),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["status"] });
    },
  });
}; 