import { useQuery } from "@tanstack/react-query";
import { useApi } from "./api";

const getTabs = (playlistId: string) => {
  return {
    queryKey: ['tabs', playlistId],
    queryFn: async () => {
      const response = await useApi('/playlists/{playlist_id}/tabs', 'get', {
        path: {
          playlist_id: playlistId,
        },
      });
      return response.data;
    },
  };
};

export const useTabs = (playlistId: string) =>
  useQuery({
    ...getTabs(playlistId),
  }); 