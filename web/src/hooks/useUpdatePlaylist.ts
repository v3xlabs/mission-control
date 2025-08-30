import { useMutation, useQueryClient } from "@tanstack/react-query";
import { apiRequest } from "../api/api";

interface UpdatePlaylistRequest {
  name: string;
  interval_seconds: number;
}

export const useUpdatePlaylist = (options?: { onSuccess?: () => void }) => {
  const qc = useQueryClient();
  
  return useMutation({
    mutationFn: async ({ playlistId, data }: { playlistId: string; data: UpdatePlaylistRequest }) => {
      return apiRequest("/playlists/{playlist_id}", "put", {
        path: {
          playlist_id: playlistId,
        },
        contentType: "application/json; charset=utf-8",
        data,
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["playlists"] });
      qc.invalidateQueries({ queryKey: ["status"] });
      options?.onSuccess?.();
    },
  });
};