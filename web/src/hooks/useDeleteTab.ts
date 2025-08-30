import { useMutation, useQueryClient } from "@tanstack/react-query";
import { apiRequest } from "../api/api";

export const useDeleteTab = (options?: { onSuccess?: () => void }) => {
  const qc = useQueryClient();
  
  return useMutation({
    mutationFn: async (tabId: string) => {
      return apiRequest("/tabs/{tab_id}", "delete", {
        path: {
          tab_id: tabId,
        },
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["playlists"] });
      qc.invalidateQueries({ queryKey: ["playlist-tabs"] });
      options?.onSuccess?.();
    },
  });
};