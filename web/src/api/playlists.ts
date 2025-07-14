import { useQuery } from "@tanstack/react-query";
import { useApi } from "./api";

const getPlaylists = () => {
  return {
    queryKey: ['playlists'],
    queryFn: async () => {
      const response = await useApi('/playlists', 'get', {});
      return response.data;
    },
  };
};

export const usePlaylists = () =>
  useQuery(getPlaylists()); 