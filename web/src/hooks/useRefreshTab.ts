import { useMutation } from "@tanstack/react-query";
import { apiRequest } from "../api/api";

export const useRefreshTab = () => {
  return useMutation({
    mutationFn: async (tabId: string) => {
      return apiRequest("/tabs/{tab_id}/refresh", "post", {
        path: {
          tab_id: tabId,
        },
      });
    },
  });
};