import { useMutation, useQueryClient } from "@tanstack/react-query";

export const useActivateTab = (playlistId: string) => {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (tabId: string) => {
      await fetch(`/api/playlists/${playlistId}/tabs/${tabId}/activate`, {
        method: "POST",
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["status"] });
    },
  });
}; 