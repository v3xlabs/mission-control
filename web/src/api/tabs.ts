import { useQuery } from "@tanstack/react-query";
import { apiRequest } from "./api";

const getTabs = (playlistId: string) => {
  return {
    queryKey: ['tabs', playlistId],
    queryFn: async () => {
      const response = await apiRequest('/playlists/{playlist_id}/tabs', 'get', {
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