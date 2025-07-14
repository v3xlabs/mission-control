import { useMutation, useQueryClient } from "@tanstack/react-query";

export const useActivatePlaylist = () => {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (playlistId: string) => {
      await fetch(`/api/playlists/${playlistId}/activate`, { method: "POST" });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["status"] });
      qc.invalidateQueries({ queryKey: ["playlists"] });
    },
  });
}; 