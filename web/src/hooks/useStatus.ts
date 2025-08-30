import { useQuery } from "@tanstack/react-query";
import { apiRequest } from "../api/api";

export const useStatus = () => {
  return useQuery({
    queryKey: ['status'],
    queryFn: async () => {
      const response = await apiRequest('/status', 'get', {});
      return response.data;
    },
    refetchInterval: 5000,
    staleTime: 500,
  });
};
