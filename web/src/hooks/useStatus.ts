import { useQuery } from "@tanstack/react-query";
import { useApi } from "../api/api";

const getStatus = () => {
  return {
    queryKey: ['status'],
    queryFn: async () => {
      const response = await useApi('/status', 'get', {});
      return response.data;
    },
  };
};

export const useStatus = () =>
  useQuery({
    ...getStatus(),
    refetchInterval: 2000,
  }); 