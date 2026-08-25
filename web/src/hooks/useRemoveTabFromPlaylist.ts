import { useMutation, useQueryClient } from "@tanstack/react-query";

import { apiRequest } from "../api/api";
import { assertRequest } from "../api/request";

export const useRemoveTabFromPlaylist = () => {
  const client = useQueryClient();

  return useMutation({
    mutationFn: async ({ playlistId, tabId }: { playlistId: string; tabId: string; }) =>
      assertRequest(await apiRequest("/playlists/{playlist_id}/tabs/{tab_id}", "delete", {
        path: { playlist_id: playlistId, tab_id: tabId },
      })),
    onSuccess: () => {
      client.invalidateQueries({ queryKey: ["playlist-tabs"] });
      client.invalidateQueries({ queryKey: ["playlists"] });
    },
  });
};
