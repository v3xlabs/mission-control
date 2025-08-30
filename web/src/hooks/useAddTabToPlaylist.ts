import { useMutation, useQueryClient } from "@tanstack/react-query";
import { apiRequest } from "../api/api";
import type { components } from "../api/schema.gen";

type AddTabToPlaylistRequest = components["schemas"]["AddTabToPlaylistRequest"];

interface AddTabToPlaylistParams {
  playlistId: string;
  data: AddTabToPlaylistRequest;
}

export const useAddTabToPlaylist = (options?: { onSuccess?: () => void }) => {
  const qc = useQueryClient();
  
  return useMutation({
    mutationFn: async ({ playlistId, data }: AddTabToPlaylistParams) => {
      return apiRequest("/playlists/{playlist_id}/tabs", "post", {
        path: {
          playlist_id: playlistId,
        },
        contentType: "application/json; charset=utf-8",
        data,
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["playlist-tabs"] });
      qc.invalidateQueries({ queryKey: ["playlists"] });
      options?.onSuccess?.();
    },
  });
};