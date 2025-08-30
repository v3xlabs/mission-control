import { useMutation, useQueryClient } from "@tanstack/react-query";
import { apiRequest } from "../api/api";

import type { components } from "../api/schema.gen";

type ReorderTabsRequest = components["schemas"]["ReorderTabsRequest"];

export const useReorderTabs = () => {
  const qc = useQueryClient();
  
  return useMutation({
    mutationFn: async ({ playlistId, data }: { playlistId: string; data: ReorderTabsRequest }) => {
      return apiRequest("/playlists/{playlist_id}/reorder", "put", {
        path: {
          playlist_id: playlistId,
        },
        contentType: "application/json; charset=utf-8",
        data,
      });
    },
    onSuccess: (_, { playlistId }) => {
      qc.invalidateQueries({ queryKey: ["playlist-tabs", playlistId] });
    },
  });
};