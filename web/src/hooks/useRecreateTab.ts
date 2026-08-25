import { useMutation } from "@tanstack/react-query";

import { apiRequest } from "../api/api";
import { assertRequest } from "../api/request";

export const useRecreateTab = () => useMutation({
  mutationFn: async (tabId: string) => assertRequest(await apiRequest("/tabs/{tab_id}/recreate", "post", {
    path: { tab_id: tabId },
  })),
});
