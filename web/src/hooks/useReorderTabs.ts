import { useMutation, useQueryClient } from "@tanstack/react-query";

import { apiRequest } from "../api/api";
import { assertRequest } from "../api/request";

export const useReorderTabs = () => {
  const client = useQueryClient();

  return useMutation({
    mutationFn: async ({ playlistId, tabIds }: { playlistId: string; tabIds: string[]; }) =>
      assertRequest(await apiRequest("/playlists/{playlist_id}/reorder", "put", {
        path: { playlist_id: playlistId },
        contentType: "application/json; charset=utf-8",
        data: { tab_ids: tabIds },
      })),
    onSuccess: (_result, { playlistId }) => {
      client.invalidateQueries({ queryKey: ["playlist-tabs", playlistId] });
    },
  });
};
