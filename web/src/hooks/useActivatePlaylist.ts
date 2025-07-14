import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useApi } from "../api/api";

const activatePlaylist = () => {
  return {
    mutationFn: async (playlistId: string) => {
      const response = await useApi('/playlists/{playlist_id}/activate', 'post', {
        path: {
          playlist_id: playlistId,
        },
      });
      return response.data;
    },
  };
};

export const useActivatePlaylist = () => {
  const qc = useQueryClient();
  
  return useMutation({
    ...activatePlaylist(),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["status"] });
      qc.invalidateQueries({ queryKey: ["playlists"] });
    },
  });
}; 