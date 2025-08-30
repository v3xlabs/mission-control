import { useMutation, useQueryClient } from "@tanstack/react-query";
import { apiRequest } from "../api/api";
import type { components } from "../api/schema.gen";

type UpdateTabRequest = components["schemas"]["UpdateTabRequest"];

interface UpdateTabParams extends UpdateTabRequest {
  tabId: string;
}

export const useUpdateTab = (options?: { onSuccess?: () => void }) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ tabId, ...data }: UpdateTabParams) => {
      return apiRequest("/tabs/{tab_id}", "put", {
        path: {
          tab_id: tabId,
        },
        contentType: "application/json; charset=utf-8",
        data,
      });
    },
    onSuccess: () => {
      // Invalidate all related queries
      queryClient.invalidateQueries({ queryKey: ["playlists"] });
      queryClient.invalidateQueries({ queryKey: ["status"] });
      options?.onSuccess?.();
    },
  });
};