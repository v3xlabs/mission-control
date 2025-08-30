import { useMutation, useQueryClient } from "@tanstack/react-query";
import { apiRequest } from "../api/api";
import type { components } from "../api/schema.gen";

type CreateTabRequest = components["schemas"]["CreateTabRequest"];

export const useCreateTab = (options?: { onSuccess?: (tabId: string) => void }) => {
  const qc = useQueryClient();
  
  return useMutation({
    mutationFn: async (data: CreateTabRequest) => {
      return apiRequest("/tabs", "post", {
        contentType: "application/json; charset=utf-8",
        data,
      });
    },
    onSuccess: (result) => {
      qc.invalidateQueries({ queryKey: ["tabs"] });
      options?.onSuccess?.(result.data.id);
    },
  });
};