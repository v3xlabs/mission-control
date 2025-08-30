import { useMutation, useQueryClient } from "@tanstack/react-query";
import { apiRequest } from "../api/api";

interface CreatePlaylistRequest {
  id: string;
  name: string;
  interval_seconds: number;
}

export const useCreatePlaylist = (options?: { onSuccess?: () => void }) => {
  const qc = useQueryClient();
  
  return useMutation({
    mutationFn: async (data: CreatePlaylistRequest) => {
      return apiRequest("/playlists", "post", {
        contentType: "application/json; charset=utf-8",
        data,
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["playlists"] });
      options?.onSuccess?.();
    },
  });
};