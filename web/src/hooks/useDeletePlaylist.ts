import { useMutation, useQueryClient } from "@tanstack/react-query";

import { apiRequest } from "../api/api";
import { assertRequest } from "../api/request";

export const useDeletePlaylist = () => {
  const client = useQueryClient();

  return useMutation({
    mutationFn: async (playlistId: string) => assertRequest(await apiRequest("/playlists/{playlist_id}", "delete", {
      path: { playlist_id: playlistId },
    })),
    onSuccess: () => {
      client.invalidateQueries({ queryKey: ["playlists"] });
      client.invalidateQueries({ queryKey: ["status"] });
    },
  });
};
