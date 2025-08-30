import { useQuery } from "@tanstack/react-query";
import { apiRequest } from "./api";

const getPlaylists = () => {
  return {
    queryKey: ['playlists'],
    queryFn: async () => {
      const response = await apiRequest('/playlists', 'get', {});
      return response.data;
    },
  };
};

const getPlaylistTabs = (playlistId: string) => {
  return {
    queryKey: ['playlist-tabs', playlistId],
    queryFn: async () => {
      const response = await apiRequest('/playlists/{playlist_id}/tabs', 'get', {
        path: { playlist_id: playlistId }
      });
      return response.data;
    },
  };
};

export const usePlaylists = () =>
  useQuery(getPlaylists());

export const usePlaylistTabs = (playlistId: string) =>
  useQuery(getPlaylistTabs(playlistId)); 