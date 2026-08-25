import { useMutation, useQueryClient } from "@tanstack/react-query";

import { apiRequest } from "../api/api";
import { assertRequest } from "../api/request";

type UpsertTab = {
  tabId: string;
  name?: string;
  url: string;
  persist?: boolean;
  scale?: number;
};

export const useUpsertTab = (options?: { onSuccess?: (tabId: string) => void; }) => {
  const client = useQueryClient();

  return useMutation({
    mutationFn: async ({ tabId, ...data }: UpsertTab) => assertRequest(await apiRequest("/tabs/{tab_id}", "put", {
      path: { tab_id: tabId },
      contentType: "application/json; charset=utf-8",
      data,
    })),
    onSuccess: (_result, { tabId }) => {
      client.invalidateQueries({ queryKey: ["tabs"] });
      client.invalidateQueries({ queryKey: ["playlist-tabs"] });
      options?.onSuccess?.(tabId);
    },
  });
};
