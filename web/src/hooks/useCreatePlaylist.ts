import { useMutation, useQueryClient } from "@tanstack/react-query";

import { apiRequest } from "../api/api";
import { assertRequest } from "../api/request";

type CreatePlaylistRequest = {
  playlist_id: string;
  name?: string;
  /** A duration such as `30s`, `5m` or `1h`. */
  interval: string;
  hold?: string;
};

export const useCreatePlaylist = (options?: { onSuccess?: () => void; }) => {
  const client = useQueryClient();

  return useMutation({
    mutationFn: async (data: CreatePlaylistRequest) => assertRequest(await apiRequest("/playlists", "post", {
      contentType: "application/json; charset=utf-8",
      data,
    })),
    onSuccess: () => {
      client.invalidateQueries({ queryKey: ["playlists"] });
      options?.onSuccess?.();
    },
  });
};
