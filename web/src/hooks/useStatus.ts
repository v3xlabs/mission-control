import { useQuery } from "@tanstack/react-query";

import { apiRequest } from "../api/api";
import { failure } from "../api/request";

export const useStatus = () => useQuery({
  queryKey: ["status"],
  queryFn: async () => {
    const response = await apiRequest("/status", "get", {});

    if (response.status !== 200) {
      throw failure(response);
    }

    return response.data;
  },
});
