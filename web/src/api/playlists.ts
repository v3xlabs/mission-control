import { useQuery } from "@tanstack/react-query";

import { apiRequest } from "./api";
import { failure } from "./request";

export const usePlaylists = () => useQuery({
  queryKey: ["playlists"],
  queryFn: async () => {
    const response = await apiRequest("/playlists", "get", {});

    if (response.status !== 200) {
      throw failure(response);
    }

    return response.data;
  },
});

export const usePlaylistTabs = (playlistId: string) => useQuery({
  queryKey: ["playlist-tabs", playlistId],
  queryFn: async () => {
    const response = await apiRequest("/playlists/{playlist_id}/tabs", "get", {
      path: { playlist_id: playlistId },
    });

    if (response.status !== 200) {
      throw failure(response);
    }

    return response.data;
  },
});
