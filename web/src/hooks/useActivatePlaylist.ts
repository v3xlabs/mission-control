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
    },
    onError: () => {
      // Revert optimistic updates on error
      qc.invalidateQueries({ queryKey: ["status"] });
    },
  });
}; 