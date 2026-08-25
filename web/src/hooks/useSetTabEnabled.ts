import { useMutation, useQueryClient } from "@tanstack/react-query";

import { apiRequest } from "../api/api";
import { assertRequest } from "../api/request";

export const useSetTabEnabled = () => {
  const client = useQueryClient();

  return useMutation({
    mutationFn: async ({ playlistId, tabId, enabled }: {
      playlistId: string;
      tabId: string;
      enabled: boolean;
    }) => assertRequest(await apiRequest("/playlists/{playlist_id}/tabs/{tab_id}/enabled", "put", {
      path: { playlist_id: playlistId, tab_id: tabId },
      contentType: "application/json; charset=utf-8",
      data: { enabled },
    })),
    onSuccess: (_result, { playlistId }) => {
      client.invalidateQueries({ queryKey: ["playlist-tabs", playlistId] });
    },
  });
};
