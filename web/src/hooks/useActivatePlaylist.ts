import { useMutation, useQueryClient } from "@tanstack/react-query";
import { apiRequest } from "../api/api";

export const useActivatePlaylist = () => {
  const qc = useQueryClient();
  
  return useMutation({
    mutationFn: async (playlistId: string) => {
      const response = await apiRequest('/playlists/{playlist_id}/activate', 'post', {
        path: {
          playlist_id: playlistId,
        },
      });
      return response.data;
    },
    onMutate: async (playlistId: string) => {
      // Only optimistically update the current playlist from status API
      qc.setQueryData(['status'], (old: any) => {
        if (!old) return old;
        return {
          ...old,
          current_playlist: playlistId,
          current_tab: undefined, // Reset current tab when playlist changes
        };
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["status"] });
      qc.invalidateQueries({ queryKey: ["playlists"] });
    },
    onError: () => {
      // Revert optimistic updates on error
      qc.invalidateQueries({ queryKey: ["status"] });
      qc.invalidateQueries({ queryKey: ["playlists"] });
    },
  });
}; 