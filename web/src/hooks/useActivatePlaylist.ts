import { useMutation, useQueryClient } from "@tanstack/react-query";

import { apiRequest } from "../api/api";
import { assertRequest } from "../api/request";

export const useActivatePlaylist = () => {
  const client = useQueryClient();

  return useMutation({
    mutationFn: async (playlistId: string) => assertRequest(await apiRequest("/playlists/{playlist_id}/activate", "post", {
      path: { playlist_id: playlistId },
    })),
    onSuccess: () => {
      client.invalidateQueries({ queryKey: ["status"] });
      client.invalidateQueries({ queryKey: ["playlists"] });
    },
  });
};
