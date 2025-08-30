import { useMutation, useQueryClient } from "@tanstack/react-query";
import { apiRequest } from "../api/api";

export const useDeletePlaylist = () => {
  const qc = useQueryClient();
  
  return useMutation({
    mutationFn: async (playlistId: string) => {
      return apiRequest("/playlists/{playlist_id}", "delete", {
        path: {
          playlist_id: playlistId,
        },
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["playlists"] });
      qc.invalidateQueries({ queryKey: ["status"] });
    },
  });
};