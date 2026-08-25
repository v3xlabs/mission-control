import { useMutation, useQueryClient } from "@tanstack/react-query";

import { apiRequest } from "../api/api";
import { assertRequest } from "../api/request";

export const useDeleteTab = (options?: { onSuccess?: () => void; }) => {
  const client = useQueryClient();

  return useMutation({
    mutationFn: async (tabId: string) => assertRequest(await apiRequest("/tabs/{tab_id}", "delete", {
      path: { tab_id: tabId },
    })),
    onSuccess: () => {
      client.invalidateQueries({ queryKey: ["tabs"] });
      client.invalidateQueries({ queryKey: ["playlist-tabs"] });
      client.invalidateQueries({ queryKey: ["playlists"] });
      options?.onSuccess?.();
    },
  });
};
