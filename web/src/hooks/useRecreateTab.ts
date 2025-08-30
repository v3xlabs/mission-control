import { useMutation } from "@tanstack/react-query";
import { apiRequest } from "../api/api";

export const useRecreateTab = () => {
  return useMutation({
    mutationFn: async (tabId: string) => {
      return apiRequest("/tabs/{tab_id}/recreate", "post", {
        path: {
          tab_id: tabId,
        },
      });
    },
  });
};