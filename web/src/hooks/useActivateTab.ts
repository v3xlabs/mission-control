import { useMutation, useQueryClient } from "@tanstack/react-query";

import { apiRequest } from "../api/api";
import { assertRequest } from "../api/request";

export const useActivateTab = () => {
  const client = useQueryClient();

  return useMutation({
    mutationFn: async ({ playlistId, tabId }: { playlistId: string; tabId: string; }) =>
      assertRequest(await apiRequest("/playlists/{playlist_id}/tabs/{tab_id}/activate", "post", {
        path: { playlist_id: playlistId, tab_id: tabId },
      })),
    onSuccess: () => {
      client.invalidateQueries({ queryKey: ["status"] });
    },
  });
};
