import { useMutation, useQueryClient } from "@tanstack/react-query";

import { apiRequest } from "../api/api";
import { assertRequest } from "../api/request";

type Action = "next" | "previous" | "pause" | "resume";

export const usePlayback = () => {
  const client = useQueryClient();

  return useMutation({
    mutationFn: async (action: Action) =>
      assertRequest(await apiRequest(`/playback/${action}` as "/playback/next", "post", {})),
    onSuccess: () => {
      client.invalidateQueries({ queryKey: ["status"] });
    },
  });
};
