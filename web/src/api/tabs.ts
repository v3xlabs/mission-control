import { useQuery } from "@tanstack/react-query";

import { apiRequest } from "./api";
import { failure } from "./request";

export const useTabs = () => useQuery({
  queryKey: ["tabs"],
  queryFn: async () => {
    const response = await apiRequest("/tabs", "get", {});

    if (response.status !== 200) {
      throw failure(response);
    }

    return response.data;
  },
});
