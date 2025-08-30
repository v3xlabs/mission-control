import { useMutation, useQueryClient } from "@tanstack/react-query";
import { apiRequest } from "../api/api";

export const useToggleTabEnabled = () => {
  const qc = useQueryClient();
  
  return useMutation({
    mutationFn: async ({ playlistId, tabId, enabled }: { playlistId: string; tabId: string; enabled: boolean }) => {
      return apiRequest("/playlists/{playlist_id}/tabs/{tab_id}/toggle", "put", {
        path: {
          playlist_id: playlistId,
          tab_id: tabId,
        },
        contentType: "application/json; charset=utf-8",
        data: { enabled },
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["playlists"] });
    },
  });
};